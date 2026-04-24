#[cfg(feature = "server")]
use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use rand::RngCore;
#[cfg(feature = "server")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sha2::{Digest, Sha256};

#[cfg(feature = "server")]
fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set. Create a .env file or set the environment variable.")
}
const SESSION_COOKIE: &str = "fluxbb_rs_session";
const CSRF_COOKIE: &str = "fluxbb_rs_csrf";
const SESSION_MAX_AGE_SECS: i64 = 60 * 60 * 24 * 14;

#[cfg(feature = "server")]
use sqlx::Row;
#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

#[cfg(feature = "server")]
async fn db_pool() -> Result<&'static sqlx::PgPool, String> {
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool);
    }
    let url = database_url();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(|e| format!("DB connection failed: {e}"))?;
    let _ = DB_POOL.set(pool);
    Ok(DB_POOL.get().unwrap())
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
    #[serde(default)]
    pub email: String,
    #[serde(default = "default_group_id")]
    pub group_id: i32,
}

fn default_group_id() -> i32 {
    4
}

impl UserProfile {
    pub fn group_id(&self) -> i32 {
        self.group_id
    }
    pub fn email_display(&self) -> &str {
        if self.email.is_empty() {
            "no email"
        } else {
            &self.email
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub id: i32,
    pub forum_id: i32,
    pub author_id: i32,
    pub subject: String,
    #[serde(default)]
    pub closed: bool,
    pub views: i32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub activity_rank: i32,
    #[serde(default)]
    pub reply_count: i32,
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub moved_to: i32,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardStats {
    pub members: usize,
    pub topics: usize,
    pub posts: usize,
    pub newest_member: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShellData {
    pub meta: BoardMeta,
    pub stats: BoardStats,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForumStats {
    pub forum_id: i32,
    pub topic_count: usize,
    pub post_count: usize,
    pub last_topic_id: i32,
    pub last_topic_subject: String,
    pub last_post_author: String,
    pub last_posted_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub forum_stats: Vec<ForumStats>,
    pub stats: BoardStats,
    pub recent_topics: Vec<Topic>,
    pub recent_users: Vec<UserProfile>,
    #[serde(default)]
    pub last_visit: i64,
}

#[allow(unused)]
const FORUM_TOPICS_PER_PAGE: i32 = 25;
#[allow(unused)]
const TOPIC_POSTS_PER_PAGE: i32 = 20;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForumData {
    pub forum: Forum,
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
    pub total_topics: i32,
    pub page: i32,
    pub per_page: i32,
    #[serde(default)]
    pub last_visit: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TopicData {
    pub topic: Topic,
    pub posts: Vec<Post>,
    pub users: Vec<UserProfile>,
    pub forum: Option<Forum>,
    pub total_posts: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileData {
    pub user: UserProfile,
    pub topics: Vec<Topic>,
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub users: Vec<UserProfile>,
    pub topics: Vec<Topic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SearchResults {
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: i32,
    pub username: String,
    #[serde(default)]
    pub email: String,
    pub title: String,
    pub group_id: i32,
    #[serde(default)]
    pub csrf_token: String,
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
pub struct InstallForm {
    pub board_title: String,
    pub board_tagline: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: String,
    pub db_host: String,
    pub db_port: String,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewTopicForm {
    pub forum_id: i32,
    pub subject: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplyForm {
    pub topic_id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewTopicResult {
    pub topic_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditPostForm {
    pub post_id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateProfileForm {
    pub user_id: i32,
    pub email: String,
    pub location: String,
    pub about: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub user_id: i32,
    pub password: String,
}

// ── Admin forms ──

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminCategoryForm {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminForumForm {
    pub category_id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminUserUpdate {
    pub user_id: i32,
    pub group_id: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminTopicUpdate {
    pub topic_id: i32,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminBoardSettings {
    pub title: String,
    pub tagline: String,
    pub announcement_title: String,
    pub announcement_body: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminDeleteItem {
    pub id: i32,
}

#[cfg(feature = "server")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct StoredAuthUser {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub group_id: i32,
    pub email: String,
    pub password_hash: String,
}

// ── View-specific loaders ──

#[post("/api/shell-data")]
pub async fn load_shell_data() -> Result<ShellData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let data = run_json_query::<ShellData>(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body FROM board_meta LIMIT 1) m),
                'stats', json_build_object(
                    'members', (SELECT COUNT(*)::int FROM users),
                    'topics', (SELECT COUNT(*)::int FROM topics),
                    'posts', (SELECT COUNT(*)::int FROM posts),
                    'newest_member', COALESCE((SELECT username FROM users ORDER BY id DESC LIMIT 1), '')
                )
            )::json;",
        ).await.map_err(server_error)?;
        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/index-data", headers: HeaderMap)]
pub async fn load_index_data() -> Result<IndexData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let token = parse_session_cookie(&headers);
        let user_id = if let Some(token) = token {
            run_scalar_i64(&format!(
                "SELECT COALESCE((SELECT user_id FROM forum_sessions WHERE token = {} AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1), 0);",
                sql_literal(&token)
            )).await.unwrap_or(0)
        } else {
            0
        };
        let data = run_json_query::<IndexData>(&format!(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body FROM board_meta LIMIT 1) m),
                'categories', (SELECT COALESCE(json_agg(row_to_json(c)), '[]'::json) FROM (SELECT id, name, description, sort_order FROM categories ORDER BY sort_order, id) c),
                'forums', (SELECT COALESCE(json_agg(row_to_json(f)), '[]'::json) FROM (SELECT id, category_id, name, description, moderators, sort_order FROM forums ORDER BY category_id, sort_order, id) f),
                'forum_stats', (SELECT COALESCE(json_agg(row_to_json(fa)), '[]'::json) FROM (
                    SELECT
                        f.id AS forum_id,
                        (SELECT COUNT(*)::int FROM topics WHERE forum_id = f.id) AS topic_count,
                        COALESCE((SELECT COUNT(*)::int FROM posts WHERE topic_id IN (SELECT id FROM topics WHERE forum_id = f.id)), 0) AS post_count,
                        COALESCE((SELECT id FROM topics WHERE forum_id = f.id ORDER BY sticky DESC, activity_rank DESC, id LIMIT 1), 0) AS last_topic_id,
                        COALESCE((SELECT subject FROM topics WHERE forum_id = f.id ORDER BY sticky DESC, activity_rank DESC, id LIMIT 1), '') AS last_topic_subject,
                        COALESCE((SELECT u.username FROM posts p JOIN users u ON u.id = p.author_id WHERE p.topic_id = (SELECT id FROM topics WHERE forum_id = f.id ORDER BY sticky DESC, activity_rank DESC, id LIMIT 1) ORDER BY p.position DESC, p.id DESC LIMIT 1), '') AS last_post_author,
                        COALESCE((SELECT p.posted_at FROM posts p WHERE p.topic_id = (SELECT id FROM topics WHERE forum_id = f.id ORDER BY sticky DESC, activity_rank DESC, id LIMIT 1) ORDER BY p.position DESC, p.id DESC LIMIT 1), '') AS last_posted_at
                    FROM forums f
                ) fa),
                'stats', json_build_object(
                    'members', (SELECT COUNT(*)::int FROM users),
                    'topics', (SELECT COUNT(*)::int FROM topics),
                    'posts', (SELECT COUNT(*)::int FROM posts),
                    'newest_member', COALESCE((SELECT username FROM users ORDER BY id DESC LIMIT 1), '')
                ),
                'recent_topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                    FROM topics ORDER BY activity_rank DESC, id LIMIT 4
                ) t),
                'recent_users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                    FROM users WHERE id IN (SELECT author_id FROM topics ORDER BY activity_rank DESC, id LIMIT 4)
                ) u),
                'last_visit', COALESCE((SELECT last_visit FROM users WHERE id = {user_id}), 0)
            )::json;",
            user_id = user_id,
        )).await.map_err(server_error)?;
        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/forums")]
pub async fn load_forums() -> Result<Vec<Forum>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let forums = run_json_query::<Vec<Forum>>(
            "SELECT COALESCE(json_agg(row_to_json(f)), '[]'::json) FROM (SELECT id, category_id, name, description, moderators, sort_order FROM forums ORDER BY category_id, sort_order, id) f;",
        ).await.map_err(server_error)?;
        Ok(forums)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/forum/:id/:page", headers: HeaderMap)]
pub async fn load_forum_data(id: i32, page: i32) -> Result<ForumData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let token = parse_session_cookie(&headers);
        let user_id = if let Some(token) = token {
            run_scalar_i64(&format!(
                "SELECT COALESCE((SELECT user_id FROM forum_sessions WHERE token = {} AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1), 0);",
                sql_literal(&token)
            )).await.unwrap_or(0)
        } else {
            0
        };
        let page = page.max(1);
        let per_page = FORUM_TOPICS_PER_PAGE;
        let offset = (page - 1) * per_page;
        let data = run_json_query::<ForumData>(&format!(
            "SELECT json_build_object(
                'forum', (SELECT row_to_json(f) FROM forums f WHERE f.id = {id}),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to,
                        COALESCE((SELECT COUNT(*) FROM posts WHERE topic_id = t.id), 0) - 1 AS reply_count
                    FROM topics t WHERE forum_id = {id} ORDER BY sticky DESC, activity_rank DESC, id
                    LIMIT {per_page} OFFSET {offset}
                ) t),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                    FROM users WHERE id IN (SELECT author_id FROM topics WHERE forum_id = {id})
                ) u),
                'total_topics', (SELECT COUNT(*) FROM topics WHERE forum_id = {id}),
                'page', {page},
                'per_page', {per_page},
                'last_visit', COALESCE((SELECT last_visit FROM users WHERE id = {user_id}), 0)
            )::json;",
            id = id,
            page = page,
            per_page = per_page,
            offset = offset,
            user_id = user_id,
        )).await.map_err(server_error)?;
        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        let _ = page;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/topic/:id/:page", headers: HeaderMap)]
pub async fn load_topic_data(id: i32, page: i32) -> Result<TopicData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let page = page.max(1);
        let per_page = TOPIC_POSTS_PER_PAGE;
        let offset = (page - 1) * per_page;
        let data = run_json_query::<TopicData>(&format!(
            "SELECT json_build_object(
                'topic', (SELECT row_to_json(t) FROM topics t WHERE t.id = {id}),
                'posts', (SELECT COALESCE(json_agg(row_to_json(p)), '[]'::json) FROM (
                    SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position
                    FROM posts WHERE topic_id = {id} ORDER BY position, id
                    LIMIT {per_page} OFFSET {offset}
                ) p),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                    FROM users WHERE id IN (SELECT author_id FROM posts WHERE topic_id = {id})
                ) u),
                'forum', (SELECT row_to_json(f) FROM forums f WHERE f.id = (SELECT forum_id FROM topics WHERE id = {id})),
                'total_posts', (SELECT COUNT(*) FROM posts WHERE topic_id = {id}),
                'page', {page},
                'per_page', {per_page}
            )::json;",
            id = id,
            page = page,
            per_page = per_page,
            offset = offset
        )).await.map_err(server_error)?;

        // Mark topic as read for logged-in user
        if let Some(token) = parse_session_cookie(&headers) {
            let _ = run_exec(&format!(
                "UPDATE users SET last_visit = EXTRACT(EPOCH FROM now())::bigint WHERE id = (SELECT user_id FROM forum_sessions WHERE token = {} AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1);",
                sql_literal(&token)
            )).await;
        }

        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        let _ = page;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/profile/:id")]
