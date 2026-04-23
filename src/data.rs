use std::{
    cmp::Reverse,
    net::IpAddr,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use dioxus::prelude::*;
use http::HeaderMap;
use rand::RngCore;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DATABASE_URL: &str = "postgresql://dev:password@localhost:5432/fluxbb";
const SESSION_COOKIE: &str = "fluxbb_rs_session";
const SESSION_MAX_AGE_SECS: i64 = 60 * 60 * 24 * 14;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub users: Vec<UserProfile>,
    pub topics: Vec<Topic>,
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardMeta {
    pub title: String,
    pub tagline: String,
    pub announcement_title: String,
    pub announcement_body: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forum {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub moderators: Vec<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub status: String,
    pub joined_at: String,
    pub post_count: i32,
    pub location: String,
    pub about: String,
    pub last_seen: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub id: i32,
    pub forum_id: i32,
    pub author_id: i32,
    pub subject: String,
    pub status: TopicStatus,
    pub views: i32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub activity_rank: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub topic_id: i32,
    pub author_id: i32,
    pub posted_at: String,
    pub edited_at: Option<String>,
    pub body: Vec<String>,
    pub signature: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicStatus {
    Pinned,
    Hot,
    Resolved,
    Fresh,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardStats {
    pub members: usize,
    pub topics: usize,
    pub posts: usize,
    pub newest_member: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForumSnapshot {
    pub forum: Forum,
    pub topic_count: usize,
    pub post_count: usize,
    pub last_topic_id: i32,
    pub last_topic_subject: String,
    pub last_post_author: String,
    pub last_posted_at: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchResults {
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: i32,
    pub username: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: SessionUser,
    pub session_token: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub location: String,
    pub about: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredAuthUser {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub password_hash: String,
}

impl AppData {
    pub fn board_stats(&self) -> BoardStats {
        let newest_member = self
            .users
            .iter()
            .max_by_key(|user| user.id)
            .map(|user| user.username.clone())
            .unwrap_or_else(|| "guest".to_string());

        BoardStats {
            members: self.users.len(),
            topics: self.topics.len(),
            posts: self.posts.len(),
            newest_member,
        }
    }

    pub fn categories_sorted(&self) -> Vec<Category> {
        let mut categories = self.categories.clone();
        categories.sort_by_key(|category| category.sort_order);
        categories
    }

    pub fn forums_in_category(&self, category_id: i32) -> Vec<Forum> {
        let mut forums = self
            .forums
            .iter()
            .filter(|forum| forum.category_id == category_id)
            .cloned()
            .collect::<Vec<_>>();
        forums.sort_by_key(|forum| forum.sort_order);
        forums
    }

    pub fn forum(&self, id: i32) -> Option<Forum> {
        self.forums.iter().find(|forum| forum.id == id).cloned()
    }

    pub fn user(&self, id: i32) -> Option<UserProfile> {
        self.users.iter().find(|user| user.id == id).cloned()
    }

    pub fn topic(&self, id: i32) -> Option<Topic> {
        self.topics.iter().find(|topic| topic.id == id).cloned()
    }

    pub fn topics_for_forum(&self, forum_id: i32) -> Vec<Topic> {
        let mut topics = self
            .topics
            .iter()
            .filter(|topic| topic.forum_id == forum_id)
            .cloned()
            .collect::<Vec<_>>();
        topics.sort_by_key(|topic| Reverse(topic.activity_rank));
        topics
    }

    pub fn posts_for_topic(&self, topic_id: i32) -> Vec<Post> {
        let mut posts = self
            .posts
            .iter()
            .filter(|post| post.topic_id == topic_id)
            .cloned()
            .collect::<Vec<_>>();
        posts.sort_by_key(|post| post.position);
        posts
    }

    pub fn recent_topics(&self, limit: usize) -> Vec<Topic> {
        let mut topics = self.topics.clone();
        topics.sort_by_key(|topic| Reverse(topic.activity_rank));
        topics.into_iter().take(limit).collect()
    }

    pub fn forum_snapshot(&self, forum_id: i32) -> Option<ForumSnapshot> {
        let forum = self.forum(forum_id)?;
        let topics = self.topics_for_forum(forum_id);
        let topic_count = topics.len();
        let post_count = topics
            .iter()
            .map(|topic| self.posts_for_topic(topic.id).len())
            .sum::<usize>();
        let latest_topic = topics.first()?.clone();
        let latest_post = self.posts_for_topic(latest_topic.id).last()?.clone();
        let latest_user = self.user(latest_post.author_id)?;

        Some(ForumSnapshot {
            forum,
            topic_count,
            post_count,
            last_topic_id: latest_topic.id,
            last_topic_subject: latest_topic.subject,
            last_post_author: latest_user.username,
            last_posted_at: latest_post.posted_at,
        })
    }

    pub fn search(&self, query: &str) -> SearchResults {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return SearchResults::default();
        }

        let topics = self
            .topics
            .iter()
            .filter(|topic| {
                topic.subject.to_lowercase().contains(&needle)
                    || topic
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&needle))
                    || self
                        .posts_for_topic(topic.id)
                        .iter()
                        .flat_map(|post| post.body.iter())
                        .any(|paragraph| paragraph.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect::<Vec<_>>();

        let users = self
            .users
            .iter()
            .filter(|user| {
                user.username.to_lowercase().contains(&needle)
                    || user.title.to_lowercase().contains(&needle)
                    || user.about.to_lowercase().contains(&needle)
                    || user.location.to_lowercase().contains(&needle)
            })
            .cloned()
            .collect::<Vec<_>>();

        SearchResults { topics, users }
    }
}

impl Topic {
    pub fn reply_count(&self, board: &AppData) -> usize {
        board.posts_for_topic(self.id).len().saturating_sub(1)
    }
}

impl TopicStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pinned => "Pinned",
            Self::Hot => "Hot",
            Self::Resolved => "Resolved",
            Self::Fresh => "Fresh",
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Pinned => "badge badge-pinned",
            Self::Hot => "badge badge-hot",
            Self::Resolved => "badge badge-resolved",
            Self::Fresh => "badge badge-fresh",
        }
    }
}

#[post("/api/board")]
pub async fn load_board() -> Result<AppData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        load_board_from_postgres().map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new(
            "Board loading requires the server feature.",
        ))
    }
}

#[post("/api/register")]
pub async fn register_account(input: RegisterForm) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "server")]
    {
        register_account_impl(input).map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new(
            "Registration requires the server feature.",
        ))
    }
}

