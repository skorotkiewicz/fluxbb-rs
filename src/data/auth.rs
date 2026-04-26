use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use super::{
    db::{
        run_exec, run_json_query, run_parameterized_exec, run_parameterized_json,
        run_parameterized_scalar_i64, run_scalar_i64, server_error, PgBind,
    },
    security::{
        check_ban, hash_password, normalize_username, parse_session_cookie, random_hex, unix_now,
        validate_email, validate_username, verify_password,
    },
};
use super::{
    AuthResponse, InstallForm, LoginForm, RegisterForm, RequestPasswordResetForm,
    ResetPasswordForm, SessionUser,
};

#[cfg(feature = "server")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredAuthUser {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub group_id: i32,
    pub email: String,
    pub password_hash: String,
    pub timezone: String,
    pub disp_topics: i32,
    pub disp_posts: i32,
    pub show_online: bool,
    pub theme: String,
    pub post_topics: bool,
    pub post_replies: bool,
    pub edit_posts: bool,
    pub delete_posts: bool,
    pub delete_topic: bool,
    pub move_topic: bool,
    pub sticky_topic: bool,
    pub close_topic: bool,
    pub manage_users: bool,
    pub manage_forums: bool,
    pub manage_categories: bool,
    pub manage_bans: bool,
    pub manage_groups: bool,
    pub manage_settings: bool,
    pub is_moderator: bool,
    pub is_admin: bool,
}

#[post("/api/register")]
pub async fn register_account(input: RegisterForm) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "server")]
    {
        register_account_impl(input).await.map_err(server_error)
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
        login_account_impl(input).await.map_err(server_error)
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
        current_session_user_impl(headers)
            .await
            .map_err(server_error)
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
        logout_account_impl(headers).await.map_err(server_error)
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(())
    }
}

#[post("/api/check-installed")]
pub async fn check_installed() -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        check_installed_impl().await.map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(true)
    }
}

#[post("/api/install")]
pub async fn install_board(input: InstallForm) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "server")]
    {
        install_board_impl(input).await.map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("Install requires the server feature."))
    }
}

