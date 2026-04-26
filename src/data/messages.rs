use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;

#[cfg(feature = "server")]
use super::{
    db::{
        run_parameterized_exec, run_parameterized_json, run_parameterized_scalar_i64, server_error,
        PgBind,
    },
    security::{check_flood, require_session_csrf, unix_now},
};
use super::{
    ComposeMessageForm, ConversationThread, InboxData, NewConversationResult, ReplyMessageForm,
};

/// Get or create a conversation between two users
#[cfg(feature = "server")]
async fn get_or_create_conversation(
    user1_id: i32,
    user2_id: i32,
    subject: &str,
) -> Result<i32, String> {
    let now = unix_now();

    // First, try to find an existing 1:1 conversation between these two users
    let sql = r#"
        SELECT cp1.conversation_id
        FROM conversation_participants cp1
        JOIN conversation_participants cp2 ON cp1.conversation_id = cp2.conversation_id
        WHERE cp1.user_id = $1 AND cp2.user_id = $2
        AND cp1.is_deleted = false AND cp2.is_deleted = false
        AND (SELECT COUNT(*) FROM conversation_participants WHERE conversation_id = cp1.conversation_id) = 2
        LIMIT 1
    "#;

    let existing = run_parameterized_scalar_i64(sql, &[&user1_id, &user2_id])
        .await
        .unwrap_or(0);

    if existing != 0 {
        let conv_id = existing as i32;

        // Undelete for both users if needed
        let _ = run_parameterized_exec(
            "UPDATE conversation_participants SET is_deleted = false, last_read_at = 0
             WHERE conversation_id = $1 AND user_id IN ($2, $3);",
            &[&conv_id as &(dyn PgBind + Sync), &user1_id, &user2_id],
        )
        .await;
        return Ok(conv_id);
    }

    // Create new conversation
    let subject = subject.to_string();
    let conv_id = run_parameterized_scalar_i64(
        "INSERT INTO conversations (subject, created_at, updated_at, last_message_at)
         VALUES ($1, $2, $2, $2) RETURNING id;",
        &[&subject as &(dyn PgBind + Sync), &now],
    )
    .await? as i32;

    run_parameterized_exec(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at, last_read_at)
         VALUES ($1, $2, $3, 0), ($1, $4, $3, 0);",
        &[&conv_id as &(dyn PgBind + Sync), &user1_id, &now, &user2_id],
    )
    .await?;

    Ok(conv_id)
}

/// Load user's inbox with conversations
#[post("/api/messages/inbox", headers: HeaderMap)]
pub async fn load_inbox() -> Result<InboxData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let headers = headers;
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        let sql = r#"
            SELECT json_build_object(
                'conversations', COALESCE((
                    SELECT json_agg(json_build_object(
                        'id', c.id,
                        'subject', c.subject,
                        'created_at', c.created_at,
                        'updated_at', c.updated_at,
                        'last_message_at', c.last_message_at,
                        'participants', COALESCE((
                            SELECT json_agg(json_build_object(
                                'user_id', u.id,
                                'username', u.username,
                                'title', u.title
                            ))
                            FROM conversation_participants cp2
                            JOIN users u ON u.id = cp2.user_id
                            WHERE cp2.conversation_id = c.id AND cp2.user_id != $1 AND cp2.is_deleted = false
                        ), '[]'::json),
                        'unread_count', COALESCE((
                            SELECT COUNT(*)::int
                            FROM messages m
                            WHERE m.conversation_id = c.id
                            AND m.sender_id != $1
                            AND m.created_at > cp.last_read_at
                        ), 0),
                        'last_message', (
                            SELECT json_build_object(
                                'id', m.id,
                                'conversation_id', m.conversation_id,
                                'sender_id', m.sender_id,
                                'sender_name', u.username,
                                'sender_title', u.title,
                                'body', m.body,
                                'created_at', m.created_at
                            )
                            FROM messages m
                            JOIN users u ON u.id = m.sender_id
                            WHERE m.conversation_id = c.id
                            ORDER BY m.created_at DESC
                            LIMIT 1
                        )
                    ) ORDER BY c.last_message_at DESC)
                    FROM conversations c
                    JOIN conversation_participants cp ON cp.conversation_id = c.id
                    WHERE cp.user_id = $1 AND cp.is_deleted = false
                ), '[]'::json),
                'total_count', (SELECT COUNT(*)::int FROM conversation_participants WHERE user_id = $1 AND is_deleted = false),
                'unread_count', COALESCE((
                    SELECT COUNT(*)::int
                    FROM conversation_participants cp
                    JOIN messages m ON m.conversation_id = cp.conversation_id
                    WHERE cp.user_id = $1
                    AND cp.is_deleted = false
                    AND m.sender_id != $1
                    AND m.created_at > cp.last_read_at
                ), 0)
            )::json;
        "#;

        let data = run_parameterized_json::<InboxData>(sql, &[&user.id])
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