pub async fn load_profile_data(id: i32) -> Result<ProfileData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let data = run_json_query::<ProfileData>(&format!(
            "SELECT json_build_object(
                'user', (SELECT row_to_json(u) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                    FROM users WHERE id = {id}
                ) u),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                    FROM topics WHERE author_id = {id} ORDER BY activity_rank DESC, id LIMIT 10
                ) t),
                'posts', (SELECT COALESCE(json_agg(row_to_json(p)), '[]'::json) FROM (
                    SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position
                    FROM posts WHERE author_id = {id} ORDER BY posted_at DESC LIMIT 10
                ) p)
            )::json;",
            id = id
        )).await.map_err(server_error)?;
        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/users")]
pub async fn load_users_data() -> Result<Vec<UserProfile>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let users = run_json_query::<Vec<UserProfile>>(
            "SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                FROM users ORDER BY post_count DESC, id
            ) u;",
        ).await.map_err(server_error)?;
        Ok(users)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/search")]
pub async fn search_server(query: String) -> Result<SearchResults, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Ok(SearchResults::default());
        }
        let like = format!("%{}%", needle.replace('%', "\\%").replace('_', "\\_"));
        let results = run_json_query::<SearchResults>(&format!(
            "SELECT json_build_object(
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                    FROM topics
                    WHERE LOWER(subject) LIKE {}
                       OR EXISTS (SELECT 1 FROM unnest(tags) tag WHERE LOWER(tag) LIKE {})
                       OR id IN (SELECT DISTINCT topic_id FROM posts p WHERE EXISTS (
                           SELECT 1 FROM unnest(p.body) para WHERE LOWER(para) LIKE {}
                       ))
                    ORDER BY activity_rank DESC
                    LIMIT 20
                ) t),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id
                    FROM users
                    WHERE LOWER(username) LIKE {}
                       OR LOWER(title) LIKE {}
                       OR LOWER(about) LIKE {}
                       OR LOWER(location) LIKE {}
                    LIMIT 20
                ) u)
            )::json;",
            sql_literal(&like),
            sql_literal(&like),
            sql_literal(&like),
            sql_literal(&like),
            sql_literal(&like),
            sql_literal(&like),
            sql_literal(&like),
        )).await.map_err(server_error)?;
        Ok(results)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = query;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin-data", headers: HeaderMap)]
