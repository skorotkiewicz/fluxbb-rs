use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use serde::Deserialize;

#[cfg(feature = "server")]
use super::{
    db::{run_parameterized_exec, run_parameterized_json, run_parameterized_scalar_i64, server_error, PgBind},
    security::{
        check_ban,
        check_flood,
        check_permission,
        hash_password,
        parse_session_cookie,
        require_session_csrf,
        unix_now,
        verify_password,
        Permission,
    },
};
use super::{
    Attachment, ChangePasswordForm, EditPostForm, Forum, ForumData, IndexData, MoveTopicForm,
    NewTopicForm, NewTopicResult, Post, ProfileData, ReplyForm, SearchResults, ShellData,
    TopicData, UpdateProfileForm, UserProfile,
};

#[post("/api/shell-data")]
pub async fn load_shell_data() -> Result<ShellData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let cutoff = unix_now() - 900;
        let data = run_parameterized_json::<ShellData>(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body, smtp_enable FROM board_meta LIMIT 1) m),
                'stats', json_build_object(
                    'members', (SELECT COUNT(*)::int FROM users),
                    'topics', (SELECT COUNT(*)::int FROM topics),
                    'posts', (SELECT COUNT(*)::int FROM posts),
                    'newest_member', COALESCE((SELECT username FROM users ORDER BY id DESC LIMIT 1), '')
                ),
                'online_users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT DISTINCT u.id, u.username, u.title
                    FROM forum_sessions s
                    JOIN users u ON u.id = s.user_id
                    WHERE s.last_seen > $1
                      AND u.show_online = true
                    ORDER BY u.username
                ) u)
            )::json;",
            &[&cutoff as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
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
            run_parameterized_scalar_i64(
                "SELECT COALESCE((SELECT user_id FROM forum_sessions WHERE token = $1 AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1), 0);",
                &[&token as &(dyn PgBind + Sync)],
            )
            .await
            .unwrap_or(0)
        } else {
            0
        };
        let data = run_parameterized_json::<IndexData>(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body, smtp_enable FROM board_meta LIMIT 1) m),
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
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, group_id
                    FROM users WHERE id IN (SELECT author_id FROM topics ORDER BY activity_rank DESC, id LIMIT 4)
                ) u),
                'last_visit', COALESCE((SELECT last_visit FROM users WHERE id = $1), 0)
            )::json;",
            &[&user_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
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
        let forums = run_parameterized_json::<Vec<Forum>>(
            "SELECT COALESCE(json_agg(row_to_json(f)), '[]'::json) FROM (SELECT id, category_id, name, description, moderators, sort_order FROM forums ORDER BY category_id, sort_order, id) f;",
            &[],
        )
        .await
        .map_err(server_error)?;
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
            run_parameterized_scalar_i64(
                "SELECT COALESCE((SELECT user_id FROM forum_sessions WHERE token = $1 AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1), 0);",
                &[&token as &(dyn PgBind + Sync)],
            )
            .await
            .unwrap_or(0)
        } else {
            0
        };
        let page = page.max(1);
        let per_page = if user_id > 0 {
            run_parameterized_scalar_i64(
                "SELECT COALESCE((SELECT disp_topics FROM users WHERE id = $1), 25);",
                &[&user_id as &(dyn PgBind + Sync)],
            )
            .await
            .unwrap_or(25) as i32
        } else {
            25
        }
        .clamp(5, 100);
        let offset = (page - 1) * per_page;
        let data = run_parameterized_json::<ForumData>(
            "SELECT json_build_object(
                'forum', (SELECT row_to_json(f) FROM forums f WHERE f.id = $1),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to,
                        COALESCE((SELECT COUNT(*) FROM posts WHERE topic_id = t.id), 0) - 1 AS reply_count
                    FROM topics t WHERE forum_id = $1 ORDER BY sticky DESC, activity_rank DESC, id
                    LIMIT $2 OFFSET $3
                ) t),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, group_id
                    FROM users WHERE id IN (SELECT author_id FROM topics WHERE forum_id = $1)
                ) u),
                'total_topics', (SELECT COUNT(*) FROM topics WHERE forum_id = $1),
                'page', $4,
                'per_page', $2,
                'last_visit', COALESCE((SELECT last_visit FROM users WHERE id = $5), 0)
            )::json;",
            &[&id as &(dyn PgBind + Sync), &per_page, &offset, &page, &user_id],
        )
        .await
        .map_err(server_error)?;
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
        let token = parse_session_cookie(&headers);
        let user_id = if let Some(token) = token {
            run_parameterized_scalar_i64(
                "SELECT COALESCE((SELECT user_id FROM forum_sessions WHERE token = $1 AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1), 0);",
                &[&token as &(dyn PgBind + Sync)],
            )
            .await
            .unwrap_or(0)
        } else {
            0
        };
        let page = page.max(1);
        let per_page = if user_id > 0 {
            run_parameterized_scalar_i64(
                "SELECT COALESCE((SELECT disp_posts FROM users WHERE id = $1), 20);",
                &[&user_id as &(dyn PgBind + Sync)],
            )
            .await
            .unwrap_or(20) as i32
        } else {
            20
        }
        .clamp(5, 100);
        let offset = (page - 1) * per_page;
        let data = run_parameterized_json::<TopicData>(
            "SELECT json_build_object(
                'topic', (SELECT row_to_json(t) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to,
                        COALESCE((SELECT COUNT(*) FROM posts WHERE topic_id = topics.id), 0) - 1 AS reply_count
                    FROM topics WHERE id = $1
                ) t),
                'posts', (SELECT COALESCE(json_agg(row_to_json(p)), '[]'::json) FROM (
                    SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position,
                        COALESCE((SELECT json_agg(row_to_json(a)) FROM (
                            SELECT id, post_id, filename, file_size, mime_type, '/' || storage_path AS download_url, uploaded_at
                            FROM attachments WHERE post_id = posts.id
                        ) a), '[]'::json) AS attachments
                    FROM posts WHERE topic_id = $1 ORDER BY position, id
                    LIMIT $2 OFFSET $3
                ) p),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, group_id
                    FROM users WHERE id IN (SELECT author_id FROM posts WHERE topic_id = $1)
                ) u),
                'forum', (SELECT row_to_json(f) FROM forums f WHERE f.id = (SELECT forum_id FROM topics WHERE id = $1)),
                'total_posts', (SELECT COUNT(*) FROM posts WHERE topic_id = $1),
                'page', $4,
                'per_page', $2
            )::json;",
            &[&id as &(dyn PgBind + Sync), &per_page, &offset, &page],
        )
        .await
        .map_err(server_error)?;

        if let Some(token) = parse_session_cookie(&headers) {
            let _ = run_parameterized_exec(
                "UPDATE users SET last_visit = EXTRACT(EPOCH FROM now())::bigint WHERE id = (SELECT user_id FROM forum_sessions WHERE token = $1 AND expires_at > EXTRACT(EPOCH FROM now())::bigint LIMIT 1);",
                &[&token as &(dyn PgBind + Sync)],
            )
            .await;
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
        let data = run_parameterized_json::<ProfileData>(
            "SELECT json_build_object(
                'user', (SELECT row_to_json(u) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id, timezone, disp_topics, disp_posts, show_online, theme
                    FROM users WHERE id = $1
                ) u),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                    FROM topics WHERE author_id = $1 ORDER BY activity_rank DESC, id LIMIT 10
                ) t),
                'posts', (SELECT COALESCE(json_agg(row_to_json(p)), '[]'::json) FROM (
                    SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position
                    FROM posts WHERE author_id = $1 ORDER BY posted_at DESC LIMIT 10
                ) p)
            )::json;",
            &[&id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
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
        let users = run_parameterized_json::<Vec<UserProfile>>(
            "SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, group_id
                FROM users ORDER BY post_count DESC, id
            ) u;",
            &[],
        )
        .await
        .map_err(server_error)?;
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
        let results = run_parameterized_json::<SearchResults>(
            "SELECT json_build_object(
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                    SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                    FROM topics
                    WHERE LOWER(subject) LIKE $1
                       OR EXISTS (SELECT 1 FROM unnest(tags) tag WHERE LOWER(tag) LIKE $2)
                       OR id IN (SELECT DISTINCT topic_id FROM posts p WHERE EXISTS (
                           SELECT 1 FROM unnest(p.body) para WHERE LOWER(para) LIKE $3
                       ))
                    ORDER BY activity_rank DESC
                    LIMIT 20
                ) t),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (
                    SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, group_id
                    FROM users
                    WHERE LOWER(username) LIKE $4
                       OR LOWER(title) LIKE $5
                       OR LOWER(about) LIKE $6
                       OR LOWER(location) LIKE $7
                    LIMIT 20
                ) u)
            )::json;",
            &[&like as &(dyn PgBind + Sync), &like, &like, &like, &like, &like, &like],
        )
        .await
        .map_err(server_error)?;
        Ok(results)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = query;
        Err(ServerFnError::new("server only"))
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

