use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use serde::Deserialize;

#[cfg(feature = "server")]
use super::{
    db::{
        run_parameterized_exec, run_parameterized_json, run_parameterized_scalar_i64, server_error,
        PgBind,
    },
    security::{require_session, require_session_csrf, unix_now},
};
use super::{
    AdminBoardSettings, AdminCategoryForm, AdminCategoryUpdate, AdminData, AdminDeleteItem,
    AdminForumForm, AdminForumUpdate, AdminUserUpdate, Ban, BanForm, Group, GroupUpdateForm,
    ReportPostForm, TestSmtpForm,
};

#[post("/api/admin-data", headers: HeaderMap)]
pub async fn load_admin_data() -> Result<AdminData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        let data = run_parameterized_json::<AdminData>(
            "SELECT json_build_object(
                'meta', (SELECT row_to_json(m) FROM (SELECT title, tagline, announcement_title, announcement_body, smtp_host, smtp_port, smtp_user, smtp_pass, smtp_from_email, smtp_from_name, smtp_enable FROM board_meta LIMIT 1) m),
                'categories', (SELECT COALESCE(json_agg(row_to_json(c)), '[]'::json) FROM (SELECT id, name, description, sort_order FROM categories ORDER BY sort_order, id) c),
                'forums', (SELECT COALESCE(json_agg(row_to_json(f)), '[]'::json) FROM (SELECT id, category_id, name, description, moderators, sort_order FROM forums ORDER BY category_id, sort_order, id) f),
                'users', (SELECT COALESCE(json_agg(row_to_json(u)), '[]'::json) FROM (SELECT id, username, title, status, joined_at, post_count, location, about, last_seen, email, group_id, timezone, disp_topics, disp_posts, show_online FROM users ORDER BY id) u),
                'topics', (SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to FROM topics ORDER BY sticky DESC, activity_rank DESC, id) t),
                'reports', (SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (
                    SELECT rep.id, rep.post_id, rep.reporter_id, u.username AS reporter_name, rep.reason, rep.created_at, rep.zapped,
                           p.body AS post_body, p.topic_id, t.subject AS topic_subject, p.author_id, au.username AS author_name
                    FROM reports rep
                    JOIN posts p ON p.id = rep.post_id
                    JOIN topics t ON t.id = p.topic_id
                    JOIN users u ON u.id = rep.reporter_id
                    JOIN users au ON au.id = p.author_id
                    WHERE rep.zapped = false
                    ORDER BY rep.created_at DESC
                ) r)
            )::json;",
            &[],
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