#[post("/api/forgot-password")]
pub async fn request_password_reset(
    input: RequestPasswordResetForm,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email = input.email.trim().to_lowercase();
        if email.is_empty() || !email.contains('@') {
            return Err(server_error("Enter a valid email address.".into()));
        }

        let smtp_config = run_json_query::<Option<serde_json::Value>>(
            "SELECT COALESCE((SELECT row_to_json(m) FROM (SELECT smtp_enable, smtp_host, smtp_port, smtp_user, smtp_pass, smtp_from_email, smtp_from_name FROM board_meta LIMIT 1) m), 'null'::json);"
        ).await.map_err(server_error)?;

        let config = match smtp_config {
            Some(c) => c,
            None => return Err(server_error("Password reset is not enabled.".into())),
        };

        let enabled = config
            .get("smtp_enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !enabled {
            return Err(server_error("Password reset is not enabled.".into()));
        }

        let user = run_parameterized_json::<Option<SessionUser>>(
            "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT id, username, title, group_id FROM users WHERE LOWER(email) = LOWER($1) LIMIT 1) r), 'null'::json);",
            &[&email as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if let Some(user) = user {
            let host = config
                .get("smtp_host")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let from_email = config
                .get("smtp_from_email")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if host.is_empty() || from_email.is_empty() {
                return Err(server_error(
                    "Password reset is not properly configured.".into(),
                ));
            }

            run_parameterized_exec(
                "DELETE FROM password_resets WHERE user_id = $1;",
                &[&user.id as &(dyn PgBind + Sync)],
            )
            .await
            .ok();

            let token = random_hex(32);
            let now = unix_now();
            let expires = now + 86400;

            run_parameterized_exec(
                "INSERT INTO password_resets (user_id, token, created_at, expires_at) VALUES ($1, $2, $3, $4);",
                &[&user.id as &(dyn PgBind + Sync), &token, &now, &expires],
            )
            .await
            .map_err(server_error)?;

            let port = config
                .get("smtp_port")
                .and_then(|v| v.as_i64())
                .unwrap_or(587) as u16;
            let user_smtp = config
                .get("smtp_user")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pass = config
                .get("smtp_pass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let from_name = config
                .get("smtp_from_name")
                .and_then(|v| v.as_str())
                .unwrap_or("FluxBB")
                .to_string();

            let email_result = send_reset_email(
                host,
                port,
                &user_smtp,
                &pass,
                from_email,
                &from_name,
                &email,
                &user.username,
                &token,
            )
            .await;

            if email_result.is_ok() {
                return Ok("A password reset link has been sent to your email address.".to_string());
            } else {
                return Err(server_error(
                    "Failed to send reset email. Please try again later.".into(),
                ));
            }
        }

        Ok(
            "If an account with that email exists, a password reset link has been sent."
                .to_string(),
        )
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/reset-password")]
pub async fn reset_password(input: ResetPasswordForm) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        if input.password.chars().count() < 9 {
            return Err(server_error(
                "Password must be at least 9 characters.".into(),
            ));
        }

        let now = unix_now();
        let user_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT user_id FROM password_resets WHERE token = $1 AND expires_at > $2 LIMIT 1), 0);",
            &[&input.token as &(dyn PgBind + Sync), &now],
        )
        .await
        .map_err(server_error)?;

        if user_id == 0 {
            return Err(server_error(
                "Invalid or expired reset token. Please request a new password reset.".into(),
            ));
        }

        let hash = hash_password(&input.password);
        run_parameterized_exec(
            "UPDATE users SET password_hash = $1 WHERE id = $2;",
            &[&hash as &(dyn PgBind + Sync), &user_id],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "DELETE FROM password_resets WHERE token = $1;",
            &[&input.token as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(
            "Password updated successfully. You can now sign in with your new password."
                .to_string(),
        )
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "server")]
async fn register_account_impl(input: RegisterForm) -> Result<AuthResponse, String> {
    let username = normalize_username(&input.username);
    let email = input.email.trim().to_lowercase();
    let location = input.location.trim().to_string();
    let about = input.about.trim().to_string();

    validate_username(&username)?;
    validate_email(&email)?;

    if input.password.chars().count() < 9 {
        return Err("Password must be at least 9 characters long.".to_string());
    }

    if username_exists(&username).await? {
        return Err("That username is already registered.".to_string());
    }

    if email_exists(&email).await? {
        return Err("That email address is already in use.".to_string());
    }

    let password_hash = hash_password(&input.password);

    let mut user = run_parameterized_json::<SessionUser>(
        "WITH inserted AS (
             INSERT INTO users (
                 username, title, status, joined_at, post_count, location, about, last_seen,
                 email, password_hash, group_id, registered_at, last_visit, registration_ip,
                 timezone, disp_topics, disp_posts, show_online, theme
             )
             VALUES (
                 $1, 'Member', 'Online',
                 to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD'),
                 0,
                 $2,
                 $3,
                 'just now',
                 $4,
                 $5,
                 4,
                 EXTRACT(EPOCH FROM now())::bigint,
                 EXTRACT(EPOCH FROM now())::bigint,
                 '127.0.0.1',
                 'UTC', 25, 20, true, 'light'
             )
             RETURNING id, username, title, group_id, timezone, disp_topics, disp_posts, show_online, theme,
                       true AS post_topics, true AS post_replies, true AS edit_posts, false AS delete_posts,
                       false AS delete_topic, false AS move_topic, false AS sticky_topic, false AS close_topic,
                       false AS manage_users, false AS manage_forums, false AS manage_categories,
                       false AS manage_bans, false AS manage_groups, false AS manage_settings,
                       false AS is_moderator, false AS is_admin
         )
         SELECT row_to_json(inserted) FROM inserted;",
        &[
            &username as &(dyn PgBind + Sync),
            &location,
            &about,
            &email,
            &password_hash,
        ],
    )
    .await?;

    let (session_token, csrf_token) = create_session(user.id).await?;
    user.csrf_token = csrf_token;

    Ok(AuthResponse {
        user,
        session_token,
        message: "Registration complete. You are now signed in.".to_string(),
    })
}