pub async fn load_admin_data() -> Result<AdminData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        let data = run_json_query::<AdminData>(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body FROM board_meta LIMIT 1) m),
                'categories', (SELECT COALESCE(json_agg(row_to_json(c)), '[]'::json) FROM (SELECT id, name, description, sort_order FROM categories ORDER BY sort_order, id) c),
                'forums', (SELECT COALESCE(json_agg(row_to_json(f)), '[]'::json) FROM (SELECT id, category_id, name, description, moderators, sort_order FROM forums ORDER BY category_id, sort_order, id) f),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id FROM users ORDER BY id) u),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to FROM topics ORDER BY sticky DESC, activity_rank DESC, id) t)
            )::json;",
        ).await.map_err(server_error)?;
        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
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

#[post("/api/new-topic", headers: HeaderMap)]
pub async fn create_topic(input: NewTopicForm) -> Result<NewTopicResult, ServerFnError> {
    #[cfg(feature = "server")]
    {
        create_topic_impl(input, headers)
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("Posting requires the server feature."))
    }
}

#[post("/api/reply", headers: HeaderMap)]
pub async fn create_reply(input: ReplyForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        create_reply_impl(input, headers)
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("Posting requires the server feature."))
    }
}

// ── Admin endpoints ──

#[post("/api/admin/add-category", headers: HeaderMap)]
pub async fn admin_add_category(input: AdminCategoryForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("INSERT INTO categories (name, description, sort_order) VALUES ({}, {}, (SELECT COALESCE(MAX(sort_order),0)+1 FROM categories));",
        sql_literal(input.name.trim()), sql_literal(input.description.trim()))).await.map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/add-forum", headers: HeaderMap)]
pub async fn admin_add_forum(input: AdminForumForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("INSERT INTO forums (category_id, name, description, sort_order) VALUES ({}, {}, {}, (SELECT COALESCE(MAX(sort_order),0)+1 FROM forums WHERE category_id={}));",
        input.category_id, sql_literal(input.name.trim()), sql_literal(input.description.trim()), input.category_id)).await.map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/delete-category", headers: HeaderMap)]
