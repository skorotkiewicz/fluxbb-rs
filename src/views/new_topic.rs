use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, StatusMessage},
    data::{clean_error, create_topic, load_forum_data, NewTopicForm, SessionUser},
    Route,
};

#[component]
pub fn NewTopic(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_forum_data(id, 1).await
    });

    let Some(resource) = data_resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading forum…".to_string(),
                    body: "Preparing the new topic form.".to_string(),
                }
            }
        };
    };

    let Ok(data) = resource else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Forum unavailable".to_string(),
                    body: "The requested forum could not be loaded.".to_string(),
                }
            }
        };
    };

    let forum = data.forum.clone();

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
                StatusMessage { message: status(), is_error: is_error() }

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
                        let s = subject().trim().to_string();
                        let m = message().trim().to_string();

                        let validation = if s.is_empty() {
                            "Subject is required."
                        } else if s.len() > 70 {
                            "Subject must be at most 70 characters."
                        } else if m.is_empty() {
                            "Message body is required."
                        } else {
                            ""
                        };

                        if !validation.is_empty() {
                            is_error.set(true);
                            status.set(validation.to_string());
                            return;
                        }

                        let form = NewTopicForm {
                            forum_id: id,
                            subject: s,
                            message: m,
                        };
                        spawn(async move {
                            submitting.set(true);
                            match create_topic(form).await {
                                Ok(result) => {
                                    refresh.set(());
                                    navigator
                                        .push(Route::TopicPage {
                                            id: result.topic_id,
                                            page: 1,
                                        });
                                }
                                Err(e) => {
                                    is_error.set(true);
                                    status.set(clean_error(e));
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