#[cfg(feature = "server")]
async fn login_account_impl(input: LoginForm) -> Result<AuthResponse, String> {
    let username = normalize_username(&input.username);
    if username.is_empty() || input.password.is_empty() {
        return Err("Username and password are required.".to_string());
    }

    let user = run_parameterized_json::<Option<StoredAuthUser>>(
        "SELECT COALESCE((
             SELECT row_to_json(user_row)
             FROM (
                 SELECT u.id, u.username, u.title, u.group_id, u.email, u.password_hash, u.timezone, u.disp_topics, u.disp_posts, u.show_online, u.theme,
                        g.post_topics, g.post_replies, g.edit_posts, g.delete_posts,
                        g.delete_topic, g.move_topic, g.sticky_topic, g.close_topic,
                        g.manage_users, g.manage_forums, g.manage_categories, g.manage_bans,
                        g.manage_groups, g.manage_settings, g.is_moderator, g.is_admin
                 FROM users u
                 JOIN groups g ON g.id = u.group_id
                 WHERE LOWER(u.username) = LOWER($1)
                 LIMIT 1
             ) AS user_row
         ), 'null'::json);",
        &[&username as &(dyn PgBind + Sync)],
    )
    .await?
    .ok_or_else(|| "Wrong username or password.".to_string())?;

    if user.password_hash.is_empty() || !verify_password(&input.password, &user.password_hash) {
        return Err("Wrong username or password.".to_string());
    }

    if let Some(msg) = check_ban(&user.username, &user.email).await? {
        return Err(format!("Your account has been banned. Reason: {msg}"));
    }

    run_exec(&format!(
        "UPDATE users
         SET status = 'Online',
             last_seen = 'just now'
         WHERE id = {};",
        user.id
    ))
    .await?;

    let (session_token, csrf_token) = create_session(user.id).await?;

    Ok(AuthResponse {
        user: SessionUser {
            id: user.id,
            username: user.username,
            email: user.email,
            title: user.title,
            group_id: user.group_id,
            csrf_token,
            timezone: user.timezone,
            disp_topics: user.disp_topics,
            disp_posts: user.disp_posts,
            show_online: user.show_online,
            theme: user.theme,
            post_topics: user.post_topics,
            post_replies: user.post_replies,
            edit_posts: user.edit_posts,
            delete_posts: user.delete_posts,
            delete_topic: user.delete_topic,
            move_topic: user.move_topic,
            sticky_topic: user.sticky_topic,
            close_topic: user.close_topic,
            manage_users: user.manage_users,
            manage_forums: user.manage_forums,
            manage_categories: user.manage_categories,
            manage_bans: user.manage_bans,
            manage_groups: user.manage_groups,
            manage_settings: user.manage_settings,
            is_moderator: user.is_moderator,
            is_admin: user.is_admin,
        },
        session_token,
        message: "Signed in successfully.".to_string(),
    })
}

#[cfg(feature = "server")]
async fn current_session_user_impl(headers: HeaderMap) -> Result<Option<SessionUser>, String> {
    let Some(token) = parse_session_cookie(&headers) else {
        return Ok(None);
    };

    let now = unix_now();
    let _ = run_parameterized_exec(
        "UPDATE forum_sessions SET last_seen = $1 WHERE token = $2 AND expires_at > $1;",
        &[&now as &(dyn PgBind + Sync), &token],
    )
    .await;

    run_parameterized_json::<Option<SessionUser>>(
        "SELECT COALESCE((
             SELECT row_to_json(session_row)
             FROM (
                   SELECT u.id, u.username, u.email, u.title, u.group_id, s.csrf_token, u.timezone, u.disp_topics, u.disp_posts, u.show_online, u.theme,
                          g.post_topics, g.post_replies, g.edit_posts, g.delete_posts,
                          g.delete_topic, g.move_topic, g.sticky_topic, g.close_topic,
                          g.manage_users, g.manage_forums, g.manage_categories, g.manage_bans,
                          g.manage_groups, g.manage_settings, g.is_moderator, g.is_admin
                   FROM forum_sessions AS s
                   INNER JOIN users AS u ON u.id = s.user_id
                   INNER JOIN groups AS g ON g.id = u.group_id
                   WHERE s.token = $1
                    AND s.expires_at > EXTRACT(EPOCH FROM now())::bigint
                   LIMIT 1
             ) AS session_row
         ), 'null'::json);",
        &[&token as &(dyn PgBind + Sync)],
    )
    .await
}

