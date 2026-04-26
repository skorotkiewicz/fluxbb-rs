use dioxus::prelude::*;

use crate::{
    components::{ConfirmButton, EmptyState, Pagination, PollSection, PostCard, StatusMessage},
    data::{
        clean_error, create_reply, delete_topic, increment_topic_views, load_forums,
        load_topic_data, move_topic, render_paragraph, toggle_sticky, toggle_topic_status,
        MoveTopicForm, ReplyForm, SessionUser,
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
    let mut current_page = use_signal(|| page);

    // Only update signal when page actually changes
    if current_page() != page {
        current_page.set(page);
    }

    // Increment view counter once when topic loads
    use_resource(move || async move {
        let _ = increment_topic_views(id).await;
    });

    let data_resource = use_resource(move || async move {
        refresh();
        let p = current_page();
        load_topic_data(id, p).await
    });

    let Some(resource) = data_resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading topic…".to_string(),
                    body: "Fetching the discussion.".to_string(),
                }
            }
        };
    };

    let Ok(data) = resource else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Topic unavailable".to_string(),
                    body: "This discussion could not be loaded right now.".to_string(),
                }
            }
        };
    };

    let topic = data.topic.clone();
    let posts = data.posts.clone();
    let review_posts: Vec<_> = posts.iter().rev().take(3).rev().cloned().collect();
    let users: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.users.iter().map(|u| (u.id, u.clone())).collect();
    let forum = data.forum.clone();

    let mut reply_text = use_signal(String::new);
    let mut reply_status = use_signal(String::new);
    let mut reply_error = use_signal(|| false);
    let mut replying = use_signal(|| false);
    let mut move_forum_id = use_signal(|| 0_i32);

    let can_move_topic = current_user()
        .as_ref()
        .is_some_and(|u| u.move_topic || u.is_admin);
    let can_sticky_topic = current_user()
        .as_ref()
        .is_some_and(|u| u.sticky_topic || u.is_admin);
    let can_close_topic = current_user()
        .as_ref()
        .is_some_and(|u| u.close_topic || u.is_admin);
    let can_delete_topic = current_user()
        .as_ref()
        .is_some_and(|u| u.delete_topic || u.is_admin);
    let can_post_replies = current_user().as_ref().is_some_and(|u| u.post_replies);
    let is_closed = topic.closed;

    let total_pages = ((data.total_posts + data.per_page - 1) / data.per_page).max(1);
    let current_page = data.page;
    let prev_route = (current_page > 1).then(|| Route::TopicPage {
        id,
        page: current_page - 1,
    });
    let next_route = (current_page < total_pages).then(|| Route::TopicPage {
        id,
        page: current_page + 1,
    });
    let page_routes = (1..=total_pages)
        .map(|page_number| {
            (
                page_number,
                Route::TopicPage {
                    id,
                    page: page_number,
                },
            )
        })
        .collect();

    let forums_resource = use_resource(move || async move {
        if can_move_topic {
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

                if can_move_topic || can_sticky_topic || can_close_topic || can_delete_topic {
                    div { class: "post-actions",
                        if can_move_topic && !forums.is_empty() {
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
                                    spawn(async move {
                                        if move_topic(MoveTopicForm {
                                                topic_id: tid,
                                                forum_id: fid,
                                            })
                                            .await
                                            .is_ok()
                                        {
                                            navigator
                                                .push(Route::ForumPage {
                                                    id: fid,
                                                    page: 1,
                                                });
                                        }
                                    });
                                },
                                "Move"
                            }
                        }
                        if can_sticky_topic {
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
                        }
                        if can_close_topic {
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
                        }
                        if can_delete_topic {
                            ConfirmButton {
                                label: "Delete topic",
                                class: "danger-button small-button",
                                on_confirm: move |_| {
                                    let tid = id;
                                    let fid = forum_id;
                                    spawn(async move {
                                        if delete_topic(tid).await.is_ok() {
                                            navigator
                                                .push(Route::ForumPage {
                                                    id: fid,
                                                    page: 1,
                                                });
                                        }
                                    });
                                },
                            }
                        }
                    }
                }
            }

            PollSection {
                topic_id: id,
                author_id: topic.author_id,
                current_user: current_user(),
                refresh,
            }

            for post in posts {
                if let Some(author) = users.get(&post.author_id) {
                    PostCard {
                        author_name: author.username.clone(),
                        author_role: author.title.clone(),
                        author_id: author.id,
                        post: post.clone(),
                        current_user: current_user().clone(),
                        forum_id,
                    }
                }
            }

            Pagination {
                current_page,
                total_pages,
                prev_route,
                next_route,
                page_routes,
            }

            if can_post_replies && !is_closed {
                if total_pages > 1 && !review_posts.is_empty() {
                    article { class: "panel topic-review",
                        div { class: "panel-heading",
                            h4 { "Topic review" }
                            p { class: "panel-meta", "Last {review_posts.len()} posts" }
                        }
                        for post in review_posts {
                            if let Some(author) = users.get(&post.author_id) {
                                div { class: "review-post",
                                    div { class: "review-post-header",
                                        Link {
                                            to: Route::Profile { id: author.id },
                                            class: "review-author",
                                            "{author.username}"
                                        }
                                        span { class: "review-date", "{post.posted_at}" }
                                    }
                                    for line in post.body.clone() {
                                        p {
                                            class: "review-body",
                                            dangerous_inner_html: render_paragraph(&line),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                article { class: "form-card",
                    h3 { "Post a reply" }

                    StatusMessage { message: reply_status(), is_error: reply_error() }

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
                    p { class: "form-hint", "Tip: After posting, click 'Edit' to add attachments." }
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
