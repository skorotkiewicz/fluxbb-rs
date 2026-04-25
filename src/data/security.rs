#[cfg(feature = "server")]
use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use dioxus::prelude::ServerFnError;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use rand::RngCore;
#[cfg(feature = "server")]
use serde::Deserialize;
#[cfg(feature = "server")]
use sha2::{Digest, Sha256};

#[cfg(feature = "server")]
use super::{
    db::{run_json_query, run_scalar_i64, sql_literal},
    Group, SessionUser,
};

const SESSION_COOKIE: &str = "fluxbb_rs_session";
const CSRF_COOKIE: &str = "fluxbb_rs_csrf";
const SESSION_MAX_AGE_SECS: i64 = 60 * 60 * 24 * 14;

pub fn cookie_name() -> &'static str {
    SESSION_COOKIE
}

pub fn csrf_cookie_name() -> &'static str {
    CSRF_COOKIE
}

pub fn cookie_max_age() -> i64 {
    SESSION_MAX_AGE_SECS
}

pub fn clean_error(e: ServerFnError) -> String {
    let s = e.to_string();
    let prefix = "error running server function: ";
    let trimmed = if let Some(rest) = s.strip_prefix(prefix) {
        rest
    } else {
        &s
    };

    if let Some(idx) = trimmed.rfind(" (details: ") {
        trimmed[..idx].to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(feature = "server")]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Permission {
    PostTopics,
    PostReplies,
    EditPosts,
    DeletePosts,
    DeleteTopic,
    MoveTopic,
    StickyTopic,
    CloseTopic,
    ManageUsers,
    ManageForums,
    ManageCategories,
    ManageBans,
    ManageGroups,
    ManageSettings,
    Moderator,
}

#[cfg(feature = "server")]
pub(crate) async fn check_ban(username: &str, email: &str) -> Result<Option<String>, String> {
    let now = unix_now();

    #[derive(Deserialize)]
    struct BanRow {
        message: String,
    }

    let ban = run_json_query::<Option<BanRow>>(&format!(
        "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT message FROM bans WHERE ((username <> '' AND LOWER(username) = LOWER({u})) OR (email <> '' AND LOWER(email) = LOWER({e}))) AND (expires_at IS NULL OR expires_at > {now}) LIMIT 1) r), 'null'::json);",
        u = sql_literal(username),
        e = sql_literal(email),
        now = now,
    ))
    .await?;
    Ok(ban.map(|b| b.message))
}

#[cfg(feature = "server")]
pub(crate) async fn require_session(headers: &HeaderMap) -> Result<SessionUser, String> {
    let token = parse_session_cookie(headers)
        .ok_or_else(|| "You must be signed in to do this.".to_string())?;

    run_json_query::<Option<SessionUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(r) FROM (
                 SELECT u.id, u.username, u.title, u.group_id,
                        g.post_topics, g.post_replies, g.edit_posts, g.delete_posts,
                        g.delete_topic, g.move_topic, g.sticky_topic, g.close_topic,
                        g.manage_users, g.manage_forums, g.manage_categories, g.manage_bans,
                        g.manage_groups, g.manage_settings, g.is_moderator, g.is_admin
                 FROM forum_sessions s
                 JOIN users u ON u.id = s.user_id
                 JOIN groups g ON g.id = u.group_id
                 WHERE s.token = {token} AND s.expires_at > EXTRACT(EPOCH FROM now())::bigint
                 LIMIT 1
             ) r
         ), 'null'::json);",
        token = sql_literal(&token),
    ))
    .await?
    .ok_or_else(|| "Session expired. Please sign in again.".to_string())
}

#[cfg(feature = "server")]
pub(crate) async fn require_session_csrf(headers: &HeaderMap) -> Result<SessionUser, String> {
    let user = require_session(headers).await?;
    validate_csrf(headers).await?;
    Ok(user)
}

#[cfg(feature = "server")]
pub(crate) async fn get_group(group_id: i32) -> Result<Group, String> {
    run_json_query::<Option<Group>>(&format!(
        "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT id, title, read_board, post_topics, post_replies, edit_posts, delete_posts, is_moderator, is_admin FROM groups WHERE id = {}) r), 'null'::json);",
        group_id,
    ))
    .await?
    .ok_or_else(|| "Group not found.".to_string())
}

#[cfg(feature = "server")]
pub(crate) async fn check_flood(user_id: i32, is_admin: bool) -> Result<(), String> {
    if is_admin {
        return Ok(());
    }

    let last_post = run_scalar_i64(&format!(
        "SELECT COALESCE(EXTRACT(EPOCH FROM MAX(posted_at::timestamp))::bigint, 0) FROM posts WHERE author_id = {};",
        user_id
    ))
    .await?;
    let now = unix_now();
    if last_post > 0 && now - last_post < 30 {
        return Err("Please wait at least 30 seconds between posts.".to_string());
    }

    Ok(())
}

