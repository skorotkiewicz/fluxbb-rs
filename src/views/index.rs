use dioxus::prelude::*;

use crate::{
    components::{SectionHeader, StatCard, TopicStatusBadge},
    data::{load_index_data, mark_all_read, SessionUser},
    Route,
};

#[component]
pub fn Index() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let mut refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_index_data().await
    });

    let data = if let Some(Ok(data)) = data_resource() {
        data
    } else {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Loading board…" }
                }
            }
        };
    };

    let stats = data.stats.clone();
    let categories = data.categories.clone();
    let forums = data.forums.clone();
    let forum_stats: std::collections::HashMap<i32, crate::data::ForumStats> =
        data.forum_stats.iter().map(|fs| (fs.forum_id, fs.clone())).collect();
    let recent_topics = data.recent_topics.clone();
    let recent_users: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.recent_users.iter().map(|u| (u.id, u.clone())).collect();

    let cat_items: Vec<_> = categories
        .iter()
        .map(|cat| {
            let cat_forums: Vec<_> = forums
                .iter()
                .filter(|f| f.category_id == cat.id)
                .cloned()
                .collect();
            (cat.clone(), cat_forums)
        })
        .filter(|(_, cat_forums)| !cat_forums.is_empty())
        .collect();

    rsx! {
        section { class: "page",
            article { class: "hero-card",
                SectionHeader {
                    kicker: "Board index".to_string(),
                    title: data.meta.announcement_title.clone(),
                    subtitle: data.meta.announcement_body.clone(),
                }

                div { class: "stat-grid",
                    StatCard {
                        label: "Members".to_string(),
                        value: stats.members.to_string(),
                        detail: format!("Newest: {}", stats.newest_member),
                    }
                    StatCard {
                        label: "Topics".to_string(),
                        value: stats.topics.to_string(),
                        detail: "Active discussions".to_string(),
                    }
                    StatCard {
                        label: "Posts".to_string(),
                        value: stats.posts.to_string(),
                        detail: "Total contributions".to_string(),
                    }
                }

                if current_user().is_some() {
                    button {
                        class: "small-button",
                        style: "margin-top: 12px; align-self: start;",
                        onclick: move |_| {
                            spawn(async move {
                                let _ = mark_all_read().await;
                                refresh.set(());
                            });
                        },
                        "Mark all as read"
                    }
                }
            }

            for (cat, cat_forums) in cat_items {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "{cat.name}" }
                        p { "{cat.description}" }
                    }

                    div { class: "forum-table",
                        div { class: "forum-table-head",
                            span { "Forum" }
                            span { "Topics" }
                            span { "Posts" }
                            span { "Last post" }
                        }

                        for forum in cat_forums {
                            if let Some(fs) = forum_stats.get(&forum.id) {
                                div { class: "forum-row",
                                    div { class: "forum-main",
                                        Link {
                                            class: "forum-link",
                                            to: Route::Forum { id: forum.id },
                                            "{forum.name}"
                                        }
                                        p { class: "forum-description", "{forum.description}" }
                                        p { class: "forum-moderators",
                                            "Moderators: {forum.moderators.join(\", \")}"
                                        }
                                    }
                                    p { class: "forum-count", "{fs.topic_count}" }
                                    p { class: "forum-count", "{fs.post_count}" }
                                    div { class: "forum-last",
                                        Link {
                                            class: "last-topic-link",
                                            to: Route::Topic {
                                                id: fs.last_topic_id,
                                            },
                                            "{fs.last_topic_subject}"
                                        }
                                        p { "{fs.last_post_author} on {fs.last_posted_at}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "Recent activity" }
                }

                div { class: "recent-list",
                    for topic in recent_topics {
                        if let Some(author) = recent_users.get(&topic.author_id) {
                            div { class: "recent-row",
                                div { class: "recent-main",
                                    TopicStatusBadge { status: topic.status.clone() }
                                    Link {
                                        class: "recent-topic-link",
                                        to: Route::Topic { id: topic.id },
                                        "{topic.subject}"
                                    }
                                }
                                p { class: "recent-meta",
                                    "by {author.username} · {topic.updated_at}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
