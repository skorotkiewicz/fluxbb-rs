use dioxus::prelude::*;

use crate::{
    components::SectionHeader,
    data::{load_profile_data, SessionUser},
    Route,
};

#[component]
pub fn Profile(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_profile_data(id).await
    });

    let data = if let Some(Ok(data)) = data_resource() {
        data
    } else {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Loading profile…" }
                }
            }
        };
    };

    let user = data.user.clone();
    let topics = data.topics.clone();
    let posts = data.posts.clone();

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

                    if current_user().as_ref().is_some_and(|u| u.id == id || u.group_id == 1) {
                        Link {
                            class: "small-button",
                            to: Route::ProfileEdit { id },
                            "Edit profile"
                        }
                    }
                }

                article { class: "user-card",
                    h3 { "Activity" }
                    p { class: "user-meta", "Topics started: {topics.len()}" }
                    p { class: "user-meta", "Posts written: {posts.len()}" }
                }
            }

            if !topics.is_empty() {
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Recent topics" }
                    }
                    div { class: "search-results",
                        for topic in topics.iter().take(10) {
                            div { class: "search-result-row",
                                div { class: "search-result-copy",
                                    Link {
                                        class: "topic-link",
                                        to: Route::TopicPage {
                                            id: topic.id,
                                            page: 1,
                                        },
                                        "{topic.subject}"
                                    }
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