pub async fn admin_delete_category(input: AdminDeleteItem) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("DELETE FROM categories WHERE id = {};", input.id))
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/delete-forum", headers: HeaderMap)]
pub async fn admin_delete_forum(input: AdminDeleteItem) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("DELETE FROM forums WHERE id = {};", input.id))
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/update-user", headers: HeaderMap)]
pub async fn admin_update_user(input: AdminUserUpdate) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!(
            "UPDATE users SET group_id = {}, title = {} WHERE id = {};",
            input.group_id,
            sql_literal(input.title.trim()),
            input.user_id
        ))
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/delete-user", headers: HeaderMap)]
pub async fn admin_delete_user(input: AdminDeleteItem) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        if input.id == u.id {
            return Err(server_error("Cannot delete yourself.".into()));
        }
        run_exec(&format!(
            "DELETE FROM forum_sessions WHERE user_id = {};",
            input.id
        ))
        .await
        .map_err(server_error)?;
        run_exec(&format!("DELETE FROM users WHERE id = {};", input.id))
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/update-topic", headers: HeaderMap)]
pub async fn admin_update_topic(input: AdminTopicUpdate) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!(
            "UPDATE topics SET closed = {} WHERE id = {};",
            input.closed, input.topic_id
        ))
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/delete-topic", headers: HeaderMap)]
pub async fn admin_delete_topic(input: AdminDeleteItem) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("DELETE FROM topics WHERE id = {};", input.id))
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/update-board", headers: HeaderMap)]
pub async fn admin_update_board(input: AdminBoardSettings) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("UPDATE board_meta SET title = {}, tagline = {}, announcement_title = {}, announcement_body = {} WHERE id = 1;",
        sql_literal(input.title.trim()), sql_literal(input.tagline.trim()),
        sql_literal(input.announcement_title.trim()), sql_literal(input.announcement_body.trim()))).await.map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/clean-sessions", headers: HeaderMap)]
pub async fn admin_clean_sessions() -> Result<i64, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if u.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        let deleted = run_scalar_i64(
            "WITH deleted AS (DELETE FROM forum_sessions WHERE expires_at < EXTRACT(EPOCH FROM now())::bigint RETURNING *) SELECT COUNT(*) FROM deleted;"
        ).await.map_err(server_error)?;
        Ok(deleted)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

// ── Post editing ──

#[post("/api/post/:id", headers: HeaderMap)]
pub async fn load_post(id: i32) -> Result<Post, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let post = run_json_query::<Post>(&format!(
            "SELECT row_to_json(post_row) FROM (SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position FROM posts WHERE id = {}) AS post_row;",
            id
        )).await.map_err(server_error)?;
        Ok(post)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/edit-post", headers: HeaderMap)]
pub async fn edit_post(input: EditPostForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if let Some(msg) = check_ban(&user.username, &user.email)
            .await
            .map_err(server_error)?
        {
            return Err(server_error(format!("You are banned: {msg}")));
        }
        let post = run_json_query::<Option<Post>>(&format!(
            "SELECT COALESCE((SELECT row_to_json(post_row) FROM (SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position FROM posts WHERE id = {}) AS post_row), 'null'::json);",
            input.post_id
        )).await.map_err(server_error)?;

        let Some(post) = post else {
            return Err(server_error("Post not found.".into()));
        };

        let group = get_group(user.group_id).await.map_err(server_error)?;
        if post.author_id != user.id && !group.edit_posts && !group.is_admin {
            return Err(server_error("You can only edit your own posts.".into()));
        }

        let message = input.message.trim();
        if message.is_empty() {
            return Err(server_error("Message is required.".into()));
        }

        let now_str = "to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC')";
        run_exec(&format!(
            "UPDATE posts SET body = ARRAY[{msg}], edited_at = {now} WHERE id = {pid};",
            msg = sql_literal(message),
            now = now_str,
            pid = input.post_id,
        ))
        .await
        .map_err(server_error)?;

        // Update topic activity
        run_exec(&format!(
            "UPDATE topics SET updated_at = {now}, activity_rank = EXTRACT(EPOCH FROM now())::integer WHERE id = {tid};",
            now = now_str,
            tid = post.topic_id,
        )).await.map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/delete-post", headers: HeaderMap)]
pub async fn delete_post(post_id: i32) -> Result<i32, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if let Some(msg) = check_ban(&user.username, &user.email)
            .await
            .map_err(server_error)?
        {
            return Err(server_error(format!("You are banned: {msg}")));
        }
        let group = get_group(user.group_id).await.map_err(server_error)?;

        #[derive(Deserialize)]
        struct PostInfo {
            topic_id: i32,
            author_id: i32,
            is_first: bool,
        }

        let info = run_json_query::<PostInfo>(&format!(
            "SELECT row_to_json(r) FROM (
                SELECT p.topic_id, p.author_id,
                       CASE WHEN p.id = (SELECT MIN(id) FROM posts WHERE topic_id = p.topic_id) THEN true ELSE false END AS is_first
                FROM posts p
                WHERE p.id = {}
            ) r;",
            post_id
        )).await.map_err(server_error)?;

        if info.author_id != user.id && !group.delete_posts && !group.is_admin {
            return Err(server_error("You can only delete your own posts.".into()));
        }

        if info.is_first {
            #[derive(Deserialize)]
            struct PostCount {
                author_id: i32,
                cnt: i64,
            }
            // Count posts to subtract from user post counts
            let post_counts = run_json_query::<Vec<PostCount>>(&format!(
                "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT author_id, COUNT(*)::bigint AS cnt FROM posts WHERE topic_id = {} GROUP BY author_id) r;",
                info.topic_id
            )).await.map_err(server_error)?;
            for pc in post_counts {
                run_exec(&format!(
                    "UPDATE users SET post_count = GREATEST(post_count - {}, 0) WHERE id = {};",
                    pc.cnt, pc.author_id
                ))
                .await
                .map_err(server_error)?;
            }
            // Delete topic and all posts
            run_exec(&format!(
                "DELETE FROM posts WHERE topic_id = {};",
                info.topic_id
            ))
            .await
            .map_err(server_error)?;
            run_exec(&format!("DELETE FROM topics WHERE id = {};", info.topic_id))
                .await
                .map_err(server_error)?;
            Ok(info.topic_id)
        } else {
            run_exec(&format!(
                "UPDATE users SET post_count = GREATEST(post_count - 1, 0) WHERE id = {};",
                info.author_id
            ))
            .await
            .map_err(server_error)?;
            run_exec(&format!("DELETE FROM posts WHERE id = {};", post_id))
                .await
                .map_err(server_error)?;
            Ok(0)
        }
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = post_id;
        Err(ServerFnError::new("server only"))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupUpdateForm {
    pub group_id: i32,
    pub title: String,
    pub read_board: bool,
    pub post_topics: bool,
    pub post_replies: bool,
    pub edit_posts: bool,
    pub delete_posts: bool,
    pub is_moderator: bool,
    pub is_admin: bool,
}

