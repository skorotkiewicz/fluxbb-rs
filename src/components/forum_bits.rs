use dioxus::prelude::*;

use crate::data::{Post, TopicStatus};

#[component]
pub fn SectionHeader(kicker: String, title: String, subtitle: String) -> Element {
    rsx! {
        div { class: "section-header",
            p { class: "section-kicker", "{kicker}" }
            h2 { class: "section-title", "{title}" }
            p { class: "section-subtitle", "{subtitle}" }
        }
    }
}

#[component]
pub fn StatCard(label: String, value: String, detail: String) -> Element {
    rsx! {
        article { class: "stat-card",
            p { class: "stat-label", "{label}" }
            p { class: "stat-value", "{value}" }
            p { class: "stat-detail", "{detail}" }
        }
    }
}

#[component]
pub fn TopicStatusBadge(status: TopicStatus) -> Element {
    rsx! {
        span { class: status.class_name(), "{status.label()}" }
    }
}

#[component]
pub fn PostCard(author_name: String, author_role: String, post: Post) -> Element {
    rsx! {
        article { class: "post-card",
            aside { class: "post-aside",
                p { class: "post-author", "{author_name}" }
                p { class: "post-role", "{author_role}" }
                p { class: "post-timestamp", "{post.posted_at}" }
                if let Some(edited_at) = post.edited_at.clone() {
                    p { class: "post-edited", "Edited {edited_at}" }
                }
            }

            div { class: "post-body",
                for paragraph in post.body {
                    p { "{paragraph}" }
                }

                if let Some(signature) = post.signature {
                    p { class: "post-signature", "{signature}" }
                }
            }
        }
    }
}

#[component]
pub fn EmptyState(title: String, body: String) -> Element {
    rsx! {
        article { class: "empty-state",
            h3 { "{title}" }
            p { "{body}" }
        }
    }
}
