use dioxus::prelude::*;
#[cfg(feature = "server")]
use http::HeaderMap;

#[cfg(feature = "server")]
use super::{
    db::{run_parameterized_exec, run_parameterized_json, run_parameterized_scalar_i64, server_error, PgBind},
    security::{require_session_csrf, unix_now},
};
use super::{CastVoteForm, CreatePollForm, Poll, PollOption};

/// Get poll data for a topic including options and user's vote
#[post("/api/topic/:topic_id/poll")]
pub async fn get_poll(
    topic_id: i32,
) -> Result<Option<(Poll, Vec<PollOption>, Option<i32>)>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let poll = run_parameterized_json::<Option<Poll>>(
            "SELECT COALESCE((SELECT row_to_json(p) FROM (SELECT id, topic_id, question, created_at, ends_at, allow_multiple, allow_change, is_closed FROM polls WHERE topic_id = $1) AS p), 'null'::json);",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let Some(poll) = poll else {
            return Ok(None);
        };

        let options = run_parameterized_json::<Vec<PollOption>>(
            "SELECT COALESCE(json_agg(row_to_json(o)), '[]'::json) FROM (SELECT id, poll_id, option_text, sort_order, vote_count FROM poll_options WHERE poll_id = $1 ORDER BY sort_order, id) AS o;",
            &[&poll.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(Some((poll, options, None)))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Err(ServerFnError::new("server only"))
    }
}

/// Get poll data for a topic with current user's vote
#[post("/api/topic/:topic_id/poll-with-vote", headers: HeaderMap)]
pub async fn get_poll_with_user_vote(
    topic_id: i32,
) -> Result<Option<(Poll, Vec<PollOption>, Option<i32>)>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        let poll = run_parameterized_json::<Option<Poll>>(
            "SELECT COALESCE((SELECT row_to_json(p) FROM (SELECT id, topic_id, question, created_at, ends_at, allow_multiple, allow_change, is_closed FROM polls WHERE topic_id = $1) AS p), 'null'::json);",
            &[&topic_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let Some(poll) = poll else {
            return Ok(None);
        };

        let options = run_parameterized_json::<Vec<PollOption>>(
            "SELECT COALESCE(json_agg(row_to_json(o)), '[]'::json) FROM (SELECT id, poll_id, option_text, sort_order, vote_count FROM poll_options WHERE poll_id = $1 ORDER BY sort_order, id) AS o;",
            &[&poll.id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let user_vote = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2), 0);",
            &[&poll.id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;

        let user_vote = if user_vote > 0 {
            Some(user_vote as i32)
        } else {
            None
        };

        Ok(Some((poll, options, user_vote)))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = topic_id;
        Err(ServerFnError::new("server only"))
    }
}

/// Create a new poll for a topic
#[post("/api/poll/create", headers: HeaderMap)]
pub async fn create_poll(input: CreatePollForm) -> Result<i32, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Verify user owns the topic or has moderator permissions
        let author_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT author_id FROM topics WHERE id = $1), 0);",
            &[&input.topic_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if author_id != user.id as i64 && !user.is_admin && !user.is_moderator {
            return Err(server_error(
                "You can only create polls in your own topics.".into(),
            ));
        }

        // Check if topic already has a poll
        let existing = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT id FROM polls WHERE topic_id = $1), 0);",
            &[&input.topic_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if existing > 0 {
            return Err(server_error("This topic already has a poll.".into()));
        }

        let question = input.question.trim();
        if question.is_empty() {
            return Err(server_error("Poll question is required.".into()));
        }

        if input.options.len() < 2 {
            return Err(server_error("At least 2 options are required.".into()));
        }

        let now = unix_now();
        let ends_at = input.duration_hours.map(|h| now + (h as i64 * 3600));
        let question_owned = question.to_string();

        run_parameterized_exec(
            "INSERT INTO polls (topic_id, question, created_at, ends_at, allow_multiple, allow_change, is_closed) 
             VALUES ($1, $2, $3, $4, $5, $6, false);",
            &[
                &input.topic_id as &(dyn PgBind + Sync),
                &question_owned,
                &now,
                &ends_at,
                &input.allow_multiple,
                &input.allow_change,
            ],
        )
        .await
        .map_err(server_error)?;

        let poll_id = run_parameterized_scalar_i64("SELECT LASTVAL();", &[])
            .await
            .map_err(server_error)?;

        for (idx, option) in input.options.iter().enumerate() {
            let opt_text = option.trim().to_string();
            if !opt_text.is_empty() {
                let _ = run_parameterized_exec(
                    "INSERT INTO poll_options (poll_id, option_text, sort_order) VALUES ($1, $2, $3);",
                    &[&poll_id as &(dyn PgBind + Sync), &opt_text, &(idx as i32)],
                )
                .await;
            }
        }

        Ok(poll_id as i32)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = input;
        Err(ServerFnError::new("server only"))
    }
}