#[cfg(feature = "server")]
pub(crate) async fn check_permission(
    user: &SessionUser,
    permission: Permission,
) -> Result<(), String> {
    if user.is_admin {
        return Ok(());
    }

    let allowed = match permission {
        Permission::PostTopics => user.post_topics,
        Permission::PostReplies => user.post_replies,
        Permission::EditPosts => user.edit_posts,
        Permission::DeletePosts => user.delete_posts,
        Permission::DeleteTopic => user.delete_topic,
        Permission::MoveTopic => user.move_topic,
        Permission::StickyTopic => user.sticky_topic,
        Permission::CloseTopic => user.close_topic,
        Permission::ManageUsers => user.manage_users,
        Permission::ManageForums => user.manage_forums,
        Permission::ManageCategories => user.manage_categories,
        Permission::ManageBans => user.manage_bans,
        Permission::ManageGroups => user.manage_groups,
        Permission::ManageSettings => user.manage_settings,
        Permission::Moderator => user.is_moderator,
    };

    if allowed {
        Ok(())
    } else {
        Err("You do not have permission to do this.".to_string())
    }
}

#[cfg(feature = "server")]
pub(crate) fn parse_session_cookie(headers: &HeaderMap) -> Option<String> {
    let raw_cookie = headers.get("cookie")?.to_str().ok()?;

    raw_cookie.split(';').find_map(|part| {
        let trimmed = part.trim();
        let (name, value) = trimmed.split_once('=')?;
        if name == SESSION_COOKIE {
            Some(value.to_string())
        } else {
            None
        }
    })
}

#[cfg(feature = "server")]
fn parse_csrf_cookie(headers: &HeaderMap) -> Option<String> {
    let raw_cookie = headers.get("cookie")?.to_str().ok()?;

    raw_cookie.split(';').find_map(|part| {
        let trimmed = part.trim();
        let (name, value) = trimmed.split_once('=')?;
        if name == CSRF_COOKIE {
            Some(value.to_string())
        } else {
            None
        }
    })
}

#[cfg(feature = "server")]
async fn validate_csrf(headers: &HeaderMap) -> Result<(), String> {
    let Some(session_token) = parse_session_cookie(headers) else {
        return Err("Session expired. Please sign in again.".to_string());
    };
    let Some(csrf_token) = parse_csrf_cookie(headers) else {
        return Err("CSRF token missing. Please refresh the page.".to_string());
    };

    #[derive(Deserialize)]
    struct CsrfRow {
        csrf_token: String,
    }

    let row = run_json_query::<Option<CsrfRow>>(&format!(
        "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT csrf_token FROM forum_sessions WHERE token = {token} AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1) r), 'null'::json);",
        token = sql_literal(&session_token),
    ))
    .await?;

    match row {
        Some(r) if r.csrf_token == csrf_token => Ok(()),
        _ => Err("Invalid CSRF token. Please refresh the page.".to_string()),
    }
}

#[cfg(feature = "server")]
pub(crate) fn validate_username(username: &str) -> Result<(), String> {
    let length = username.chars().count();
    if length < 2 {
        return Err("Username must be at least 2 characters long.".to_string());
    }

    if length > 25 {
        return Err("Username must be 25 characters or fewer.".to_string());
    }

    if username.eq_ignore_ascii_case("guest") {
        return Err("The username Guest is reserved.".to_string());
    }

    if username.parse::<IpAddr>().is_ok() {
        return Err("Usernames cannot be IP addresses.".to_string());
    }

    if username.contains('[')
        || username.contains(']')
        || username.contains('"')
        || username.contains('\'')
    {
        return Err("Username contains reserved characters.".to_string());
    }

    let lower = username.to_lowercase();
    for tag in [
        "[b]", "[i]", "[u]", "[img]", "[url]", "[quote]", "[code]", "[email]", "[list]", "[topic]",
        "[post]", "[forum]", "[user]",
    ] {
        if lower.contains(tag) {
            return Err("Username cannot contain BBCode-like tags.".to_string());
        }
    }

    Ok(())
}

#[cfg(feature = "server")]
pub(crate) fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() || !email.contains('@') {
        return Err("Enter a valid email address.".to_string());
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err("Enter a valid email address.".to_string());
    };

    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err("Enter a valid email address.".to_string());
    }

    Ok(())
}

#[cfg(feature = "server")]
pub(crate) fn normalize_username(username: &str) -> String {
    username.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(feature = "server")]
pub(crate) fn hash_password(password: &str, salt: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(salt.as_bytes());
    digest.update(password.as_bytes());
    let bytes = digest.finalize();
    format!("sha256${salt}${}", bytes_to_hex(&bytes))
}

#[cfg(feature = "server")]
pub(crate) fn verify_password(password: &str, stored_hash: &str) -> bool {
    let mut parts = stored_hash.split('$');
    let algorithm = parts.next();
    let salt = parts.next();
    let hash = parts.next();

    match (algorithm, salt, hash) {
        (Some("sha256"), Some(salt), Some(hash)) => {
            hash_password(password, salt) == format!("sha256${salt}${hash}")
        }
        _ => false,
    }
}

#[cfg(feature = "server")]
pub(crate) fn random_hex(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    rand::rng().fill_bytes(&mut bytes);
    bytes_to_hex(&bytes)
}

#[cfg(feature = "server")]
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(feature = "server")]
pub(crate) fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
