use dioxus::prelude::*;

use crate::components::StatusMessage;
use crate::data::{
    cast_vote, close_poll, create_poll, delete_poll, get_poll_with_user_vote, CastVoteForm,
    CreatePollForm, Poll, PollOption, SessionUser,
};

/// Main poll section component - handles both creation and display
#[component]
pub fn PollSection(
    topic_id: i32,
    author_id: i32,
    current_user: Option<SessionUser>,
    refresh: Signal<()>,
) -> Element {
    let mut poll_resource = use_resource(move || async move {
        let result: Option<(Poll, Vec<PollOption>, Option<i32>)> =
            get_poll_with_user_vote(topic_id).await.ok().flatten();
        result
    });

    let can_create_poll = current_user
        .as_ref()
        .map(|u| u.id == author_id || u.is_admin || u.is_moderator)
        .unwrap_or(false);
    let can_manage_poll = current_user
        .as_ref()
        .map(|u| u.id == author_id || u.is_admin || u.is_moderator)
        .unwrap_or(false);

    let mut creating = use_signal(|| false);

    rsx! {
        div { class: "poll-section",
            match poll_resource() {
                None => rsx! {
                    div { class: "poll-loading", "Loading poll…" }
                },
                Some(None) => {
                    if creating() {
                        rsx! {
                            PollCreator {
                                topic_id,
                                on_created: move |_| {
                                    creating.set(false);
                                    refresh.set(());
                                    poll_resource.restart();
                                },
                                on_cancel: move |_| {
                                    creating.set(false);
                                },
                            }
                        }
                    } else if can_create_poll {
                        rsx! {
                            button { class: "small-button", onclick: move |_| creating.set(true), "Create poll" }
                        }
                    } else {
                        rsx! {}
                    }
                }
                Some(Some((poll, options, user_vote))) => {
                    rsx! {
                        PollDisplay {
                            poll: poll.clone(),
                            options: options.clone(),
                            user_vote,
                            can_manage_poll,
                            current_user: current_user.clone(),
                            on_vote: move |_| {
                                refresh.set(());
                                poll_resource.restart();
                            },
                            on_close: move |_| {
                                refresh.set(());
                                poll_resource.restart();
                            },
                            on_delete: move |_| {
                                refresh.set(());
                                poll_resource.restart();
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Poll creation form component
#[component]
fn PollCreator(
    topic_id: i32,
    on_created: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut question = use_signal(String::new);
    let mut options = use_signal(|| vec!["".to_string(), "".to_string()]);
    let mut duration_hours = use_signal(|| 168_i32);
    let mut submitting = use_signal(|| false);
    let mut error = use_signal(String::new);

    rsx! {
        article { class: "form-card poll-creator",
            h4 { "Create Poll" }

            if !error().is_empty() {
                StatusMessage { message: error(), is_error: true }
            }

            label {
                "Question"
                input {
                    class: "text-input",
                    value: question(),
                    oninput: move |e| question.set(e.value()),
                    placeholder: "Ask a question…",
                }
            }

            div { class: "poll-options",
                label { "Options" }
                for (idx, opt) in options().iter().enumerate() {
                    div { class: "poll-option-row",
                        input {
                            class: "text-input",
                            value: opt.clone(),
                            oninput: move |e| {
                                let mut new_opts = options();
                                new_opts[idx] = e.value();
                                options.set(new_opts);
                            },
                            placeholder: format!("Option {}", idx + 1),
                        }
                        if options().len() > 2 {
                            button {
                                class: "icon-button danger",
                                onclick: move |_| {
                                    let mut new_opts = options();
                                    new_opts.remove(idx);
                                    options.set(new_opts);
                                },
                                "×"
                            }
                        }
                    }
                }
                button {
                    class: "small-button",
                    onclick: move |_| {
                        let mut new_opts = options();
                        new_opts.push(String::new());
                        options.set(new_opts);
                    },
                    "Add option"
                }
            }

            div { class: "poll-duration",
                label { "Duration (hours, 0 = no expiration)" }
                input {
                    class: "text-input",
                    r#type: "number",
                    value: "{duration_hours()}",
                    min: "0",
                    max: "720",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<i32>() {
                            duration_hours.set(val.max(0).min(720));
                        }
                    },
                }
            }

            div { class: "form-actions",
                button {
                    class: "small-button",
                    disabled: submitting(),
                    onclick: move |_| {
                        let trimmed_opts: Vec<String> = options()
                            .iter()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        if question().trim().is_empty() {
                            error.set("Question is required.".into());
                            return;
                        }
                        if trimmed_opts.len() < 2 {
                            error.set("At least 2 non-empty options are required.".into());
                            return;
                        }

                        let form = CreatePollForm {
                            topic_id,
                            question: question().trim().to_string(),
                            options: trimmed_opts,
                            allow_multiple: false,
                            allow_change: false,
                            duration_hours: if duration_hours() > 0 {
                                Some(duration_hours())
                            } else {
                                None
                            },
                        };
                        submitting.set(true);
                        error.set(String::new());
                        spawn(async move {
                            match create_poll(form).await {
                                Ok(_) => on_created.call(()),
                                Err(e) => {
                                    error.set(format!("Failed to create poll: {}", e));
                                    submitting.set(false);
                                }
                            }
                        });
                    },
                    if submitting() {
                        "Creating…"
                    } else {
                        "Create poll"
                    }
                }
                button {
                    class: "small-button secondary",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
            }
        }
    }
}

/// Poll display with voting and results
#[component]
fn PollDisplay(
    poll: Poll,
    options: Vec<PollOption>,
    user_vote: Option<i32>,
    can_manage_poll: bool,
    current_user: Option<SessionUser>,
    on_vote: EventHandler<()>,
    on_close: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let is_closed = poll.is_closed;
    let has_voted = user_vote.is_some();
    let show_results = has_voted || is_closed;
    let can_vote = !is_closed && current_user.is_some() && !has_voted;
    let total_votes = options.iter().map(|o| o.vote_count).sum::<i32>().max(1);
    let mut submitting = use_signal(|| false);
    let nav = use_navigator();

    rsx! {
        article { class: "poll-display",
            div { class: "poll-header",
                h4 { "{poll.question}" }
                if is_closed {
                    span { class: "badge badge-closed", "Closed" }
                }
            }

            div { class: "poll-options-display",
                if show_results {
                    // Results view
                    for opt in options.clone() {
                        PollResultBar {
                            opt: opt.clone(),
                            user_vote,
                            total_votes,
                        }
                    }
                    p { class: "poll-total", "Total votes: {total_votes}" }
                } else if can_vote {
                    // Voting view - simple button for each option
                    for opt in options.clone() {
                        button {
                            class: "poll-vote-button",
                            disabled: submitting(),
                            onclick: move |_| {
                                submitting.set(true);
                                let form = CastVoteForm {
                                    poll_id: poll.id,
                                    option_id: opt.id,
                                };
                                spawn(async move {
                                    let _ = cast_vote(form).await;
                                    submitting.set(false);
                                    on_vote.call(());
                                });
                            },
                            "{opt.option_text}"
                        }
                    }
                }
            }

            if can_manage_poll {
                div { class: "poll-actions",
                    if !is_closed {
                        button {
                            class: "small-button",
                            onclick: move |_| {
                                let poll_id = poll.id;
                                spawn(async move {
                                    let _ = close_poll(poll_id).await;
                                    on_close.call(());
                                });
                            },
                            "Close poll"
                        }
                    }
                    button {
                        class: "danger-button small-button",
                        onclick: move |_| {
                            let poll_id = poll.id;
                            spawn(async move {
                                let _ = delete_poll(poll_id).await;
                                on_delete.call(());
                            });
                        },
                        "Delete poll"
                    }
                }
            }

            if current_user.is_none() && !is_closed {
                div { class: "poll-login-prompt",
                    button {
                        class: "link-button",
                        onclick: move |_| {
                            nav.push(crate::Route::Login {});
                        },
                        "Log in to vote"
                    }
                }
            }

            if let Some(_ends_at) = poll.ends_at {
                if !poll.is_closed {
                    p { class: "poll-meta", "Poll has an end date" }
                }
            }
        }
    }
}

/// Individual poll result bar component
#[component]
fn PollResultBar(opt: PollOption, user_vote: Option<i32>, total_votes: i32) -> Element {
    let percentage = if total_votes > 0 {
        (opt.vote_count as f32 / total_votes as f32 * 100.0) as i32
    } else {
        0
    };
    let is_user_choice = user_vote == Some(opt.id);
    let class_name = if is_user_choice {
        "poll-result poll-result-selected"
    } else {
        "poll-result"
    };

    rsx! {
        div { class: class_name,
            div { class: "poll-result-bar-container",
                div {
                    class: "poll-result-bar",
                    style: format!("width: {}%", percentage),
                }
            }
            div { class: "poll-result-text",
                span { class: "poll-option-label", "{opt.option_text}" }
                span { class: "poll-vote-count", "{opt.vote_count} votes ({percentage}%)" }
            }
        }
    }
}