#[post("/api/post/:id")]
pub async fn load_post(id: i32) -> Result<Post, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let post = run_parameterized_json::<Post>(
            "SELECT row_to_json(post_row) FROM (SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position FROM posts WHERE id = $1) AS post_row;",
            &[&id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
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
        let post = run_parameterized_json::<Option<Post>>(
            "SELECT COALESCE((SELECT row_to_json(post_row) FROM (SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position FROM posts WHERE id = $1) AS post_row), 'null'::json);",
            &[&input.post_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let Some(post) = post else {
            return Err(server_error("Post not found.".into()));
        };

        if post.author_id != user.id {
            check_permission(&user, Permission::Moderator)
                .await
                .map_err(server_error)?;
        } else {
            check_permission(&user, Permission::EditPosts)
                .await
                .map_err(server_error)?;
        }

        let message = input.message.trim().to_string();
        if message.is_empty() {
            return Err(server_error("Message is required.".into()));
        }

        run_parameterized_exec(
            "UPDATE posts SET body = ARRAY[$1], edited_at = to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC') WHERE id = $2;",
            &[&message as &(dyn PgBind + Sync), &input.post_id],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE topics SET updated_at = to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), activity_rank = EXTRACT(EPOCH FROM now())::integer WHERE id = $1;",
            &[&post.topic_id as &(dyn PgBind + Sync)],
        )
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

// Attachment constants
pub const MAX_ATTACHMENT_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "pdf", "txt", "zip", "mp4"];

#[post("/api/attachments/:post_id")]
pub async fn load_attachments(post_id: i32) -> Result<Vec<Attachment>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let attachments = run_parameterized_json::<Vec<Attachment>>(
            "SELECT COALESCE(json_agg(row_to_json(a)), '[]'::json) FROM (
                SELECT id, post_id, filename, file_size, mime_type, '/' || storage_path AS download_url, uploaded_at
                FROM attachments WHERE post_id = $1 ORDER BY id
            ) a;",
            &[&post_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        Ok(attachments)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = post_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/upload-attachment", headers: HeaderMap)]
pub async fn upload_attachment(
    post_id: i32,
    filename: String,
    content: Vec<u8>,
) -> Result<Attachment, ServerFnError> {
    #[cfg(feature = "server")]
    {
        upload_attachment_impl(post_id, filename, content, headers)
            .await
            .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = post_id;
        let _ = filename;
        let _ = content;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/delete-attachment", headers: HeaderMap)]
pub async fn delete_attachment(attachment_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        #[derive(Deserialize)]
        struct AttachmentInfo {
            post_id: i32,
            storage_path: String,
        }

        let info = run_parameterized_json::<AttachmentInfo>(
            "SELECT row_to_json(r) FROM (
                SELECT post_id, storage_path FROM attachments WHERE id = $1
            ) r;",
            &[&attachment_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let author_id: i32 = run_parameterized_scalar_i64(
            "SELECT author_id FROM posts WHERE id = $1;",
            &[&info.post_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)? as i32;

        if author_id != user.id {
            check_permission(&user, Permission::Moderator)
                .await
                .map_err(server_error)?;
        }

        let _ = std::fs::remove_file(&info.storage_path);

        run_parameterized_exec(
            "DELETE FROM attachments WHERE id = $1;",
            &[&attachment_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = attachment_id;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "server")]
async fn upload_attachment_impl(
    post_id: i32,
    filename: String,
    content: Vec<u8>,
    headers: HeaderMap,
) -> Result<Attachment, String> {
    let user = require_session_csrf(&headers).await?;

    let author_id: i32 = run_parameterized_scalar_i64(
        "SELECT author_id FROM posts WHERE id = $1;",
        &[&post_id as &(dyn PgBind + Sync)],
    )
    .await? as i32;

    if author_id != user.id {
        return Err("You can only attach files to your own posts.".to_string());
    }

    // Check file size
    if content.len() > MAX_ATTACHMENT_SIZE {
        return Err(format!(
            "File too large. Maximum size is {} MB.",
            MAX_ATTACHMENT_SIZE / (1024 * 1024)
        ));
    }

    // Validate extension
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!(
            "File type not allowed. Allowed: {}",
            ALLOWED_EXTENSIONS.join(", ")
        ));
    }

    // Determine MIME type
    let mime_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    };
    let mime_type = mime_type.to_string();

    // Create storage directory if needed
    let storage_dir = std::path::Path::new("uploads");
    if !storage_dir.exists() {
        std::fs::create_dir_all(storage_dir)
            .map_err(|e| format!("Failed to create uploads dir: {e}"))?;
    }

    // Generate unique filename
    let unique_name = format!("{}_{}_{}", user.id, unix_now(), filename);
    let storage_path = storage_dir.join(&unique_name);

    // Write file
    std::fs::write(&storage_path, &content).map_err(|e| format!("Failed to save file: {e}"))?;

    let storage_path_str = storage_path.to_string_lossy().to_string();
    let now_str = "to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC')";

    // Insert into database
    let attachment = run_parameterized_json::<Attachment>(
        "WITH ins AS (
            INSERT INTO attachments (post_id, filename, file_size, mime_type, storage_path, uploaded_at)
            VALUES ($1, $2, $3, $4, $5, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'))
            RETURNING id, post_id, filename, file_size, mime_type, '/' || storage_path AS download_url, uploaded_at
        ) SELECT row_to_json(ins) FROM ins;",
        &[
            &post_id as &(dyn PgBind + Sync),
            &filename,
            &(content.len() as i64),
            &mime_type,
            &storage_path_str,
        ],
    )
    .await?;

    Ok(attachment)
}

#[cfg(feature = "server")]
async fn create_topic_impl(
    input: NewTopicForm,
    headers: HeaderMap,
) -> Result<NewTopicResult, String> {
    let user = require_session_csrf(&headers).await?;
    check_permission(&user, Permission::PostTopics).await?;
    if let Some(msg) = check_ban(&user.username, &user.email).await? {
        return Err(format!("You are banned: {msg}"));
    }
    check_flood(user.id, user.is_admin).await?;
    let subject = input.subject.trim().to_string();
    let message = input.message.trim().to_string();
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

    #[derive(Deserialize)]
    struct IdRow {
        id: i32,
    }
    let topic = run_parameterized_json::<IdRow>(
        "WITH ins AS (
             INSERT INTO topics (forum_id, author_id, subject, closed, created_at, updated_at, activity_rank, sticky, moved_to)
             VALUES ($1, $2, $3, false, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), EXTRACT(EPOCH FROM now())::integer, false, 0)
             RETURNING id
         ) SELECT row_to_json(ins) FROM ins;",
        &[&input.forum_id as &(dyn PgBind + Sync), &user.id, &subject],
    )
    .await?;

    run_parameterized_exec(
        "INSERT INTO posts (topic_id, author_id, posted_at, body, position)
         VALUES ($1, $2, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), ARRAY[$3], 1);",
        &[&topic.id as &(dyn PgBind + Sync), &user.id, &message],
    )
    .await?;

    run_parameterized_exec(
        "UPDATE users SET post_count = post_count + 1 WHERE id = $1;",
        &[&user.id as &(dyn PgBind + Sync)],
    )
    .await?;

    Ok(NewTopicResult { topic_id: topic.id })
}

#[cfg(feature = "server")]
async fn create_reply_impl(input: ReplyForm, headers: HeaderMap) -> Result<(), String> {
    let user = require_session_csrf(&headers).await?;
    check_permission(&user, Permission::PostReplies).await?;
    if let Some(msg) = check_ban(&user.username, &user.email).await? {
        return Err(format!("You are banned: {msg}"));
    }
    check_flood(user.id, user.is_admin).await?;
    let message = input.message.trim().to_string();
    if message.is_empty() {
        return Err("Message is required.".to_string());
    }

    let now_str = "to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC')";

    #[derive(Deserialize)]
    struct TopicCheck {
        closed: bool,
    }
    let topic = run_parameterized_json::<TopicCheck>(
        "SELECT row_to_json(r) FROM (SELECT closed FROM topics WHERE id = $1) AS r;",
        &[&input.topic_id as &(dyn PgBind + Sync)],
    )
    .await?;
    if topic.closed {
        return Err("This topic is closed. No new replies are allowed.".to_string());
    }

    let pos = run_parameterized_scalar_i64(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM posts WHERE topic_id = $1;",
        &[&input.topic_id as &(dyn PgBind + Sync)],
    )
    .await?;

    run_parameterized_exec(
        "INSERT INTO posts (topic_id, author_id, posted_at, body, position)
         VALUES ($1, $2, to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), ARRAY[$3], $4);",
        &[&input.topic_id as &(dyn PgBind + Sync), &user.id, &message, &(pos as i32)],
    )
    .await?;

    run_parameterized_exec(
        "UPDATE topics SET updated_at = to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI UTC'), activity_rank = EXTRACT(EPOCH FROM now())::integer WHERE id = $1;",
        &[&input.topic_id as &(dyn PgBind + Sync)],
    )
    .await?;

    run_parameterized_exec(
        "UPDATE users SET post_count = post_count + 1 WHERE id = $1;",
        &[&user.id as &(dyn PgBind + Sync)],
    )
    .await?;

    Ok(())
}

#[post("/api/delete-post", headers: HeaderMap)]
pub async fn delete_post(post_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Get post info
        #[derive(Deserialize)]
        struct PostInfo {
            author_id: i32,
            topic_id: i32,
        }
        let info = run_parameterized_json::<PostInfo>(
            "SELECT row_to_json(r) FROM (SELECT author_id, topic_id FROM posts WHERE id = $1) AS r;",
            &[&post_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        // Check if user is author or has delete permission
        if info.author_id != user.id {
            check_permission(&user, Permission::DeletePosts)
                .await
                .map_err(server_error)?;
        }

        // Delete the post
        run_parameterized_exec(
            "DELETE FROM posts WHERE id = $1;",
            &[&post_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        // Update user post count (with floor check)
        run_parameterized_exec(
            "UPDATE users SET post_count = GREATEST(post_count - 1, 0) WHERE id = $1;",
            &[&info.author_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = post_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/delete-topic", headers: HeaderMap)]
pub async fn delete_topic(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        check_permission(&user, Permission::DeleteTopic)
            .await
            .map_err(server_error)?;

        // Delete topic and all posts
        run_parameterized_exec(
            "DELETE FROM posts WHERE topic_id = $1;",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "DELETE FROM topics WHERE id = $1;",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
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
        check_permission(&user, Permission::MoveTopic)
            .await
            .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE topics SET forum_id = $1 WHERE id = $2;",
            &[&input.forum_id as &(dyn PgBind + Sync), &input.topic_id],
        )
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
pub async fn toggle_sticky(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        check_permission(&user, Permission::StickyTopic)
            .await
            .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE topics SET sticky = NOT sticky WHERE id = $1;",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
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

#[post("/api/toggle-topic-status", headers: HeaderMap)]
pub async fn toggle_topic_status(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        check_permission(&user, Permission::CloseTopic)
            .await
            .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE topics SET closed = NOT closed WHERE id = $1;",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
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

#[post("/api/mark-all-read", headers: HeaderMap)]
pub async fn mark_all_read() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        run_parameterized_exec(
            "UPDATE users SET last_visit = EXTRACT(EPOCH FROM now())::bigint WHERE id = $1;",
            &[&user.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/update-profile", headers: HeaderMap)]
pub async fn update_profile(input: UpdateProfileForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Can only update own profile unless admin
        if input.user_id != user.id && !user.is_admin {
            return Err(ServerFnError::new("Not authorized"));
        }

        let email = input.email.trim().to_string();
        let location = input.location.trim().to_string();
        let about = input.about.trim().to_string();

        if email.is_empty() {
            return Err(ServerFnError::new("Email is required."));
        }

        run_parameterized_exec(
            "UPDATE users SET email = $1, location = $2, about = $3,
             timezone = $4, disp_topics = $5, disp_posts = $6, show_online = $7, theme = $8 WHERE id = $9;",
            &[
                &email as &(dyn PgBind + Sync),
                &location,
                &about,
                &input.timezone,
                &input.disp_topics,
                &input.disp_posts,
                &input.show_online,
                &input.theme,
                &input.user_id,
            ],
        )
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

        if let Some(msg) = check_ban(&user.username, &user.email)
            .await
            .map_err(server_error)?
        {
            return Err(server_error(format!("You are banned: {msg}")));
        }

        #[derive(Deserialize)]
        struct UserPass {
            password_hash: String,
        }
        let stored = run_parameterized_json::<UserPass>(
            "SELECT row_to_json(r) FROM (SELECT password_hash FROM users WHERE id = $1) AS r;",
            &[&user.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if !verify_password(&input.old_password, &stored.password_hash) {
            return Err(ServerFnError::new("Current password is incorrect."));
        }

        if input.new_password.chars().count() < 9 {
            return Err(ServerFnError::new(
                "Password must be at least 9 characters.",
            ));
        }

        if input.new_password != input.confirm_password {
            return Err(ServerFnError::new("New passwords do not match."));
        }

        let new_hash = hash_password(&input.new_password);

        run_parameterized_exec(
            "UPDATE users SET password_hash = $1 WHERE id = $2;",
            &[&new_hash as &(dyn PgBind + Sync), &user.id],
        )
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

#[post("/api/view-topic")]
pub async fn increment_topic_views(topic_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        run_parameterized_exec(
            "UPDATE topics SET views = views + 1 WHERE id = $1;",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
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
