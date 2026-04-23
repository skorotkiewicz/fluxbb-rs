use dioxus::prelude::*;

use crate::components::SectionHeader;

#[component]
pub fn Admin() -> Element {
    let admin_sections = [
        (
            "Structure",
            "Categories, forums, and route parity for the public board.",
        ),
        (
            "Users",
            "Member groups, moderation roles, and future auth/session controls.",
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
                    title: "Migration control panel".to_string(),
                    subtitle: "This view mirrors FluxBB's admin surface at a high level so the new application structure has a place for settings and moderation tooling.".to_string(),
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
