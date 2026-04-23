use dioxus::prelude::*;

use crate::{
    components::{EmptyState, PostCard, TopicStatusBadge},
    data::{create_reply, AppData, ReplyForm, SessionUser},
    Route,
};

#[component]
pub fn Topic(id: i32) -> Element {
    let board = use_context::<AppData>();
    let current_user = use_context::<Signal<Option<SessionUser>>>();

    let Some(topic) = board.topic(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Topic not found".to_string(),
                    body: "The requested topic does not exist.".to_string(),
                }
            }
        };
    };

    let forum = board.forum(topic.forum_id);
    let posts = board.posts_for_topic(id);

    let mut reply_text = use_signal(String::new);
    let mut reply_status = use_signal(String::new);
    let mut reply_error = use_signal(|| false);
    let mut replying = use_signal(|| false);

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                if let Some(forum) = forum.clone() {
                    span { "/" }
                    Link { to: Route::Forum { id: forum.id }, "{forum.name}" }
                }
                span { "/" }
                span { "{topic.subject}" }
            }

            article { class: "hero-card compact-hero",
                div { class: "topic-hero-topline",
                    TopicStatusBadge { status: topic.status.clone() }
                    if !topic.tags.is_empty() {
                        p { class: "topic-tags", "{topic.tags.join(\" | \")}" }
                    }
                }
                h2 { class: "topic-title", "{topic.subject}" }
                p { class: "topic-summary", "Views: {topic.views} · Replies: {topic.reply_count(&board)} · Updated: {topic.updated_at}" }
            }

            for post in posts {
                if let Some(author) = board.user(post.author_id) {
                    PostCard {
                        author_name: author.username.clone(),
                        author_role: author.title.clone(),
                        author_id: author.id,
                        post,
                    }
                }
            }

            if current_user().is_some() {
                article { class: "form-card",
                    h3 { "Reply" }

                    if !reply_status().is_empty() {
                        p {
                            class: if reply_error() { "form-message form-error" } else { "form-message form-success" },
                            "{reply_status}"
                        }
                    }

                    label { "Message"
                        textarea {
                            class: "text-area",
                            rows: "6",
                            value: "{reply_text}",
                            oninput: move |e| reply_text.set(e.value()),
                            placeholder: "Write your reply…",
                        }
                    }
                    button {
                        class: "primary-button",
                        disabled: replying(),
                        onclick: move |_| {
                            let form = ReplyForm {
                                topic_id: id,
                                message: reply_text(),
                            };
                            spawn(async move {
                                replying.set(true);
                                match create_reply(form).await {
                                    Ok(_) => {
                                        reply_error.set(false);
                                        reply_status.set("Reply posted! Refresh to see it.".to_string());
                                        reply_text.set(String::new());
                                    }
                                    Err(e) => {
                                        reply_error.set(true);
                                        reply_status.set(e.to_string());
                                    }
                                }
                                replying.set(false);
                            });
                        },
                        if replying() { "Posting…" } else { "Post reply" }
                    }
                }
            } else {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Sign in to reply" }
                        p { "You must be logged in to post replies." }
                    }
                }
            }
        }
    }
}