#[cfg(feature = "server")]
async fn logout_account_impl(headers: HeaderMap) -> Result<(), String> {
    if let Some(token) = parse_session_cookie(&headers) {
        run_parameterized_exec(
            "DELETE FROM forum_sessions WHERE token = $1;",
            &[&token as &(dyn PgBind + Sync)],
        )
        .await?;
    }

    Ok(())
}

#[cfg(feature = "server")]
async fn check_installed_impl() -> Result<bool, String> {
    let count = run_scalar_i64(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'board_meta' AND table_schema = 'public';",
    )
    .await?;
    if count == 0 {
        return Ok(false);
    }
    let rows = run_scalar_i64("SELECT COUNT(*) FROM board_meta;").await?;
    Ok(rows > 0)
}

#[cfg(feature = "server")]
async fn install_board_impl(input: InstallForm) -> Result<AuthResponse, String> {
    let title = input.board_title.trim().to_string();
    let tagline = input.board_tagline.trim().to_string();
    let username = normalize_username(&input.admin_username);
    let email = input.admin_email.trim().to_lowercase();

    if title.is_empty() {
        return Err("Board title is required.".to_string());
    }
    validate_username(&username)?;
    validate_email(&email)?;
    if input.admin_password.chars().count() < 9 {
        return Err("Password must be at least 9 characters.".to_string());
    }

    let db_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        input.db_user.trim(),
        input.db_password,
        input.db_host.trim(),
        input.db_port.trim(),
        input.db_name.trim(),
    );
    std::env::set_var("DATABASE_URL", &db_url);
    let env_content = format!("DATABASE_URL={db_url}\n");
    if let Err(e) = std::fs::write(".env", env_content) {
        return Err(format!("Failed to write .env file: {e}"));
    }

    let schema_path = std::path::Path::new("db/schema.sql");
    if schema_path.exists() {
        let sql = std::fs::read_to_string(schema_path)
            .map_err(|e| format!("failed to read schema.sql: {e}"))?;
        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if !stmt.is_empty() {
                run_exec(&format!("{stmt};")).await?;
            }
        }
    } else {
        return Err("db/schema.sql not found.".to_string());
    }

    run_exec(
        "INSERT INTO groups (id, title, read_board, post_topics, post_replies, edit_posts, delete_posts,
                            delete_topic, move_topic, sticky_topic, close_topic,
                            manage_users, manage_forums, manage_categories, manage_bans, manage_groups, manage_settings,
                            is_moderator, is_admin)
         VALUES
             (1, 'Administrators', true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true),
             (2, 'Moderators', true, true, true, true, true, true, true, true, true, false, false, false, true, false, false, true, false),
             (3, 'Guests', true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false),
             (4, 'Members', true, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false)
         ON CONFLICT (id) DO NOTHING;",
    )
    .await?;

    run_parameterized_exec(
        "INSERT INTO board_meta (title, tagline) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title, tagline = EXCLUDED.tagline;",
        &[&title as &(dyn PgBind + Sync), &tagline],
    )
    .await?;

    let password_hash = hash_password(&input.admin_password);
    let mut user = run_parameterized_json::<SessionUser>(
        "WITH inserted AS (
             INSERT INTO users (
                 username, title, status, joined_at, post_count, location, about, last_seen,
                 email, password_hash, group_id, registered_at, last_visit, registration_ip,
                 timezone, disp_topics, disp_posts, show_online, theme
             )
             VALUES (
                 $1, 'Administrator', 'Online',
                 to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD'),
                 0,
                 '',
                 '',
                 'just now',
                 $2,
                 $3,
                 1,
                 EXTRACT(EPOCH FROM now())::bigint,
                 EXTRACT(EPOCH FROM now())::bigint,
                 '127.0.0.1',
                 'UTC', 25, 20, true, 'light'
             )
             ON CONFLICT DO NOTHING
             RETURNING id, username, title, group_id, timezone, disp_topics, disp_posts, show_online, theme,
                       true AS post_topics, true AS post_replies, true AS edit_posts, true AS delete_posts,
                       true AS delete_topic, true AS move_topic, true AS sticky_topic, true AS close_topic,
                       true AS manage_users, true AS manage_forums, true AS manage_categories,
                       true AS manage_bans, true AS manage_groups, true AS manage_settings,
                       true AS is_moderator, true AS is_admin
         )
         SELECT row_to_json(inserted) FROM inserted;",
        &[
            &username as &(dyn PgBind + Sync),
            &email,
            &password_hash,
        ],
    )
    .await?;

    run_exec(
        "INSERT INTO categories (name, description, sort_order) VALUES ('General', 'Main discussion area.', 1);",
    )
    .await?;

    run_parameterized_exec(
        "INSERT INTO forums (category_id, name, description, moderators, sort_order) VALUES (1, 'General Discussion', 'Talk about anything.', ARRAY[$1], 1);",
        &[&username as &(dyn PgBind + Sync)],
    )
    .await?;

    let (session_token, csrf_token) = create_session(user.id).await?;
    user.csrf_token = csrf_token;
    Ok(AuthResponse {
        user,
        session_token,
        message: "Board installed. You are signed in as administrator.".to_string(),
    })
}