#[post("/api/login")]
pub async fn login_account(input: LoginForm) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "server")]
    {
        login_account_impl(input).map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("Login requires the server feature."))
    }
}

#[post("/api/current-session", headers: HeaderMap)]
pub async fn current_session_user() -> Result<Option<SessionUser>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        current_session_user_impl(headers).map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(None)
    }
}

#[post("/api/logout", headers: HeaderMap)]
pub async fn logout_account() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        logout_account_impl(headers).map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(())
    }
}

#[cfg(feature = "server")]
fn register_account_impl(input: RegisterForm) -> Result<AuthResponse, String> {
    let username = normalize_username(&input.username);
    let email = input.email.trim().to_lowercase();
    let location = input.location.trim();
    let about = input.about.trim();

    validate_username(&username)?;
    validate_email(&email)?;

    if input.password.chars().count() < 9 {
        return Err("Password must be at least 9 characters long.".to_string());
    }

    if username_exists(&username)? {
        return Err("That username is already registered.".to_string());
    }

    if email_exists(&email)? {
        return Err("That email address is already in use.".to_string());
    }

    let salt = random_hex(16);
    let password_hash = hash_password(&input.password, &salt);

    let user = run_json_query::<SessionUser>(&format!(
        "WITH inserted AS (
             INSERT INTO users (
                 username, title, status, joined_at, post_count, location, about, last_seen,
                 email, password_hash, group_id, registered_at, last_visit, registration_ip
             )
             VALUES (
                 {username}, 'Member', 'Online',
                 to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD'),
                 0,
                 {location},
                 {about},
                 'just now',
                 {email},
                 {password_hash},
                 4,
                 EXTRACT(EPOCH FROM now())::bigint,
                 EXTRACT(EPOCH FROM now())::bigint,
                 '127.0.0.1'
             )
             RETURNING id, username, title
         )
         SELECT row_to_json(inserted) FROM inserted;",
        username = sql_literal(&username),
        location = sql_literal(location),
        about = sql_literal(about),
        email = sql_literal(&email),
        password_hash = sql_literal(&password_hash),
    ))?;

    let session_token = create_session(user.id)?;

    Ok(AuthResponse {
        user,
        session_token,
        message: "Registration complete. You are now signed in.".to_string(),
    })
}

