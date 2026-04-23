use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::{create_topic, AppData, NewTopicForm, SessionUser},
    Route,
};

#[component]
pub fn NewTopic(id: i32) -> Element {
    let board = use_context::<AppData>();
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let Some(forum) = board.forum(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Forum not found".to_string(),
                    body: "The requested forum does not exist.".to_string(),
                }
            }
        };
    };

    if current_user().is_none() {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Sign in required".to_string(),
                    body: "You must be signed in to start a new topic.".to_string(),
                }
            }
        };
    }

    let mut subject = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                Link { to: Route::Forum { id }, "{forum.name}" }
                span { "/" }
                span { "New topic" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "New topic".to_string(),
                    title: forum.name.clone(),
                    subtitle: format!("Post a new topic in {}", forum.name),
                }
            }

            article { class: "form-card",
                if !status().is_empty() {
                    p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                        "{status}"
                    }
                }

                label {
                    "Subject"
                    input {
                        class: "text-input",
                        value: "{subject}",
                        oninput: move |e| subject.set(e.value()),
                        placeholder: "Topic subject (max 70 characters)",
                    }
                }
                label {
                    "Message"
                    textarea {
                        class: "text-area",
                        rows: "10",
                        value: "{message}",
                        oninput: move |e| message.set(e.value()),
                        placeholder: "Write your message…",
                    }
                }
                button {
                    class: "primary-button",
                    disabled: submitting(),
                    onclick: move |_| {
                        let form = NewTopicForm {
                            forum_id: id,
                            subject: subject(),
                            message: message(),
                        };
                        spawn(async move {
                            submitting.set(true);
                            match create_topic(form).await {
                                Ok(result) => {
                                    refresh.set(());
                                    navigator.push(Route::Topic { id: result.topic_id });
                                }
                                Err(e) => {
                                    is_error.set(true);
                                    status.set(e.to_string());
                                }
                            }
                            submitting.set(false);
                        });
                    },
                    if submitting() {
                        "Posting…"
                    } else {
                        "Post topic"
                    }
                }
            }
        }
    }
}
