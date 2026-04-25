use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;
#[cfg(feature = "server")]
use serde::Deserialize;

#[cfg(feature = "server")]
use super::{
    db::{run_exec, run_json_query, run_scalar_i64, server_error, sql_literal},
    security::{require_session, require_session_csrf, unix_now},
    // security::{check_permission, require_session, require_session_csrf, unix_now, Permission},
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
        let data = run_json_query::<AdminData>(
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
        run_exec(&format!(
            "INSERT INTO categories (name, description, sort_order) VALUES ({}, {}, (SELECT COALESCE(MAX(sort_order),0)+1 FROM categories));",
            sql_literal(input.name.trim()),
            sql_literal(input.description.trim())
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
        run_exec(&format!(
            "INSERT INTO forums (category_id, name, description, sort_order) VALUES ({}, {}, {}, (SELECT COALESCE(MAX(sort_order),0)+1 FROM forums WHERE category_id={}));",
            input.category_id,
            sql_literal(input.name.trim()),
            sql_literal(input.description.trim()),
            input.category_id
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageForums)
        //     .await
        //     .map_err(server_error)?;
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
        run_exec(&format!(
            "UPDATE categories SET name = {}, description = {}, sort_order = {} WHERE id = {};",
            sql_literal(input.name.trim()),
            sql_literal(input.description.trim()),
            input.sort_order,
            input.id,
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
        run_exec(&format!(
            "UPDATE forums SET category_id = {}, name = {}, description = {}, sort_order = {} WHERE id = {};",
            input.category_id,
            sql_literal(input.name.trim()),
            sql_literal(input.description.trim()),
            input.sort_order,
            input.id,
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageUsers)
        //     .await
        //     .map_err(server_error)?;
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
        if !u.is_admin {
            return Err(server_error("Admin only.".into()));
        }
        // check_permission(&u, Permission::ManageSettings)
        //     .await
        //     .map_err(server_error)?;
        run_exec(&format!(
            "UPDATE board_meta SET title = {}, tagline = {}, announcement_title = {}, announcement_body = {}, smtp_host = {}, smtp_port = {}, smtp_user = {}, smtp_pass = {}, smtp_from_email = {}, smtp_from_name = {}, smtp_enable = {} WHERE id = 1;",
            sql_literal(input.title.trim()),
            sql_literal(input.tagline.trim()),
            sql_literal(input.announcement_title.trim()),
            sql_literal(input.announcement_body.trim()),
            sql_literal(input.smtp_host.trim()),
            input.smtp_port,
            sql_literal(input.smtp_user.trim()),
            sql_literal(input.smtp_pass.trim()),
            sql_literal(input.smtp_from_email.trim()),
            sql_literal(input.smtp_from_name.trim()),
            if input.smtp_enable { "true" } else { "false" }
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
        let deleted = run_scalar_i64(
            "WITH deleted AS (DELETE FROM forum_sessions WHERE expires_at < EXTRACT(EPOCH FROM now())::bigint RETURNING *) SELECT COUNT(*) FROM deleted;"
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
        if input.reason.trim().is_empty() {
            return Err(server_error(
                "Please provide a reason for the report.".into(),
            ));
        }
        let now = unix_now();
        run_exec(&format!(
            "INSERT INTO reports (post_id, reporter_id, reason, created_at) VALUES ({}, {}, {}, {});",
            input.post_id,
            user.id,
            sql_literal(input.reason.trim()),
            now,
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
        run_exec(&format!(
            "UPDATE reports SET zapped = true WHERE id = {};",
            report_id
        ))
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
        run_exec(&format!(
            "UPDATE reports SET zapped = true WHERE id = {};",
            report_id
        ))
        .await
        .map_err(server_error)?;
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
                WHERE p.id = (SELECT post_id FROM reports WHERE id = {})
            ) r;",
            report_id
        ))
        .await
        .map_err(server_error)?;
        if info.is_first {
            run_exec(&format!(
                "DELETE FROM posts WHERE topic_id = {};",
                info.topic_id
            ))
            .await
            .map_err(server_error)?;
            run_exec(&format!("DELETE FROM topics WHERE id = {};", info.topic_id))
                .await
                .map_err(server_error)?;
        } else {
            run_exec(&format!(
                "DELETE FROM posts WHERE id = (SELECT post_id FROM reports WHERE id = {});",
                report_id
            ))
            .await
            .map_err(server_error)?;
            run_exec(&format!(
                "UPDATE users SET post_count = GREATEST(post_count - 1, 0) WHERE id = {};",
                info.author_id
            ))
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

#[post("/api/groups")]
pub async fn load_groups() -> Result<Vec<Group>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let groups = run_json_query::<Vec<Group>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (
                SELECT id, title, read_board, post_topics, post_replies, edit_posts, delete_posts,
                       delete_topic, move_topic, sticky_topic, close_topic,
                       manage_users, manage_forums, manage_categories, manage_bans, manage_groups, manage_settings,
                       is_moderator, is_admin
                FROM groups ORDER BY id
            ) r;",
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
        run_exec(&format!(
            "UPDATE groups SET title = {title}, read_board = {rb}, post_topics = {pt}, post_replies = {pr}, edit_posts = {ep}, delete_posts = {dp},
                             delete_topic = {dt}, move_topic = {mt}, sticky_topic = {st}, close_topic = {ct},
                             manage_users = {mu}, manage_forums = {mf}, manage_categories = {mc}, manage_bans = {mb}, manage_groups = {mg}, manage_settings = {ms},
                             is_moderator = {im}, is_admin = {ia} WHERE id = {gid};",
            title = sql_literal(input.title.trim()),
            rb = input.read_board,
            pt = input.post_topics,
            pr = input.post_replies,
            ep = input.edit_posts,
            dp = input.delete_posts,
            dt = input.delete_topic,
            mt = input.move_topic,
            st = input.sticky_topic,
            ct = input.close_topic,
            mu = input.manage_users,
            mf = input.manage_forums,
            mc = input.manage_categories,
            mb = input.manage_bans,
            mg = input.manage_groups,
            ms = input.manage_settings,
            im = input.is_moderator,
            ia = input.is_admin,
            gid = input.group_id,
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

#[post("/api/bans")]
pub async fn load_bans() -> Result<Vec<Ban>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let bans = run_json_query::<Vec<Ban>>(
            "SELECT COALESCE(json_agg(row_to_json(r)), '[]'::json) FROM (SELECT id, username, email, ip, message, created_at, expires_at FROM bans ORDER BY created_at DESC) r;",
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

        let config = run_json_query::<Option<serde_json::Value>>(
            "SELECT COALESCE((SELECT row_to_json(m) FROM (SELECT smtp_enable, smtp_host, smtp_port, smtp_user, smtp_pass, smtp_from_email, smtp_from_name FROM board_meta LIMIT 1) m), 'null'::json);"
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
