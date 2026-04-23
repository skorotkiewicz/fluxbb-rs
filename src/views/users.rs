use dioxus::prelude::*;

use crate::{components::SectionHeader, data::AppData};

#[component]
pub fn Users() -> Element {
    let board = use_context::<AppData>();
    let mut users = board.users.clone();
    users.sort_by_key(|user| std::cmp::Reverse(user.post_count));

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Members".to_string(),
                    title: "User directory".to_string(),
                    subtitle: "A compact member listing that stands in for FluxBB's classic user list while the profile and auth flows are still being ported.".to_string(),
                }
            }

            div { class: "user-grid",
                for user in users {
                    article { class: "user-card",
                        div { class: "user-card-top",
                            h3 { "{user.username}" }
                            p { class: "user-title", "{user.title}" }
                        }
                        p { class: "user-status", "{user.status}" }
                        p { class: "user-copy", "{user.about}" }
                        p { class: "user-meta", "Joined {user.joined_at} from {user.location}" }
                        p { class: "user-meta", "{user.post_count} posts | Last seen {user.last_seen}" }
                    }
                }
            }
        }
    }
}