#[cfg(feature = "server")]
async fn username_exists(username: &str) -> Result<bool, String> {
    let username = username.to_string();
    let count = run_parameterized_scalar_i64(
        "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER($1);",
        &[&username as &(dyn PgBind + Sync)],
    )
    .await?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
async fn email_exists(email: &str) -> Result<bool, String> {
    let email = email.to_string();
    let count = run_parameterized_scalar_i64(
        "SELECT COUNT(*) FROM users WHERE LOWER(email) = LOWER($1);",
        &[&email as &(dyn PgBind + Sync)],
    )
    .await?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
async fn create_session(user_id: i32) -> Result<(String, String), String> {
    let token = random_hex(32);
    let csrf = random_hex(16);
    let now = unix_now();
    let expires = now + super::cookie_max_age();

    run_parameterized_exec(
        "INSERT INTO forum_sessions (token, user_id, created_at, expires_at, last_seen, csrf_token)
         VALUES ($1, $2, $3, $4, $3, $5);",
        &[&token as &(dyn PgBind + Sync), &user_id, &now, &expires, &csrf],
    )
    .await?;

    Ok((token, csrf))
}

#[cfg(feature = "server")]
async fn send_reset_email(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    from_email: &str,
    from_name: &str,
    to_email: &str,
    to_username: &str,
    token: &str,
) -> Result<(), String> {
    use lettre::{
        message::header::ContentType, transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };

    let from = format!("{} <{}>", from_name, from_email);
    let to = format!("{} <{}>", to_username, to_email);

    let email = Message::builder()
        .from(from.parse().map_err(|e| format!("Invalid from address: {e}"))?)
        .to(to.parse().map_err(|e| format!("Invalid to address: {e}"))?)
        .subject("Password reset request")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Hello {},\n\nYou have requested a password reset for your account.\n\nClick the link below to reset your password:\n\n/reset-password?token={}\n\nIf you did not request this, please ignore this email. The link will expire in 24 hours.\n\nRegards,\n{}",
            to_username, token, from_name
        ))
        .map_err(|e| format!("Failed to build email: {e}"))?;

    let creds = Credentials::new(username.to_string(), password.to_string());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
        .map_err(|e| format!("Failed to create mailer: {e}"))?
        .port(port)
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| format!("Failed to send email: {e}"))?;

    Ok(())
}
