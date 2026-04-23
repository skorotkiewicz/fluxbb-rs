use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, TopicStatusBadge},
    data::{AppData, SessionUser},
    Route,
};

#[component]
pub fn Forum(id: i32) -> Element {
    let board = use_context::<AppData>();
    let current_user = use_context::<Signal<Option<SessionUser>>>();

    let Some(forum) = board.forum(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Forum not found".to_string(),
                    body: "The requested forum does not exist.".to_string(),
                }
            }
        };
    };

    let topics = board.topics_for_forum(id);

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

            if current_user().is_some() {
                Link { class: "primary-button", to: Route::NewTopic { id }, "New topic" }
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
                            if let Some(author) = board.user(topic.author_id) {
                                div { class: "topic-row",
                                    div { class: "topic-main",
                                        TopicStatusBadge { status: topic.status.clone() }
                                        Link {
                                            class: "topic-link",
                                            to: Route::Topic { id: topic.id },
                                            "{topic.subject}"
                                        }
                                        if !topic.tags.is_empty() {
                                            p { class: "topic-tags", "{topic.tags.join(\" | \")}" }
                                        }
                                        p { class: "topic-meta",
                                            "by {author.username} · {topic.created_at}"
                                        }
                                    }
                                    p { class: "topic-metric", "{topic.reply_count(&board)}" }
                                    p { class: "topic-metric", "{topic.views}" }
                                    p { class: "topic-metric topic-update", "{topic.updated_at}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