/// Cast a vote on a poll
#[post("/api/poll/vote", headers: HeaderMap)]
pub async fn cast_vote(input: CastVoteForm) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        #[derive(serde::Deserialize)]
        struct PollCheck {
            is_closed: bool,
            ends_at: Option<i64>,
        }

        let poll = run_parameterized_json::<Option<PollCheck>>(
            "SELECT COALESCE((SELECT row_to_json(r) FROM (SELECT is_closed, ends_at FROM polls WHERE id = $1) AS r), 'null'::json);",
            &[&input.poll_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        let Some(poll) = poll else {
            return Err(server_error("Poll not found.".into()));
        };

        if poll.is_closed {
            return Err(server_error("This poll is closed.".into()));
        }

        if let Some(ends_at) = poll.ends_at {
            if unix_now() > ends_at {
                return Err(server_error("This poll has ended.".into()));
            }
        }

        let existing_vote = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT id FROM poll_votes WHERE poll_id = $1 AND user_id = $2), 0);",
            &[&input.poll_id as &(dyn PgBind + Sync), &user.id],
        )
        .await
        .map_err(server_error)?;

        if existing_vote > 0 {
            return Err(server_error("You have already voted on this poll.".into()));
        }

        let now = unix_now();
        run_parameterized_exec(
            "INSERT INTO poll_votes (poll_id, option_id, user_id, voted_at) VALUES ($1, $2, $3, $4);",
            &[&input.poll_id as &(dyn PgBind + Sync), &input.option_id, &user.id, &now],
        )
        .await
        .map_err(server_error)?;

        run_parameterized_exec(
            "UPDATE poll_options SET vote_count = vote_count + 1 WHERE id = $1;",
            &[&input.option_id as &(dyn PgBind + Sync)],
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

/// Close/remove a poll
#[post("/api/poll/:poll_id/close", headers: HeaderMap)]
pub async fn close_poll(poll_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Get topic_id and check ownership
        let topic_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT topic_id FROM polls WHERE id = $1), 0);",
            &[&poll_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if topic_id == 0 {
            return Err(server_error("Poll not found.".into()));
        }

        let author_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT author_id FROM topics WHERE id = $1), 0);",
            &[&(topic_id as i32) as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if author_id != user.id as i64 && !user.is_admin && !user.is_moderator {
            return Err(server_error(
                "You can only close polls in your own topics.".into(),
            ));
        }

        run_parameterized_exec(
            "UPDATE polls SET is_closed = true WHERE id = $1;",
            &[&poll_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = poll_id;
        Err(ServerFnError::new("server only"))
    }
}

/// Delete a poll completely
#[post("/api/poll/:poll_id/delete", headers: HeaderMap)]
pub async fn delete_poll(poll_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let user = require_session_csrf(&headers).await.map_err(server_error)?;

        // Get topic_id and check ownership
        let topic_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT topic_id FROM polls WHERE id = $1), 0);",
            &[&poll_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if topic_id == 0 {
            return Err(server_error("Poll not found.".into()));
        }

        let author_id = run_parameterized_scalar_i64(
            "SELECT COALESCE((SELECT author_id FROM topics WHERE id = $1), 0);",
            &[&(topic_id as i32) as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        if author_id != user.id as i64 && !user.is_admin && !user.is_moderator {
            return Err(server_error(
                "You can only delete polls in your own topics.".into(),
            ));
        }

        run_parameterized_exec(
            "DELETE FROM polls WHERE id = $1;",
            &[&poll_id as &(dyn PgBind + Sync)],
        )
        .await
        .map_err(server_error)?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = poll_id;
        Err(ServerFnError::new("server only"))
    }
}
