use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;

#[cfg(feature = "server")]
use super::{
    db::{
        run_exec, run_parameterized_json, run_parameterized_scalar_i64, run_scalar_i64,
        server_error, sql_literal,
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
        let _ = run_exec(&format!(
            "UPDATE conversation_participants SET is_deleted = false, last_read_at = 0
             WHERE conversation_id = {} AND user_id IN ({}, {});",
            conv_id, user1_id, user2_id
        ))
        .await;
        return Ok(conv_id);
    }

    // Create new conversation
    let conv_id = run_scalar_i64(&format!(
        "INSERT INTO conversations (subject, created_at, updated_at, last_message_at)
         VALUES ({}, {}, {}, {}) RETURNING id;",
        sql_literal(subject),
        now,
        now,
        now
    ))
    .await? as i32;

    // Add both participants
    run_exec(&format!(
        "INSERT INTO conversation_participants (conversation_id, user_id, joined_at, last_read_at)
         VALUES ({}, {}, {}, 0), ({}, {}, {}, 0);",
        conv_id, user1_id, now, conv_id, user2_id, now
    ))
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
        let is_participant: i64 = run_scalar_i64(&format!(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = {} AND user_id = {} AND is_deleted = false;",
            id, user.id
        ))
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
        let _ = run_exec(&format!(
            "UPDATE conversation_participants SET last_read_at = {}
             WHERE conversation_id = {} AND user_id = {};",
            now, id, user.id
        ))
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
        let recipient_id: i64 = run_scalar_i64(&format!(
            "SELECT COALESCE((SELECT id FROM users WHERE LOWER(username) = LOWER({}) LIMIT 1), 0);",
            sql_literal(&form.recipient_username)
        ))
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
        run_exec(&format!(
            "INSERT INTO messages (conversation_id, sender_id, body, created_at)
             VALUES ({}, {}, {}, {});",
            conversation_id,
            user.id,
            sql_literal(&form.body),
            now
        ))
        .await
        .map_err(server_error)?;

        // Update conversation timestamps
        run_exec(&format!(
            "UPDATE conversations SET updated_at = {}, last_message_at = {} WHERE id = {};",
            now, now, conversation_id
        ))
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
        let is_participant: i64 = run_scalar_i64(&format!(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = {} AND user_id = {} AND is_deleted = false;",
            form.conversation_id, user.id
        ))
        .await
        .map_err(server_error)?;

        if is_participant == 0 {
            return Err(ServerFnError::new(
                "Conversation not found or access denied",
            ));
        }

        let now = unix_now();

        // Add the message
        run_exec(&format!(
            "INSERT INTO messages (conversation_id, sender_id, body, created_at)
             VALUES ({}, {}, {}, {});",
            form.conversation_id,
            user.id,
            sql_literal(&form.body),
            now
        ))
        .await
        .map_err(server_error)?;

        // Update conversation timestamps
        run_exec(&format!(
            "UPDATE conversations SET updated_at = {}, last_message_at = {} WHERE id = {};",
            now, now, form.conversation_id
        ))
        .await
        .map_err(server_error)?;

        // Undelete for all participants (in case someone deleted it)
        run_exec(&format!(
            "UPDATE conversation_participants SET is_deleted = false
             WHERE conversation_id = {};",
            form.conversation_id
        ))
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
        let is_participant: i64 = run_scalar_i64(&format!(
            "SELECT COUNT(*) FROM conversation_participants
             WHERE conversation_id = {} AND user_id = {};",
            id, user.id
        ))
        .await
        .map_err(server_error)?;

        if is_participant == 0 {
            return Err(ServerFnError::new("Conversation not found"));
        }

        // Soft delete for this user
        run_exec(&format!(
            "UPDATE conversation_participants SET is_deleted = true
                     WHERE conversation_id = {} AND user_id = {};",
            id, user.id
        ))
        .await
        .map_err(server_error)?;
        // Check if all participants have deleted
        let remaining: i64 = run_scalar_i64(&format!(
            "SELECT COUNT(*) FROM conversation_participants
                     WHERE conversation_id = {} AND is_deleted = false;",
            id
        ))
        .await
        .map_err(server_error)?;
        if remaining == 0 {
            // Hard delete: CASCADE wipes messages and participants automatically
            run_exec(&format!("DELETE FROM conversations WHERE id = {};", id))
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
