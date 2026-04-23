use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, TopicStatusBadge},
    data::{load_forum_data, ForumData, SessionUser},
    Route,
};

#[component]
pub fn Forum(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let mut refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_forum_data(id).await
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
    }
        
};

    let forum = data.forum.clone();
    let topics = data.topics.clone();
    let users: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.users.iter().map(|u| (u.id, u.clone())).collect();

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
                            if let Some(author) = users.get(&topic.author_id) {
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
                                    p { class: "topic-metric", "{topic.reply_count()}" }
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