#[cfg(feature = "server")]
fn login_account_impl(input: LoginForm) -> Result<AuthResponse, String> {
    let username = normalize_username(&input.username);
    if username.is_empty() || input.password.is_empty() {
        return Err("Username and password are required.".to_string());
    }

    let user = run_json_query::<Option<StoredAuthUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(user_row)
             FROM (
                 SELECT id, username, title, password_hash
                 FROM users
                 WHERE LOWER(username) = LOWER({username})
                 LIMIT 1
             ) AS user_row
         ), 'null'::json);",
        username = sql_literal(&username),
    ))?
    .ok_or_else(|| "Wrong username or password.".to_string())?;

    if user.password_hash.is_empty() || !verify_password(&input.password, &user.password_hash) {
        return Err("Wrong username or password.".to_string());
    }

    run_exec(&format!(
        "UPDATE users
         SET status = 'Online',
             last_seen = 'just now',
             last_visit = EXTRACT(EPOCH FROM now())::bigint
         WHERE id = {};",
        user.id
    ))?;

    let session_token = create_session(user.id)?;
    let message = if input.remember {
        "Signed in and this browser session was remembered.".to_string()
    } else {
        "Signed in successfully.".to_string()
    };

    Ok(AuthResponse {
        user: SessionUser {
            id: user.id,
            username: user.username,
            title: user.title,
        },
        session_token,
        message,
    })
}

#[cfg(feature = "server")]
fn current_session_user_impl(headers: HeaderMap) -> Result<Option<SessionUser>, String> {
    let Some(token) = parse_session_cookie(&headers) else {
        return Ok(None);
    };

    run_json_query::<Option<SessionUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(session_row)
             FROM (
                 SELECT u.id, u.username, u.title
                 FROM forum_sessions AS s
                 INNER JOIN users AS u ON u.id = s.user_id
                 WHERE s.token = {token}
                   AND s.expires_at > EXTRACT(EPOCH FROM now())::bigint
                 LIMIT 1
             ) AS session_row
         ), 'null'::json);",
        token = sql_literal(&token),
    ))
}

#[cfg(feature = "server")]
fn logout_account_impl(headers: HeaderMap) -> Result<(), String> {
    if let Some(token) = parse_session_cookie(&headers) {
        run_exec(&format!(
            "DELETE FROM forum_sessions WHERE token = {token};",
            token = sql_literal(&token)
        ))?;
    }

    Ok(())
}

#[cfg(feature = "server")]
fn load_board_from_postgres() -> Result<AppData, String> {
    let meta = run_json_query::<BoardMeta>(
        "SELECT row_to_json(meta_row)
         FROM (
             SELECT title, tagline, announcement_title, announcement_body
             FROM board_meta
             ORDER BY id
             LIMIT 1
         ) AS meta_row;",
    )?;

    let categories = run_json_query::<Vec<Category>>(
        "SELECT COALESCE(json_agg(row_to_json(category_row)), '[]'::json)
         FROM (
             SELECT id, name, description, sort_order
             FROM categories
             ORDER BY sort_order, id
         ) AS category_row;",
    )?;

    let forums = run_json_query::<Vec<Forum>>(
        "SELECT COALESCE(json_agg(row_to_json(forum_row)), '[]'::json)
         FROM (
             SELECT id, category_id, name, description, moderators, sort_order
             FROM forums
             ORDER BY category_id, sort_order, id
         ) AS forum_row;",
    )?;

    let users = run_json_query::<Vec<UserProfile>>(
        "SELECT COALESCE(json_agg(row_to_json(user_row)), '[]'::json)
         FROM (
             SELECT id, username, title, status, joined_at, post_count, location, about, last_seen
             FROM users
             ORDER BY id
         ) AS user_row;",
    )?;

    let topics = run_json_query::<Vec<Topic>>(
        "SELECT COALESCE(json_agg(row_to_json(topic_row)), '[]'::json)
         FROM (
             SELECT id, forum_id, author_id, subject, status, views, tags, created_at, updated_at, activity_rank
             FROM topics
             ORDER BY activity_rank DESC, id
         ) AS topic_row;",
    )?;

    let posts = run_json_query::<Vec<Post>>(
        "SELECT COALESCE(json_agg(row_to_json(post_row)), '[]'::json)
         FROM (
             SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position
             FROM posts
             ORDER BY topic_id, position, id
         ) AS post_row;",
    )?;

    Ok(AppData {
        meta,
        categories,
        forums,
        users,
        topics,
        posts,
    })
}

