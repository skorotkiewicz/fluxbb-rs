use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::AppData,
};

#[component]
pub fn Profile(id: i32) -> Element {
    let board = use_context::<AppData>();

    let Some(user) = board.user(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "User not found".to_string(),
                    body: "This user does not exist.".to_string(),
                }
            }
        };
    };

    let user_topics: Vec<_> = board.topics.iter().filter(|t| t.author_id == id).cloned().collect();
    let user_posts: Vec<_> = board.posts.iter().filter(|p| p.author_id == id).cloned().collect();

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: user.title.clone(),
                    title: user.username.clone(),
                    subtitle: user.about.clone(),
                }
            }

            div { class: "user-grid",
                article { class: "user-card",
                    h3 { "Details" }
                    p { class: "user-status", "{user.status}" }
                    if !user.location.is_empty() {
                        p { class: "user-meta", "Location: {user.location}" }
                    }
                    p { class: "user-meta", "Joined: {user.joined_at}" }
                    p { class: "user-meta", "Posts: {user.post_count}" }
                    p { class: "user-meta", "Last seen: {user.last_seen}" }
                }

                article { class: "user-card",
                    h3 { "Activity" }
                    p { class: "user-meta", "Topics started: {user_topics.len()}" }
                    p { class: "user-meta", "Posts written: {user_posts.len()}" }
                }
            }

            if !user_topics.is_empty() {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Recent topics" }
                    }
                    div { class: "search-results",
                        for topic in user_topics.iter().take(10) {
                            div { class: "search-result-row",
                                div { class: "search-result-copy",
                                    Link { class: "topic-link", to: crate::Route::Topic { id: topic.id }, "{topic.subject}" }
                                    p { class: "topic-meta", "{topic.created_at}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
