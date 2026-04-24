use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::{search_server, SearchResults},
    Route,
};

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);

    let results_resource = use_resource(move || async move {
        let q = query();
        if q.trim().is_empty() {
            Ok(SearchResults::default())
        } else {
            search_server(q).await
        }
    });

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
                    }
                    if let Some(Ok(matches)) = results_resource() {
                        p { "{matches.topics.len()} result(s)" }
                        if matches.topics.is_empty() {
                            EmptyState {
                                title: "No topics found".to_string(),
                                body: if query().trim().is_empty() { "Type a search term to find topics.".to_string() } else { "Try a different search term.".to_string() },
                            }
                        } else {
                            div { class: "search-results",
                                for topic in matches.topics {
                                    div { class: "search-result-row",
                                        if topic.closed {
                                            span { class: "badge badge-closed", "Closed" }
                                        }
                                        if topic.sticky {
                                            span { class: "badge badge-pinned", "Sticky" }
                                        }
                                        div { class: "search-result-copy",
                                            Link {
                                                class: "topic-link",
                                                to: Route::TopicPage {
                                                    id: topic.id,
                                                    page: 1,
                                                },
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
                    } else {
                        EmptyState {
                            title: "Searching…".to_string(),
                            body: "".to_string(),
                        }
                    }
                }
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Members" }
                    }
                    if let Some(Ok(matches)) = results_resource() {
                        p { "{matches.users.len()} result(s)" }
                        if matches.users.is_empty() {
                            EmptyState {
                                title: "No members found".to_string(),
                                body: if query().trim().is_empty() { "Type a search term to find members.".to_string() } else { "Search by username, role, or location.".to_string() },
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
                    } else {
                        EmptyState {
                            title: "Searching…".to_string(),
                            body: "".to_string(),
                        }
                    }
                }
            }
        }
    }
}
