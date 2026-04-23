use dioxus::prelude::*;

use crate::data::{delete_post, Post, SessionUser, TopicStatus};
use crate::Route;

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
pub fn PostCard(
    author_name: String,
    author_role: String,
    author_id: i32,
    post: Post,
    current_user: Option<SessionUser>,
    topic_id: i32,
) -> Element {
    let mut refresh = use_context::<Signal<()>>();
    let navigator = use_navigator();

    let can_edit = current_user
        .as_ref()
        .is_some_and(|u| u.id == author_id || u.group_id == 1);

    let post_id = post.id;

    rsx! {
        article { class: "post-card",
            aside { class: "post-aside",
                Link {
                    class: "post-author",
                    to: Route::Profile { id: author_id },
                    "{author_name}"
                }
                p { class: "post-role", "{author_role}" }
                p { class: "post-timestamp", "{post.posted_at}" }
                if let Some(edited_at) = post.edited_at.clone() {
                    p { class: "post-edited", "Edited {edited_at}" }
                }

                if can_edit {
                    div { class: "post-actions",
                        Link {
                            class: "small-button",
                            to: Route::EditPost { id: post_id },
                            "Edit"
                        }
                        button {
                            class: "danger-button small-button",
                            onclick: move |_| {
                                spawn(async move {
                                    match delete_post(post_id).await {
                                        Ok(0) => {
                                            refresh.set(());
                                        }
                                        Ok(topic_id) => {
                                            navigator
                                                .push(Route::ForumPage {
                                                    id: topic_id,
                                                    page: 1,
                                                });
                                        }
                                        Err(_) => {}
                                    }
                                });
                            },
                            "Delete"
                        }
                    }
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
