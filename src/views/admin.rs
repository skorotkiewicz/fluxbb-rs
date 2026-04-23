use dioxus::prelude::*;

use crate::{
    components::SectionHeader,
    data::SessionUser,
};

#[component]
pub fn Admin() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();

    let is_admin = current_user()
        .as_ref()
        .is_some_and(|u| u.group_id == 1);

    if !is_admin {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Access denied" }
                    p { "You must be an administrator to view this page." }
                }
            }
        };
    }

    let admin_sections = [
        (
            "Structure",
            "Manage categories, forums, and board structure.",
        ),
        (
            "Users",
            "Member groups, moderation roles, and account management.",
        ),
        (
            "Moderation",
            "Reports, bans, topic state changes, and queue handling.",
        ),
        ("Appearance", "Theme packs, board copy, and layout tuning."),
    ];

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Admin".to_string(),
                    title: "Control panel".to_string(),
                    subtitle: "Board administration and moderation tools.".to_string(),
                }
            }

            div { class: "admin-grid",
                for (title, body) in admin_sections {
                    article { class: "admin-card",
                        h3 { "{title}" }
                        p { "{body}" }
                    }
                }
            }
        }
    }
}
