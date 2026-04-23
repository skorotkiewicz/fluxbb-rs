use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, TopicStatusBadge},
    data::AppData,
    Route,
};

#[component]
pub fn Search() -> Element {
    let board = use_context::<AppData>();
    let mut query = use_signal(String::new);
    let board_clone = board.clone();
    let results = use_memo(move || board_clone.search(&query()));
    let matches = results().clone();

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Search".to_string(),
                    title: "Search".to_string(),
                    subtitle: "Find topics and members across the forum.".to_string(),
                }
                div { class: "search-toolbar",
                    input {
                        class: "search-input",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                        placeholder: "Search topics, tags, or members…",
                    }
                }
            }
            div { class: "search-grid",
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Topics" }
                        p { "{matches.topics.len()} result(s)" }
                    }
                    if matches.topics.is_empty() {
                        EmptyState {
                            title: "No topics found".to_string(),
                            body: "Try a different search term.".to_string(),
                        }
                    } else {
                        div { class: "search-results",
                            for topic in matches.topics {
                                div { class: "search-result-row",
                                    TopicStatusBadge { status: topic.status.clone() }
                                    div { class: "search-result-copy",
                                        Link {
                                            class: "topic-link",
                                            to: Route::Topic { id: topic.id },
                                            "{topic.subject}"
                                        }
                                        p { class: "topic-meta",
                                            "{topic.tags.join(\" | \")} · {topic.updated_at}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Members" }
                        p { "{matches.users.len()} result(s)" }
                    }
                    if matches.users.is_empty() {
                        EmptyState {
                            title: "No members found".to_string(),
                            body: "Search by username, role, or location.".to_string(),
                        }
                    } else {
                        div { class: "search-results",
                            for user in matches.users {
                                div { class: "member-result-row",
                                    h4 { "{user.username}" }
                                    p { class: "user-title", "{user.title}" }
                                    p { class: "user-copy", "{user.about}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