#[post("/api/admin/add-category", headers: HeaderMap)]
pub async fn admin_add_category(input: AdminCategoryForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageCategories)
        //     .await
        //     .map_err(server_error)?;
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();
        run_parameterized_exec(
            "INSERT INTO categories (name, description, sort_order) VALUES ($1, $2, (SELECT COALESCE(MAX(sort_order),0)+1 FROM categories));",
            &[&name as &(dyn PgBind + Sync), &description],
        )
        .await
        .map_err(server_error)
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageForums)
        //     .await
        //     .map_err(server_error)?;
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();
        run_parameterized_exec(
            "INSERT INTO forums (category_id, name, description, sort_order) VALUES ($1, $2, $3, (SELECT COALESCE(MAX(sort_order),0)+1 FROM forums WHERE category_id=$1));",
            &[&input.category_id as &(dyn PgBind + Sync), &name, &description],
        )
        .await
        .map_err(server_error)
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageCategories)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM categories WHERE id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
        )
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageForums)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM forums WHERE id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/update-category", headers: HeaderMap)]
pub async fn admin_update_category(input: AdminCategoryUpdate) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageCategories)
        //     .await
        //     .map_err(server_error)?;
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();
        run_parameterized_exec(
            "UPDATE categories SET name = $1, description = $2, sort_order = $3 WHERE id = $4;",
            &[
                &name as &(dyn PgBind + Sync),
                &description,
                &input.sort_order,
                &input.id,
            ],
        )
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/admin/update-forum", headers: HeaderMap)]
pub async fn admin_update_forum(input: AdminForumUpdate) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageForums)
        //     .await
        //     .map_err(server_error)?;
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();
        run_parameterized_exec(
            "UPDATE forums SET category_id = $1, name = $2, description = $3, sort_order = $4 WHERE id = $5;",
            &[&input.category_id as &(dyn PgBind + Sync), &name, &description, &input.sort_order, &input.id],
        )
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageUsers)
        //     .await
        //     .map_err(server_error)?;
        let title = input.title.trim().to_string();
        run_parameterized_exec(
            "UPDATE users SET group_id = $1, title = $2 WHERE id = $3;",
            &[
                &input.group_id as &(dyn PgBind + Sync),
                &title,
                &input.user_id,
            ],
        )
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageUsers)
        //     .await
        //     .map_err(server_error)?;
        if input.id == u.id {
            return Err(server_error("Cannot delete yourself.".into()));
        }
        run_parameterized_exec(
            "DELETE FROM forum_sessions WHERE user_id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM users WHERE id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
        )
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::DeleteTopic)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM posts WHERE topic_id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM topics WHERE id = $1;",
            &[&input.id as &(dyn PgBind + Sync)],
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

#[post("/api/admin/update-board", headers: HeaderMap)]
pub async fn admin_update_board(input: AdminBoardSettings) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageSettings)
        //     .await
        //     .map_err(server_error)?;
        let title = input.title.trim().to_string();
        let tagline = input.tagline.trim().to_string();
        let ann_title = input.announcement_title.trim().to_string();
        let ann_body = input.announcement_body.trim().to_string();
        let smtp_host = input.smtp_host.trim().to_string();
        let smtp_user = input.smtp_user.trim().to_string();
        let smtp_pass = input.smtp_pass.trim().to_string();
        let smtp_from_email = input.smtp_from_email.trim().to_string();
        let smtp_from_name = input.smtp_from_name.trim().to_string();
        run_parameterized_exec(
            "UPDATE board_meta SET title = $1, tagline = $2, announcement_title = $3, announcement_body = $4, smtp_host = $5, smtp_port = $6, smtp_user = $7, smtp_pass = $8, smtp_from_email = $9, smtp_from_name = $10, smtp_enable = $11 WHERE id = 1;",
            &[
                &title as &(dyn PgBind + Sync),
                &tagline,
                &ann_title,
                &ann_body,
                &smtp_host,
                &input.smtp_port,
                &smtp_user,
                &smtp_pass,
                &smtp_from_email,
                &smtp_from_name,
                &input.smtp_enable,
            ],
        )
        .await
        .map_err(server_error)
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageSettings)
        //     .await
        //     .map_err(server_error)?;
        let deleted = run_parameterized_scalar_i64(
            "WITH deleted AS (DELETE FROM forum_sessions WHERE expires_at < EXTRACT(EPOCH FROM now())::bigint RETURNING *) SELECT COUNT(*) FROM deleted;",
            &[],
        )
        .await
        .map_err(server_error)?;
        Ok(deleted)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/report-post", headers: HeaderMap)]
