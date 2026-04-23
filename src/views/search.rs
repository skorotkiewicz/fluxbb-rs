use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, TopicStatusBadge},
    data::AppData,
    Route,
};

#[component]
pub fn Search() -> Element {
    let board = use_context::<AppData>();
    let mut query = use_signal(|| "migration".to_string());
    let board_for_search = board.clone();
    let results = use_memo(move || board_for_search.search(&query()));
    let matches = results().clone();

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Search".to_string(),
                    title: "Topic and member search".to_string(),
                    subtitle: "This is local filtering over the loaded board data for now. Full-text Postgres search can replace it in a later backend slice.".to_string(),
                }

                div { class: "search-toolbar",
                    input {
                        class: "search-input",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                        placeholder: "Search topics, tags, people, or migration notes",
                    }
                }
            }

            div { class: "search-grid",
                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Topic matches" }
                        p { "{matches.topics.len()} result(s)" }
                    }

                    if matches.topics.is_empty() {
                        EmptyState {
                            title: "No topic matches".to_string(),
                            body: "Try searching for postgres, auth, theme, or migration.".to_string(),
                        }
                    } else {
                        div { class: "search-results",
                            for topic in matches.topics {
                                div { class: "search-result-row",
                                    TopicStatusBadge { status: topic.status.clone() }
                                    div { class: "search-result-copy",
                                        Link { class: "topic-link", to: Route::Topic { id: topic.id }, "{topic.subject}" }
                                        p { class: "topic-meta", "{topic.tags.join(\" | \")} | updated {topic.updated_at}" }
                                    }
                                }
                            }
                        }
                    }
                }

                article { class: "panel",
                    div { class: "panel-heading",
                        h3 { "Member matches" }
                        p { "{matches.users.len()} result(s)" }
                    }

                    if matches.users.is_empty() {
                        EmptyState {
                            title: "No member matches".to_string(),
                            body: "Search by username, role, location, or profile summary.".to_string(),
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
