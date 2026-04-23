use dioxus::prelude::*;

use crate::{
    components::{SectionHeader, StatCard, TopicStatusBadge},
    data::AppData,
    Route,
};

#[component]
pub fn Index() -> Element {
    let board = use_context::<AppData>();
    let stats = board.board_stats();
    let recent_topics = board.recent_topics(4);

    rsx! {
        section { class: "page",
            article { class: "hero-card",
                SectionHeader {
                    kicker: "Board index".to_string(),
                    title: board.meta.announcement_title.clone(),
                    subtitle: board.meta.announcement_body.clone(),
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
            }

            for category in board.categories_sorted() {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "{category.name}" }
                        p { "{category.description}" }
                    }

                    div { class: "forum-table",
                        div { class: "forum-table-head",
                            span { "Forum" }
                            span { "Topics" }
                            span { "Posts" }
                            span { "Last post" }
                        }

                        for forum in board.forums_in_category(category.id) {
                            if let Some(snapshot) = board.forum_snapshot(forum.id) {
                                div { class: "forum-row",
                                    div { class: "forum-main",
                                        Link { class: "forum-link", to: Route::Forum { id: forum.id }, "{forum.name}" }
                                        p { class: "forum-description", "{forum.description}" }
                                        p { class: "forum-moderators", "Moderators: {forum.moderators.join(\", \")}" }
                                    }
                                    p { class: "forum-count", "{snapshot.topic_count}" }
                                    p { class: "forum-count", "{snapshot.post_count}" }
                                    div { class: "forum-last",
                                        Link { class: "last-topic-link", to: Route::Topic { id: snapshot.last_topic_id }, "{snapshot.last_topic_subject}" }
                                        p { "{snapshot.last_post_author} on {snapshot.last_posted_at}" }
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
                        if let Some(author) = board.user(topic.author_id) {
                            div { class: "recent-row",
                                div { class: "recent-main",
                                    TopicStatusBadge { status: topic.status.clone() }
                                    Link { class: "recent-topic-link", to: Route::Topic { id: topic.id }, "{topic.subject}" }
                                }
                                p { class: "recent-meta", "by {author.username} · {topic.updated_at}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