pub async fn report_post(input: ReportPostForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        let reason = input.reason.trim().to_string();
        if reason.is_empty() {
            return Err(server_error(
                "Please provide a reason for the report.".into(),
            ));
        }
        let now = unix_now();
        run_parameterized_exec(
            "INSERT INTO reports (post_id, reporter_id, reason, created_at) VALUES ($1, $2, $3, $4);",
            &[&input.post_id as &(dyn PgBind + Sync), &user.id, &reason, &now],
        )
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/dismiss-report", headers: HeaderMap)]
pub async fn dismiss_report(report_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::Moderator)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "UPDATE reports SET zapped = true WHERE id = $1;",
            &[&report_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = report_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/zap-report", headers: HeaderMap)]
pub async fn zap_report(report_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::Moderator)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "UPDATE reports SET zapped = true WHERE id = $1;",
            &[&report_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        #[derive(Deserialize)]
        struct PostInfo {
            topic_id: i32,
            author_id: i32,
            is_first: bool,
        }
        let info = run_parameterized_json::<PostInfo>(
            "SELECT row_to_json(r) FROM (
                SELECT p.topic_id, p.author_id,
                       CASE WHEN p.id = (SELECT MIN(id) FROM posts WHERE topic_id = p.topic_id) THEN true ELSE false END AS is_first
                FROM posts p
                WHERE p.id = (SELECT post_id FROM reports WHERE id = $1)
            ) r;",
            &[&report_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        if info.is_first {
            run_parameterized_exec(
                "DELETE FROM posts WHERE topic_id = $1;",
                &[&info.topic_id as &(dyn PgBind + Sync)],
            )
            .await
            .map_err(server_error)?;
            run_parameterized_exec(
                "DELETE FROM topics WHERE id = $1;",
                &[&info.topic_id as &(dyn PgBind + Sync)],
            )
            .await
            .map_err(server_error)?;
        } else {
            run_parameterized_exec(
                "DELETE FROM posts WHERE id = (SELECT post_id FROM reports WHERE id = $1);",
                &[&report_id as &(dyn PgBind + Sync)],
            )
            .await
            .map_err(server_error)?;
            run_parameterized_exec(
                "UPDATE users SET post_count = GREATEST(post_count - 1, 0) WHERE id = $1;",
                &[&info.author_id as &(dyn PgBind + Sync)],
            )
            .await
            .map_err(server_error)?;
        }
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = report_id;
        Err(ServerFnError::new("server only"))
    }
}

#[post("/api/groups", headers: HeaderMap)]
pub async fn load_groups() -> Result<Vec<Group>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session(&headers).await.map_err(server_error)?;
        if !u.is_admin && !u.is_moderator {
            return Err(server_error("Admin or moderator only.".into()));
        }
        let groups = run_parameterized_json::<Vec<Group>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (
                SELECT id, title, read_board, post_topics, post_replies, edit_posts, delete_posts,
                       delete_topic, move_topic, sticky_topic, close_topic,
                       manage_users, manage_forums, manage_categories, manage_bans, manage_groups, manage_settings,
                       is_moderator, is_admin
                FROM groups ORDER BY id
            ) r;",
            &[],
        )
        .await
        .map_err(server_error)?;
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
        if !user.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&user, Permission::ManageGroups)
        //     .await
        //     .map_err(server_error)?;
        let title = input.title.trim().to_string();
        run_parameterized_exec(
            "UPDATE groups SET title = $1, read_board = $2, post_topics = $3, post_replies = $4, edit_posts = $5, delete_posts = $6,
                             delete_topic = $7, move_topic = $8, sticky_topic = $9, close_topic = $10,
                             manage_users = $11, manage_forums = $12, manage_categories = $13, manage_bans = $14, manage_groups = $15, manage_settings = $16,
                             is_moderator = $17, is_admin = $18 WHERE id = $19;",
            &[
                &title as &(dyn PgBind + Sync),
                &input.read_board,
                &input.post_topics,
                &input.post_replies,
                &input.edit_posts,
                &input.delete_posts,
                &input.delete_topic,
                &input.move_topic,
                &input.sticky_topic,
                &input.close_topic,
                &input.manage_users,
                &input.manage_forums,
                &input.manage_categories,
                &input.manage_bans,
                &input.manage_groups,
                &input.manage_settings,
                &input.is_moderator,
                &input.is_admin,
                &input.group_id,
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

#[post("/api/bans", headers: HeaderMap)]
pub async fn load_bans() -> Result<Vec<Ban>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session(&headers).await.map_err(server_error)?;
        if !u.is_admin && !u.is_moderator {
            return Err(server_error("Admin or moderator only.".into()));
        }
        let bans = run_parameterized_json::<Vec<Ban>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT id, username, email, ip, message, created_at, expires_at FROM bans ORDER BY created_at DESC) r;",
            &[],
        )
        .await
        .map_err(server_error)?;
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
        if !user.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&user, Permission::ManageBans)
        //     .await
        //     .map_err(server_error)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let expires = input.duration_days.map(|d| now + (d as i64) * 86400);
        let username = input.username.trim().to_string();
        let email = input.email.trim().to_lowercase();
        let message = input.message.trim().to_string();
        run_parameterized_exec(
            "INSERT INTO bans (username, email, message, created_at, expires_at) VALUES ($1, $2, $3, $4, $5);",
            &[&username as &(dyn PgBind + Sync), &email, &message, &now, &expires],
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

#[post("/api/remove-ban", headers: HeaderMap)]
pub async fn remove_ban(ban_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;
        if !user.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&user, Permission::ManageBans)
        //     .await
        //     .map_err(server_error)?;
        run_parameterized_exec(
            "DELETE FROM bans WHERE id = $1;",
            &[&ban_id as &(dyn PgBind + Sync)],
        )
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

#[post("/api/admin/test-smtp", headers: HeaderMap)]
pub async fn test_smtp_settings(input: TestSmtpForm) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let u = require_session_csrf(&headers).await.map_err(server_error)?;
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageSettings)
        //     .await
        //     .map_err(server_error)?;

        let config = run_parameterized_json::<Option<serde_json::Value>>(
            "SELECT COALESCE((SELECT row_to_json(m) FROM (SELECT smtp_enable, smtp_host, smtp_port, smtp_user, smtp_pass, smtp_from_email, smtp_from_name FROM board_meta LIMIT 1) m), 'null'::json);",
            &[],
        )
        .await
        .map_err(server_error)?;

        let config = config.ok_or_else(|| server_error("No SMTP configuration found.".into()))?;

        let enabled = config
            .get("smtp_enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !enabled {
            return Err(server_error("Email sending is not enabled.".into()));
        }

        let host = config
            .get("smtp_host")
            .and_then(|v| v.as_str())
            .unwrap_or("");
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
        let from_email = config
            .get("smtp_from_email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let from_name = config
            .get("smtp_from_name")
            .and_then(|v| v.as_str())
            .unwrap_or("FluxBB")
            .to_string();

        if host.is_empty() || from_email.is_empty() {
            return Err(server_error(
                "SMTP host and from email are required.".into(),
            ));
        }

        use lettre::{
            message::header::ContentType, transport::smtp::authentication::Credentials,
            AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
        };

        let from = format!("{} <{}>", from_name, from_email);
        let to = input.test_email.trim().to_string();

        let email = Message::builder()
            .from(from.parse().map_err(|e| server_error(format!("Invalid from address: {e}")))?)
            .to(format!("Test Recipient <{}>", to).parse().map_err(|e| server_error(format!("Invalid to address: {e}")))?)
            .subject("SMTP Test - FluxBB Forum")
            .header(ContentType::TEXT_PLAIN)
            .body(format!(
                "Hello,\n\nThis is a test email from your FluxBB forum.\n\nIf you received this message, your SMTP settings are working correctly.\n\nBoard: {}\n\nRegards,\n{}",
                from_name,
                from_name
            ))
            .map_err(|e| server_error(format!("Failed to build email: {e}")))?;

        let creds = Credentials::new(user_smtp, pass);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| server_error(format!("Failed to create mailer: {e}")))?
            .port(port)
            .credentials(creds)
            .build();

        mailer
            .send(email)
            .await
            .map_err(|e| server_error(format!("Failed to send email: {e}")))?;

        Ok(format!("Test email sent successfully to {}.", to))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}