/// Load a single conversation thread with messages
#[post("/api/messages/conversation/:id", headers: HeaderMap)]
pub async fn load_conversation(id: i32) -> Result<ConversationThread, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let headers = headers;
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Verify user is a participant and not deleted
        let is_participant: i64 = run_parameterized_scalar_i64(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = $1 AND user_id = $2 AND is_deleted = false;",
            &[&id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;

        if is_participant == 0 {
            return Err(ServerFnError::new(
                "Conversation not found or access denied",
            ));
        }

        let sql = r#"
            SELECT json_build_object(
                'conversation', (SELECT row_to_json(c) FROM (SELECT id, subject, created_at, updated_at, last_message_at FROM conversations WHERE id = $1) c),
                'participants', COALESCE((
                    SELECT json_agg(json_build_object(
                        'user_id', u.id,
                        'username', u.username,
                        'title', u.title
                    ))
                    FROM conversation_participants cp
                    JOIN users u ON u.id = cp.user_id
                    WHERE cp.conversation_id = $1 AND cp.is_deleted = false
                ), '[]'::json),
                'messages', COALESCE((
                    SELECT json_agg(json_build_object(
                        'id', m.id,
                        'conversation_id', m.conversation_id,
                        'sender_id', m.sender_id,
                        'sender_name', u.username,
                        'sender_title', u.title,
                        'body', m.body,
                        'created_at', m.created_at
                    ) ORDER BY m.created_at ASC)
                    FROM messages m
                    JOIN users u ON u.id = m.sender_id
                    WHERE m.conversation_id = $1
                ), '[]'::json),
                'current_user_id', $2
            )::json;
        "#;

        let data = run_parameterized_json::<ConversationThread>(sql, &[&id, &user.id])
            .await
            .map_err(server_error)?;

        // Mark as read
        let now = unix_now();
        let _ = run_parameterized_exec(
            "UPDATE conversation_participants SET last_read_at = $1
             WHERE conversation_id = $2 AND user_id = $3;",
            &[&now as &(dyn PgBind + Sync), &id, &user.id],
        )
        .await;

        Ok(data)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

/// Send a new message (creates conversation if needed)
#[post("/api/messages/send", headers: HeaderMap)]
pub async fn send_message(
    form: ComposeMessageForm,
) -> Result<NewConversationResult, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let headers = headers;
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        check_flood(user.id, user.is_admin)
            .await
            .map_err(server_error)?;

        let now = unix_now();

        // Look up recipient
        let recipient_id: i64 = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT id FROM users WHERE LOWER(username) = LOWER($1) LIMIT 1), 0);",
            &[&form.recipient_username as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if recipient_id == 0 {
            return Err(ServerFnError::new("Recipient not found"));
        }

        let recipient_id = recipient_id as i32;
        if recipient_id == user.id {
            return Err(ServerFnError::new("Cannot message yourself"));
        }

        // Get or create conversation
        let conversation_id = get_or_create_conversation(user.id, recipient_id, &form.subject)
            .await
            .map_err(server_error)?;

        // Add the message
        let body = form.body.trim().to_string();
        run_parameterized_exec(
            "INSERT INTO messages (conversation_id, sender_id, body, created_at)
             VALUES ($1, $2, $3, $4);",
            &[&conversation_id as &(dyn PgBind + Sync), &user.id, &body, &now],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE conversations SET updated_at = $1, last_message_at = $1 WHERE id = $2;",
            &[&now as &(dyn PgBind + Sync), &conversation_id],
        )
        .await
        .map_err(server_error)?;

        Ok(NewConversationResult { conversation_id })
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = form;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

/// Reply to an existing conversation
#[post("/api/messages/reply", headers: HeaderMap)]
pub async fn reply_message(form: ReplyMessageForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let headers = headers;
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        check_flood(user.id, user.is_admin)
            .await
            .map_err(server_error)?;

        // Verify user is a participant and not deleted
        let is_participant: i64 = run_parameterized_scalar_i64(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = $1 AND user_id = $2 AND is_deleted = false;",
            &[&form.conversation_id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;

        if is_participant == 0 {
            return Err(ServerFnError::new(
                "Conversation not found or access denied",
            ));
        }

        let now = unix_now();

        let body = form.body.trim().to_string();
        run_parameterized_exec(
            "INSERT INTO messages (conversation_id, sender_id, body, created_at)
             VALUES ($1, $2, $3, $4);",
            &[&form.conversation_id as &(dyn PgBind + Sync), &user.id, &body, &now],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE conversations SET updated_at = $1, last_message_at = $1 WHERE id = $2;",
            &[&now as &(dyn PgBind + Sync), &form.conversation_id],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE conversation_participants SET is_deleted = false
             WHERE conversation_id = $1;",
            &[&form.conversation_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = form;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}

/// Delete a conversation for the current user (soft delete)
#[post("/api/messages/delete/:id", headers: HeaderMap)]
pub async fn delete_conversation(id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let headers = headers;
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Verify user is a participant
        let is_participant: i64 = run_parameterized_scalar_i64(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = $1 AND user_id = $2;",
            &[&id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;

        if is_participant == 0 {
            return Err(ServerFnError::new("Conversation not found"));
        }

        run_parameterized_exec(
            "UPDATE conversation_participants SET is_deleted = true
             WHERE conversation_id = $1 AND user_id = $2;",
            &[&id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;
        let remaining: i64 = run_parameterized_scalar_i64(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = $1 AND is_deleted = false;",
            &[&id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;
        if remaining == 0 {
            run_parameterized_exec(
                "DELETE FROM conversations WHERE id = $1;",
                &[&id as &(dyn PgBind + Sync)],
            )
            .await
            .map_err(server_error)?;
        }
        Ok(())

        // // Soft delete for this user only
        // run_exec(&format!(
        //     "UPDATE conversation_participants SET is_deleted = true
        //      WHERE conversation_id = {} AND user_id = {};",
        //     id, user.id
        // ))
        // .await
        // .map_err(server_error)?;

        // Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = id;
        let _ = headers;
        Err(ServerFnError::new("server only"))
    }
}