#[cfg(feature = "server")]
fn username_exists(username: &str) -> Result<bool, String> {
    let count = run_scalar_i64(&format!(
        "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER({username});",
        username = sql_literal(username)
    ))?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
fn email_exists(email: &str) -> Result<bool, String> {
    let count = run_scalar_i64(&format!(
        "SELECT COUNT(*) FROM users WHERE LOWER(email) = LOWER({email});",
        email = sql_literal(email)
    ))?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
fn create_session(user_id: i32) -> Result<String, String> {
    let token = random_hex(32);
    let now = unix_now();
    let expires = now + SESSION_MAX_AGE_SECS;

    run_exec(&format!(
        "INSERT INTO forum_sessions (token, user_id, created_at, expires_at, last_seen)
         VALUES ({token}, {user_id}, {now}, {expires}, {now});",
        token = sql_literal(&token),
        user_id = user_id,
        now = now,
        expires = expires,
    ))?;

    Ok(token)
}

#[cfg(feature = "server")]
fn parse_session_cookie(headers: &HeaderMap) -> Option<String> {
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
fn validate_username(username: &str) -> Result<(), String> {
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
fn validate_email(email: &str) -> Result<(), String> {
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
fn normalize_username(username: &str) -> String {
    username.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(feature = "server")]
fn hash_password(password: &str, salt: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(salt.as_bytes());
    digest.update(password.as_bytes());
    let bytes = digest.finalize();
    format!("sha256${salt}${}", bytes_to_hex(&bytes))
}

#[cfg(feature = "server")]
fn verify_password(password: &str, stored_hash: &str) -> bool {
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
fn random_hex(size: usize) -> String {
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
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(feature = "server")]
fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(feature = "server")]
fn server_error(message: String) -> ServerFnError {
    ServerFnError::new(message)
}

#[cfg(feature = "server")]
fn run_json_query<T>(sql: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let output = Command::new("psql")
        .arg(DATABASE_URL)
        .args(["-X", "-v", "ON_ERROR_STOP=1", "-t", "-A", "-c", sql])
        .output()
        .map_err(|error| format!("failed to run psql: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("psql returned non-utf8 output: {error}"))?;
    let payload = stdout.trim();

    serde_json::from_str(payload).map_err(|error| format!("failed to parse postgres json: {error}"))
}

#[cfg(feature = "server")]
fn run_scalar_i64(sql: &str) -> Result<i64, String> {
    let output = Command::new("psql")
        .arg(DATABASE_URL)
        .args(["-X", "-v", "ON_ERROR_STOP=1", "-t", "-A", "-c", sql])
        .output()
        .map_err(|error| format!("failed to run psql: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("psql returned non-utf8 output: {error}"))?;

    stdout
        .trim()
        .parse::<i64>()
        .map_err(|error| format!("failed to parse scalar result: {error}"))
}

#[cfg(feature = "server")]
fn run_exec(sql: &str) -> Result<(), String> {
    let output = Command::new("psql")
        .arg(DATABASE_URL)
        .args(["-X", "-v", "ON_ERROR_STOP=1", "-c", sql])
        .output()
        .map_err(|error| format!("failed to run psql: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

pub fn cookie_name() -> &'static str {
    SESSION_COOKIE
}

pub fn cookie_max_age() -> i64 {
    SESSION_MAX_AGE_SECS
}
