use dioxus::prelude::*;

use crate::components::SectionHeader;

#[component]
pub fn Rules() -> Element {
    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Community".to_string(),
                    title: "Forum rules".to_string(),
                    subtitle: "The guidelines that keep discussions constructive.".to_string(),
                }
            }

            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "Rules" }
                }
                div { class: "help-content",
                    ol {
                        li { "Be respectful. No personal attacks, harassment, or hate speech." }
                        li { "Stay on topic. Post in the appropriate forum and thread." }
                        li {
                            "No spam. Unsolicited advertising and repeated off-topic posts are not allowed."
                        }
                        li { "Use descriptive subject lines so others can find and follow your topic." }
                        li {
                            "Search before posting. Duplicates clutter the forum and split conversation."
                        }
                        li { "Do not share private information—yours or anyone else's." }
                        li { "Moderators may edit, move, or remove posts that break these rules." }
                    }
                }
            }
        }
    }
}