#[post("/api/groups")]
pub async fn load_groups() -> Result<Vec<Group>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let groups = run_json_query::<Vec<Group>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT id, title, read_board, post_topics, post_replies, edit_posts, delete_posts, is_moderator, is_admin FROM groups ORDER BY id) r;",
        ).await.map_err(server_error)?;
        Ok(groups)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/update-group", headers: HeaderMap)]
pub async fn update_group(input: GroupUpdateForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!(
            "UPDATE groups SET title = {title}, read_board = {rb}, post_topics = {pt}, post_replies = {pr}, edit_posts = {ep}, delete_posts = {dp}, is_moderator = {im}, is_admin = {ia} WHERE id = {gid};",
            title = sql_literal(input.title.trim()),
            rb = input.read_board,
            pt = input.post_topics,
            pr = input.post_replies,
            ep = input.edit_posts,
            dp = input.delete_posts,
            im = input.is_moderator,
            ia = input.is_admin,
            gid = input.group_id,
        )).await.map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

// ── Ban system ──

#[post("/api/bans")]
pub async fn load_bans() -> Result<Vec<Ban>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let bans = run_json_query::<Vec<Ban>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT id, username, email, ip, message, created_at, expires_at FROM bans ORDER BY created_at DESC) r;",
        ).await.map_err(server_error)?;
        Ok(bans)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/add-ban", headers: HeaderMap)]
pub async fn add_ban(input: BanForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let expires = input.duration_days.map(|d| now + (d as i64) * 86400);
        let expires_sql = match expires {
            Some(e) => e.to_string(),
            None => "NULL".to_string(),
        };
        run_exec(&format!(
            "INSERT INTO bans (username, email, message, created_at, expires_at) VALUES ({username}, {email}, {message}, {created}, {expires});",
            username = sql_literal(input.username.trim()),
            email = sql_literal(&input.email.trim().to_lowercase()),
            message = sql_literal(input.message.trim()),
            created = now,
            expires = expires_sql,
        )).await.map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/remove-ban", headers: HeaderMap)]
pub async fn remove_ban(ban_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!("DELETE FROM bans WHERE id = {};", ban_id))
            .await
            .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = ban_id;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "server")]
async fn check_ban(username: &str, email: &str) -> Result<Option<String>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    #[derive(Deserialize)]
    struct BanRow {
        message: String,
    }
    let ban = run_json_query::<Option<BanRow>>(&format!(
        "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT message FROM bans WHERE ((username <> '' AND LOWER(username) = LOWER({u})) OR (email <> '' AND LOWER(email) = LOWER({e}))) AND (expires_at IS NULL OR expires_at > {now}) LIMIT 1) r), 'null'::json);",
        u = sql_literal(username),
        e = sql_literal(email),
        now = now,
    )).await?;
    Ok(ban.map(|b| b.message))
}

// ── Mark all as read ──

#[post("/api/mark-read", headers: HeaderMap)]
pub async fn mark_all_read() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        run_exec(&format!(
            "UPDATE users SET last_visit = EXTRACT(EPOCH FROM now())::bigint WHERE id = {};",
            user.id
        ))
        .await
        .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

// ── Profile editing ──

