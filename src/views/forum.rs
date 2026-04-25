use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::{load_forum_data, toggle_topic_status, SessionUser}, // todo, for each forum mark_all_read
    Route,
};

#[component]
pub fn Forum(id: i32) -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.push(Route::ForumPage { id, page: 1 });
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
pub fn ForumPage(id: i32, page: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let mut refresh = use_context::<Signal<()>>();
    let mut current_page = use_signal(|| page);

    // Only update signal when page actually changes
    if current_page() != page {
        current_page.set(page);
    }

    let data_resource = use_resource(move || async move {
        refresh();
        let p = current_page();
        load_forum_data(id, p).await
    });

    let data = if let Some(Ok(data)) = data_resource() {
        data
    } else {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Loading forum…" }
                }
            }
        };
    };

    let forum = data.forum.clone();
    let topics = data.topics.clone();
    let users: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.users.iter().map(|u| (u.id, u.clone())).collect();

    let total_pages = ((data.total_topics + data.per_page - 1) / data.per_page).max(1);
    let current_page = data.page;

    let can_post_topics = current_user().as_ref().is_some_and(|u| u.post_topics);
    let can_close_topic = current_user()
        .as_ref()
        .is_some_and(|u| u.close_topic || u.is_admin);

    let new_topic_ids: std::collections::HashSet<i32> = if data.last_visit > 0 {
        topics
            .iter()
            .filter(|t| (t.activity_rank as i64) > data.last_visit)
            .map(|t| t.id)
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                span { "{forum.name}" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Forum".to_string(),
                    title: forum.name.clone(),
                    subtitle: forum.description.clone(),
                }
                p { class: "forum-moderators", "Moderators: {forum.moderators.join(\", \")}" }
            }

            if can_post_topics {
                div { class: "forum-actions",
                    Link { class: "primary-button", to: Route::NewTopic { id }, "New topic" }
                    // button {
                    //     class: "secondary-button",
                    //     onclick: move |_| {
                    //         spawn(async move {
                    //             let _ = mark_all_read().await;
                    //         });
                    //     },
                    //     "Mark all as read"
                    // }
                }
            }

            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "Topics" }
                }

                if topics.is_empty() {
                    EmptyState {
                        title: "No topics yet".to_string(),
                        body: "Be the first to start a discussion in this forum.".to_string(),
                    }
                } else {
                    div { class: "topic-table",
                        div { class: "topic-table-head",
                            span { "Topic" }
                            span { "Replies" }
                            span { "Views" }
                            span { "Last update" }
                        }

                        for topic in topics {
                            if let Some(author) = users.get(&topic.author_id) {
                                div { class: if topic.sticky { "topic-row topic-sticky" } else { "topic-row" },
                                    div { class: "topic-main",
                                        if topic.closed {
                                            span { class: "badge badge-closed", "Closed" }
                                        }
                                        if topic.sticky {
                                            span { class: "badge badge-pinned", "Sticky" }
                                        }
                                        if new_topic_ids.contains(&topic.id) {
                                            span { class: "badge badge-new", "New" }
                                        }
                                        Link {
                                            class: if new_topic_ids.contains(&topic.id) { "topic-link topic-link-new" } else { "topic-link" },
                                            to: Route::TopicPage {
                                                id: topic.id,
                                                page: 1,
                                            },
                                            "{topic.subject}"
                                        }
                                        if !topic.tags.is_empty() {
                                            p { class: "topic-tags", "{topic.tags.join(\" | \")}" }
                                        }
                                        p { class: "topic-meta",
                                            "by {author.username} · {topic.created_at}"
                                        }
                                        if can_close_topic {
                                            div { class: "topic-admin-actions",
                                                button {
                                                    class: "tiny-button",
                                                    onclick: move |_| {
                                                        let tid = topic.id;
                                                        spawn(async move {
                                                            let _ = toggle_topic_status(tid).await;
                                                            refresh.set(());
                                                        });
                                                    },
                                                    if topic.closed {
                                                        "Open"
                                                    } else {
                                                        "Close"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    p { class: "topic-metric", "{topic.reply_count}" }
                                    p { class: "topic-metric", "{topic.views}" }
                                    p { class: "topic-metric topic-update", "{topic.updated_at}" }
                                }
                            }
                        }
                    }
                }

                if total_pages > 1 {
                    nav { class: "pagination",
                        if current_page > 1 {
                            Link {
                                class: "page-button",
                                to: Route::ForumPage {
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
                                    to: Route::ForumPage { id, page: p },
                                    "{p}"
                                }
                            }
                        }
                        if current_page < total_pages {
                            Link {
                                class: "page-button",
                                to: Route::ForumPage {
                                    id,
                                    page: current_page + 1,
                                },
                                "Next →"
                            }
                        }
                    }
                }
            }
        }
    }
}
