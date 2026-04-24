use dioxus::prelude::*;

use crate::{
    components::{EmptyState, PostCard},
    data::{
        clean_error, create_reply, delete_topic, increment_topic_views, load_forums, load_topic_data,
        move_topic, toggle_sticky, toggle_topic_status, MoveTopicForm, ReplyForm, SessionUser,
        TopicData,
    },
    Route,
};

#[component]
pub fn Topic(id: i32) -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.push(Route::TopicPage { id, page: 1 });
    });
    rsx! {
        section { class: "page",
            article { class: "empty-state",
                h3 { "Redirecting…" }
            }
        }
    }
}

#[component]
pub fn TopicPage(id: i32, page: i32) -> Element {
    let navigator = use_navigator();
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let mut refresh = use_context::<Signal<()>>();

    // Increment view counter once when topic loads
    use_resource(move || async move {
        let _ = increment_topic_views(id).await;
    });

    let data_resource = use_resource(move || async move {
        refresh();
        load_topic_data(id, page).await
    });

    let data = if let Some(Ok(data)) = data_resource() {
        data
    } else {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Loading topic…" }
                }
            }
        };
    };

    let topic = data.topic.clone();
    let posts = data.posts.clone();
    let users: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.users.iter().map(|u| (u.id, u.clone())).collect();
    let forum = data.forum.clone();

    let mut reply_text = use_signal(String::new);
    let mut reply_status = use_signal(String::new);
    let mut reply_error = use_signal(|| false);
    let mut replying = use_signal(|| false);
    let mut move_forum_id = use_signal(|| 0_i32);

    let is_admin = current_user().as_ref().is_some_and(|u| u.group_id == 1);
    let is_closed = topic.closed;

    let total_pages = ((data.total_posts + data.per_page - 1) / data.per_page).max(1);
    let current_page = data.page;

    let forums_resource = use_resource(move || async move {
        if is_admin {
            load_forums().await.unwrap_or_default()
        } else {
            Vec::new()
        }
    });

    let forum_id = forum.as_ref().map(|f| f.id).unwrap_or(0);
    let forums = forums_resource().unwrap_or_default();

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                if let Some(forum) = forum.clone() {
                    span { "/" }
                    Link {
                        to: Route::ForumPage {
                            id: forum.id,
                            page: 1,
                        },
                        "{forum.name}"
                    }
                }
                span { "/" }
                span { "{topic.subject}" }
            }

            article { class: "hero-card compact-hero",
                div { class: "topic-hero-topline",
                    if topic.closed {
                        span { class: "badge badge-closed", "Closed" }
                    }
                    if topic.sticky {
                        span { class: "badge badge-pinned", "Sticky" }
                    }
                    if !topic.tags.is_empty() {
                        p { class: "topic-tags", "{topic.tags.join(\" | \")}" }
                    }
                }
                h2 { class: "topic-title", "{topic.subject}" }
                p { class: "topic-summary",
                    "Views: {topic.views} · Replies: {topic.reply_count} · Updated: {topic.updated_at}"
                }

                if is_admin {
                    div { class: "post-actions",
                        if !forums.is_empty() {
                            select {
                                class: "small-select",
                                value: "{move_forum_id}",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<i32>() {
                                        move_forum_id.set(v);
                                    }
                                },
                                option { value: "0", "Move to forum…" }
                                for f in forums {
                                    if f.id != forum_id {
                                        option { value: "{f.id}", "{f.name}" }
                                    }
                                }
                            }
                            button {
                                class: "small-button",
                                disabled: move_forum_id() == 0,
                                onclick: move |_| {
                                let tid = id;
                                let fid = move_forum_id();
                                if fid == 0 {
                                    return;
                                }
                                let navigator = navigator.clone();
                                spawn(async move {
                                    match move_topic(MoveTopicForm {
                                        topic_id: tid,
                                        forum_id: fid,
                                    }).await
                                    {
                                        Ok(_) => {
                                            navigator
                                                .push(Route::ForumPage {
                                                    id: fid,
                                                    page: 1,
                                                });
                                        }
                                        Err(_) => {}
                                    }
                                });
                                },
                                "Move"
                            }
                        }
                        button {
                            class: "small-button",
                            onclick: move |_| {
                                let tid = id;
                                spawn(async move {
                                    let _ = toggle_sticky(tid).await;
                                    refresh.set(());
                                });
                            },
                            if topic.sticky {
                                "Unstick topic"
                            } else {
                                "Stick topic"
                            }
                        }
                        button {
                            class: "small-button",
                            onclick: move |_| {
                                let tid = id;
                                spawn(async move {
                                    let _ = toggle_topic_status(tid).await;
                                    refresh.set(());
                                });
                            },
                            if is_closed {
                                "Open topic"
                            } else {
                                "Close topic"
                            }
                        }
                        button {
                            class: "danger-button small-button",
                            onclick: move |_| {
                                let tid = id;
                                let fid = forum_id;
                                let navigator = navigator.clone();
                                spawn(async move {
                                    match delete_topic(tid).await {
                                        Ok(_) => {
                                            navigator
                                                .push(Route::ForumPage {
                                                    id: fid,
                                                    page: 1,
                                                });
                                        }
                                        Err(_) => {}
                                    }
                                });
                            },
                            "Delete topic"
                        }
                    }
                }
            }

            for post in posts {
                if let Some(author) = users.get(&post.author_id) {
                    PostCard {
                        author_name: author.username.clone(),
                        author_role: author.title.clone(),
                        author_id: author.id,
                        post: post.clone(),
                        current_user: current_user().clone(),
                        topic_id: id,
                    }
                }
            }

            if total_pages > 1 {
                nav { class: "pagination",
                    if current_page > 1 {
                        Link {
                            class: "page-button",
                            to: Route::TopicPage {
                                id,
                                page: current_page - 1,
                            },
                            "← Prev"
                        }
                    }
                    for p in 1..=total_pages {
                        if p == current_page {
                            span { class: "page-button active", "{p}" }
                        } else {
                            Link {
                                class: "page-button",
                                to: Route::TopicPage { id, page: p },
                                "{p}"
                            }
                        }
                    }
                    if current_page < total_pages {
                        Link {
                            class: "page-button",
                            to: Route::TopicPage {
                                id,
                                page: current_page + 1,
                            },
                            "Next →"
                        }
                    }
                }
            }

            if current_user().is_some() && !is_closed {
                article { class: "form-card",
                    h3 { "Post a reply" }

                    if !reply_status().is_empty() {
                        p { class: if reply_error() { "form-message form-error" } else { "form-message form-success" },
                            "{reply_status}"
                        }
                    }

                    label {
                        "Message"
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
                            let m = reply_text().trim().to_string();
                            if m.is_empty() {
                                reply_error.set(true);
                                reply_status.set("Message body is required.".to_string());
                                return;
                            }
                            let form = ReplyForm {
                                topic_id: id,
                                message: m,
                            };
                            spawn(async move {
                                replying.set(true);
                                match create_reply(form).await {
                                    Ok(_) => {
                                        reply_error.set(false);
                                        reply_status.set("Reply posted!".to_string());
                                        reply_text.set(String::new());
                                        refresh.set(());
                                    }
                                    Err(e) => {
                                        reply_error.set(true);
                                        reply_status.set(clean_error(e));
                                    }
                                }
                                replying.set(false);
                            });
                        },
                        if replying() {
                            "Posting…"
                        } else {
                            "Post reply"
                        }
                    }
                }
            } else if is_closed {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Topic closed" }
                        p { "This topic has been closed. No new replies are allowed." }
                    }
                }
            } else {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Sign in to reply" }
                        p { "You must be logged in to post replies." }
                    }
                    Link { class: "primary-button", to: Route::Login {}, "Sign in" }
                }
            }
        }
    }
}