#[post("/api/update-profile", headers: HeaderMap)]
pub async fn update_profile(input: UpdateProfileForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if input.user_id != user.id && user.group_id != 1 {
            return Err(server_error("You can only edit your own profile.".into()));
        }

        let email = input.email.trim().to_lowercase();
        if email.is_empty() || !email.contains('@') {
            return Err(server_error("Enter a valid email address.".into()));
        }

        run_exec(&format!(
            "UPDATE users SET email = {}, location = {}, about = {} WHERE id = {};",
            sql_literal(&email),
            sql_literal(input.location.trim()),
            sql_literal(input.about.trim()),
            input.user_id,
        ))
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/change-password", headers: HeaderMap)]
pub async fn change_password(input: ChangePasswordForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if input.user_id != user.id && user.group_id != 1 {
            return Err(server_error(
                "You can only change your own password.".into(),
            ));
        }
        if input.password.len() < 9 {
            return Err(server_error(
                "Password must be at least 9 characters.".into(),
            ));
        }
        let salt = random_hex(16);
        let hash = hash_password(&input.password, &salt);
        run_exec(&format!(
            "UPDATE users SET password = {} WHERE id = {};",
            sql_literal(&hash),
            input.user_id,
        ))
        .await
        .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoveTopicForm {
    pub topic_id: i32,
    pub forum_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ban {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub ip: String,
    pub message: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub title: String,
    pub read_board: bool,
    pub post_topics: bool,
    pub post_replies: bool,
    pub edit_posts: bool,
    pub delete_posts: bool,
    pub is_moderator: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BanForm {
    pub username: String,
    pub email: String,
    pub message: String,
    pub duration_days: Option<i32>,
}

// ── Topic moderation ──

#[post("/api/delete-topic", headers: HeaderMap)]
pub async fn delete_topic(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }

        #[derive(Deserialize)]
        struct PostCount {
            author_id: i32,
            cnt: i64,
        }
        let post_counts = run_json_query::<Vec<PostCount>>(&format!(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT author_id, COUNT(*)::bigint AS cnt FROM posts WHERE topic_id = {} GROUP BY author_id) r;",
            topic_id
        )).await.map_err(server_error)?;
        for pc in post_counts {
            run_exec(&format!(
                "UPDATE users SET post_count = GREATEST(post_count - {}, 0) WHERE id = {};",
                pc.cnt, pc.author_id
            ))
            .await
            .map_err(server_error)?;
        }
        run_exec(&format!("DELETE FROM posts WHERE topic_id = {};", topic_id))
            .await
            .map_err(server_error)?;
        run_exec(&format!("DELETE FROM topics WHERE id = {};", topic_id))
            .await
            .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/move-topic", headers: HeaderMap)]
pub async fn move_topic(input: MoveTopicForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }
        run_exec(&format!(
            "UPDATE topics SET forum_id = {} WHERE id = {};",
            input.forum_id, input.topic_id
        ))
        .await
        .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/toggle-sticky", headers: HeaderMap)]
pub async fn toggle_sticky(topic_id: i32) -> Result<bool, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }

        #[derive(Deserialize)]
        struct StickyRow {
            sticky: bool,
        }

        let row = run_json_query::<StickyRow>(&format!(
            "SELECT row_to_json(r) FROM (SELECT sticky FROM topics WHERE id = {}) AS r;",
            topic_id
        ))
        .await
        .map_err(server_error)?;

        let new_sticky = !row.sticky;
        run_exec(&format!(
            "UPDATE topics SET sticky = {} WHERE id = {};",
            new_sticky, topic_id
        ))
        .await
        .map_err(server_error)?;

        Ok(new_sticky)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/toggle-topic-status", headers: HeaderMap)]
pub async fn toggle_topic_status(topic_id: i32) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if user.group_id != 1 {
            return Err(server_error("Admin only.".into()));
        }

        #[derive(Deserialize)]
        struct ClosedRow {
            closed: bool,
        }

        let row = run_json_query::<ClosedRow>(&format!(
            "SELECT row_to_json(r) FROM (SELECT closed FROM topics WHERE id = {}) AS r;",
            topic_id
        ))
        .await
        .map_err(server_error)?;

        let new_closed = !row.closed;
        run_exec(&format!(
            "UPDATE topics SET closed = {} WHERE id = {};",
            new_closed, topic_id
        ))
        .await
        .map_err(server_error)?;

        Ok(if new_closed { "closed" } else { "open" }.to_string())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Err(ServerFnError::new("server only"))
    }
}

// ── View counter ──

#[post("/api/view-topic")]
pub async fn increment_topic_views(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        run_exec(&format!(
            "UPDATE topics SET views = views + 1 WHERE id = {};",
            topic_id
        ))
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn register_account_impl(input: RegisterForm) -> Result<AuthResponse, String> {
    let username = normalize_username(&input.username);
    let email = input.email.trim().to_lowercase();
    let location = input.location.trim();
    let about = input.about.trim();

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

    let salt = random_hex(16);
    let password_hash = hash_password(&input.password, &salt);

    let mut user = run_json_query::<SessionUser>(&format!(
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
             RETURNING id, username, title, group_id
         )
         SELECT row_to_json(inserted) FROM inserted;",
        username = sql_literal(&username),
        location = sql_literal(location),
        about = sql_literal(about),
        email = sql_literal(&email),
        password_hash = sql_literal(&password_hash),
    ))
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

    let user = run_json_query::<Option<StoredAuthUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(user_row)
             FROM (
                 SELECT id, username, title, group_id, email, password_hash
                 FROM users
                 WHERE LOWER(username) = LOWER({username})
                 LIMIT 1
             ) AS user_row
         ), 'null'::json);",
        username = sql_literal(&username),
    ))
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

    run_json_query::<Option<SessionUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(session_row)
             FROM (
                  SELECT u.id, u.username, u.email, u.title, u.group_id, s.csrf_token
                  FROM forum_sessions AS s
                  INNER JOIN users AS u ON u.id = s.user_id
                  WHERE s.token = {token}
                   AND s.expires_at > EXTRACT(EPOCH FROM now())::bigint
                 LIMIT 1
             ) AS session_row
         ), 'null'::json);",
        token = sql_literal(&token),
    ))
    .await
}

#[cfg(feature = "server")]
async fn logout_account_impl(headers: HeaderMap) -> Result<(), String> {
    if let Some(token) = parse_session_cookie(&headers) {
        run_exec(&format!(
            "DELETE FROM forum_sessions WHERE token = {token};",
            token = sql_literal(&token)
        ))
        .await?;
    }

    Ok(())
}

#[cfg(feature = "server")]
async fn check_installed_impl() -> Result<bool, String> {
    let count = run_scalar_i64(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'board_meta' AND table_schema = 'public';",
    ).await?;
    if count == 0 {
        return Ok(false);
    }
    let rows = run_scalar_i64("SELECT COUNT(*) FROM board_meta;").await?;
    Ok(rows > 0)
}

