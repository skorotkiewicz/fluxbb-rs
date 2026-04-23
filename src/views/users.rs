use dioxus::prelude::*;

use crate::{components::SectionHeader, data::load_users_data, Route};

#[component]
pub fn Users() -> Element {
    let mut refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_users_data().await
    });

    let users = match data_resource() {
        Some(Ok(u)) => u,
        _ => vec![],
    };

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Members".to_string(),
                    title: "User directory".to_string(),
                    subtitle: "Browse registered members sorted by contribution.".to_string(),
                }
            }
            div { class: "user-grid",
                for user in users {
                    article { class: "user-card",
                        div { class: "user-card-top",
                            Link {
                                class: "user-link",
                                to: Route::Profile { id: user.id },
                                "{user.username}"
                            }
                            p { class: "user-title", "{user.title}" }
                        }
                        p { class: "user-status", "{user.status}" }
                        if !user.about.is_empty() {
                            p { class: "user-copy", "{user.about}" }
                        }
                        if !user.location.is_empty() {
                            p { class: "user-meta", "From {user.location}" }
                        }
                        p { class: "user-meta", "Joined {user.joined_at} · {user.post_count} posts" }
                    }
                }
            }
        }
    }
}