#[cfg(feature = "server")]
async fn install_board_impl(input: InstallForm) -> Result<AuthResponse, String> {
    let title = input.board_title.trim();
    let tagline = input.board_tagline.trim();
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

    // Build and store database URL
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

    // Run schema — split into individual statements because sqlx
    // prepared statements only allow one command per execution.
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

    // Default groups
    run_exec(
        "INSERT INTO groups (id, title, read_board, post_topics, post_replies, edit_posts, delete_posts, is_moderator, is_admin)
         VALUES
             (1, 'Administrators', true, true, true, true, true, true, true),
             (2, 'Moderators', true, true, true, true, true, true, false),
             (3, 'Guests', true, false, false, false, false, false, false),
             (4, 'Members', true, true, true, true, false, false, false)
         ON CONFLICT (id) DO NOTHING;",
    ).await?;

    // Board meta
    run_exec(&format!(
        "INSERT INTO board_meta (title, tagline) VALUES ({title}, {tagline}) ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title, tagline = EXCLUDED.tagline;",
        title = sql_literal(title),
        tagline = sql_literal(tagline),
    )).await?;

    // Admin user
    let salt = random_hex(16);
    let password_hash = hash_password(&input.admin_password, &salt);

    let mut user = run_json_query::<SessionUser>(&format!(
        "WITH ins AS (
             INSERT INTO users (username, title, status, joined_at, email, password_hash, group_id, registered_at, last_visit)
             VALUES ({username}, 'Administrator', 'Online', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD'), {email}, {password_hash}, 1, EXTRACT(EPOCH FROM now())::bigint, EXTRACT(EPOCH FROM now())::bigint)
             RETURNING id, username, title, group_id
         ) SELECT row_to_json(ins) FROM ins;",
        username = sql_literal(&username),
        email = sql_literal(&email),
        password_hash = sql_literal(&password_hash),
    )).await?;

    // Default category and forum
    run_exec("INSERT INTO categories (name, description, sort_order) VALUES ('General', 'Main discussion area.', 1);").await?;
    run_exec(&format!(
        "INSERT INTO forums (category_id, name, description, moderators, sort_order) VALUES (1, 'General Discussion', 'Talk about anything.', ARRAY[{username}], 1);",
        username = sql_literal(&username),
    )).await?;

    let (session_token, csrf_token) = create_session(user.id).await?;
    user.csrf_token = csrf_token;
    Ok(AuthResponse {
        user,
        session_token,
        message: "Board installed. You are signed in as administrator.".to_string(),
    })
}

#[cfg(feature = "server")]
async fn require_session(headers: &HeaderMap) -> Result<SessionUser, String> {
    let token = parse_session_cookie(headers)
        .ok_or_else(|| "You must be signed in to do this.".to_string())?;
    run_json_query::<Option<SessionUser>>(&format!(
        "SELECT COALESCE((
             SELECT row_to_json(r) FROM (
                 SELECT u.id, u.username, u.title, u.group_id
                 FROM forum_sessions s
                 JOIN users u ON u.id = s.user_id
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
async fn require_session_csrf(headers: &HeaderMap) -> Result<SessionUser, String> {
    let user = require_session(headers).await?;
    validate_csrf(headers).await?;
    Ok(user)
}

#[cfg(feature = "server")]
async fn get_group(group_id: i32) -> Result<Group, String> {
    run_json_query::<Option<Group>>(&format!(
        "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT id, title, read_board, post_topics, post_replies, edit_posts, delete_posts, is_moderator, is_admin FROM groups WHERE id = {}) r), 'null'::json);",
        group_id,
    )).await?
    .ok_or_else(|| "Group not found.".to_string())
}

#[cfg(feature = "server")]
async fn check_flood(user_id: i32, is_admin: bool) -> Result<(), String> {
    if is_admin {
        return Ok(());
    }
    let last_post = run_scalar_i64(&format!(
        "SELECT COALESCE(EXTRACT(EPOCH FROM MAX(posted_at::timestamp))::bigint, 0) FROM posts WHERE author_id = {};",
        user_id
    )).await?;
    let now = unix_now();
    if last_post > 0 && now - last_post < 30 {
        return Err("Please wait at least 30 seconds between posts.".to_string());
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn require_permission(
    headers: &HeaderMap,
    perm: fn(&Group) -> bool,
) -> Result<SessionUser, String> {
    let user = require_session_csrf(headers).await?;
    let group = get_group(user.group_id).await?;
    if !perm(&group) && !group.is_admin {
        return Err("You do not have permission to do this.".to_string());
    }
    Ok(user)
}

#[cfg(feature = "server")]
async fn create_topic_impl(
    input: NewTopicForm,
    headers: HeaderMap,
) -> Result<NewTopicResult, String> {
    let user = require_permission(&headers, |g| g.post_topics).await?;
    if let Some(msg) = check_ban(&user.username, &user.email).await? {
        return Err(format!("You are banned: {msg}"));
    }
    let group = get_group(user.group_id).await?;
    check_flood(user.id, group.is_admin).await?;
    let subject = input.subject.trim();
    let message = input.message.trim();
    if subject.is_empty() {
        return Err("Subject is required.".to_string());
    }
    if message.is_empty() {
        return Err("Message is required.".to_string());
    }
    if subject.len() > 70 {
        return Err("Subject must be 70 characters or fewer.".to_string());
    }

    let now_str = "to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC')";

    // Create topic
    #[derive(Deserialize)]
    struct IdRow {
        id: i32,
    }
    let topic = run_json_query::<IdRow>(&format!(
        "WITH ins AS (
             INSERT INTO topics (forum_id, author_id, subject, closed, created_at, updated_at, activity_rank, sticky, moved_to)
             VALUES ({fid}, {uid}, {subject}, false, {now}, {now}, EXTRACT(EPOCH FROM now())::integer, false, 0)
             RETURNING id
         ) SELECT row_to_json(ins) FROM ins;",
        fid = input.forum_id,
        uid = user.id,
        subject = sql_literal(subject),
        now = now_str,
    )).await?;

    // Create first post
    run_exec(&format!(
        "INSERT INTO posts (topic_id, author_id, posted_at, body, position)
         VALUES ({tid}, {uid}, {now}, ARRAY[{msg}], 1);",
        tid = topic.id,
        uid = user.id,
        now = now_str,
        msg = sql_literal(message),
    ))
    .await?;

    // Increment post count
    run_exec(&format!(
        "UPDATE users SET post_count = post_count + 1 WHERE id = {};",
        user.id
    ))
    .await?;

    Ok(NewTopicResult { topic_id: topic.id })
}

#[cfg(feature = "server")]
async fn create_reply_impl(input: ReplyForm, headers: HeaderMap) -> Result<(), String> {
    let user = require_permission(&headers, |g| g.post_replies).await?;
    if let Some(msg) = check_ban(&user.username, &user.email).await? {
        return Err(format!("You are banned: {msg}"));
    }
    let group = get_group(user.group_id).await?;
    check_flood(user.id, group.is_admin).await?;
    let message = input.message.trim();
    if message.is_empty() {
        return Err("Message is required.".to_string());
    }

    let now_str = "to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC')";

    // Check topic is not closed
    #[derive(Deserialize)]
    struct TopicCheck {
        closed: bool,
    }
    let topic = run_json_query::<TopicCheck>(&format!(
        "SELECT row_to_json(r) FROM (SELECT closed FROM topics WHERE id = {}) AS r;",
        input.topic_id
    ))
    .await?;
    if topic.closed {
        return Err("This topic is closed. No new replies are allowed.".to_string());
    }

    // Get next position
    let pos = run_scalar_i64(&format!(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM posts WHERE topic_id = {};",
        input.topic_id,
    ))
    .await?;

    // Insert reply
    run_exec(&format!(
        "INSERT INTO posts (topic_id, author_id, posted_at, body, position)
         VALUES ({tid}, {uid}, {now}, ARRAY[{msg}], {pos});",
        tid = input.topic_id,
        uid = user.id,
        now = now_str,
        msg = sql_literal(message),
        pos = pos,
    ))
    .await?;

    // Update topic timestamps
    run_exec(&format!(
        "UPDATE topics SET updated_at = {now}, activity_rank = EXTRACT(EPOCH FROM now())::integer WHERE id = {tid};",
        now = now_str,
        tid = input.topic_id,
    )).await?;

    // Increment post count
    run_exec(&format!(
        "UPDATE users SET post_count = post_count + 1 WHERE id = {};",
        user.id
    ))
    .await?;

    Ok(())
}

#[cfg(feature = "server")]
async fn username_exists(username: &str) -> Result<bool, String> {
    let count = run_scalar_i64(&format!(
        "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER({username});",
        username = sql_literal(username)
    ))
    .await?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
async fn email_exists(email: &str) -> Result<bool, String> {
    let count = run_scalar_i64(&format!(
        "SELECT COUNT(*) FROM users WHERE LOWER(email) = LOWER({email});",
        email = sql_literal(email)
    ))
    .await?;
    Ok(count > 0)
}

#[cfg(feature = "server")]
async fn create_session(user_id: i32) -> Result<(String, String), String> {
    let token = random_hex(32);
    let csrf = random_hex(16);
    let now = unix_now();
    let expires = now + SESSION_MAX_AGE_SECS;

    run_exec(&format!(
        "INSERT INTO forum_sessions (token, user_id, created_at, expires_at, last_seen, csrf_token)
         VALUES ({token}, {user_id}, {now}, {expires}, {now}, {csrf});",
        token = sql_literal(&token),
        csrf = sql_literal(&csrf),
    ))
    .await?;

    Ok((token, csrf))
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
    )).await?;

    match row {
        Some(r) if r.csrf_token == csrf_token => Ok(()),
        _ => Err("Invalid CSRF token. Please refresh the page.".to_string()),
    }
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
async fn run_json_query<T>(sql: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let pool = db_pool().await?;
    let row = sqlx::query(sql)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("query failed: {e}"))?;
    let payload: serde_json::Value = row.get(0);
    serde_json::from_value(payload).map_err(|e| format!("failed to parse postgres json: {e}"))
}

#[cfg(feature = "server")]
async fn run_scalar_i64(sql: &str) -> Result<i64, String> {
    let pool = db_pool().await?;
    let row = sqlx::query(sql)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("query failed: {e}"))?;
    // PostgreSQL may return INT4 or INT8; accept either.
    let val: i64 = match row.try_get::<i32, _>(0) {
        Ok(v) => v as i64,
        Err(_) => row
            .try_get::<i64, _>(0)
            .map_err(|e| format!("scalar decode failed: {e}"))?,
    };
    Ok(val)
}

#[cfg(feature = "server")]
async fn run_exec(sql: &str) -> Result<(), String> {
    let pool = db_pool().await?;
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| format!("exec failed: {e}"))?;
    Ok(())
}

pub fn cookie_name() -> &'static str {
    SESSION_COOKIE
}

pub fn csrf_cookie_name() -> &'static str {
    CSRF_COOKIE
}

pub fn cookie_max_age() -> i64 {
    SESSION_MAX_AGE_SECS
}

/// Strip Dioxus server-function error wrapping so only the message is shown.
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
