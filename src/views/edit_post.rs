use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, StatusMessage},
    data::{clean_error, edit_post, load_post, load_topic_data, EditPostForm, SessionUser},
    Route,
};

#[component]
pub fn EditPost(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let post_resource = use_resource(move || async move { load_post(id).await });

    let mut message = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut initialized = use_signal(|| false);

    // Populate form when post loads
    use_effect(move || {
        if initialized() {
            return;
        }

        if let Some(Ok(post)) = post_resource() {
            let body = post.body.join("\n\n");
            message.set(body);
            initialized.set(true);
        }
    });

    let Some(post_result) = post_resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading post…".to_string(),
                    body: "Preparing the editor.".to_string(),
                }
            }
        };
    };

    let post = match post_result {
        Ok(post) => post,
        Err(_) => {
            return rsx! {
                section { class: "page",
                    EmptyState {
                        title: "Post unavailable".to_string(),
                        body: "The post you are trying to edit could not be loaded.".to_string(),
                    }
                }
            };
        }
    };

    let topic_resource =
        use_resource(move || async move { load_topic_data(post.topic_id, 1).await });

    let Some(topic_result) = topic_resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading topic…".to_string(),
                    body: "Looking up the discussion context.".to_string(),
                }
            }
        };
    };

    let topic_data = match topic_result {
        Ok(topic_data) => topic_data,
        Err(_) => {
            return rsx! {
                section { class: "page",
                    EmptyState {
                        title: "Topic unavailable".to_string(),
                        body: "The discussion for this post could not be loaded.".to_string(),
                    }
                }
            }
        }
    };

    let topic = topic_data.topic.clone();
    let forum = topic_data.forum.clone();

    let can_edit = current_user()
        .as_ref()
        .is_some_and(|u| (u.id == post.author_id && u.edit_posts) || u.is_moderator || u.is_admin);

    if !can_edit {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Access denied".to_string(),
                    body: "You can only edit your own posts.".to_string(),
                }
            }
        };
    }

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                if let Some(forum) = forum.clone() {
                    span { "/" }
                    Link { to: Route::Forum { id: forum.id }, "{forum.name}" }
                }
                span { "/" }
                Link { to: Route::Topic { id: topic.id }, "{topic.subject}" }
                span { "/" }
                span { "Edit post" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Edit".to_string(),
                    title: "Edit post".to_string(),
                    subtitle: "Update your contribution to the discussion.".to_string(),
                }
            }

            article { class: "form-card",
                StatusMessage {
                    message: status(),
                    is_error: is_error(),
                }

                label {
                    "Message"
                    textarea {
                        class: "text-area",
                        rows: "10",
                        value: "{message}",
                        oninput: move |e| message.set(e.value()),
                        placeholder: "Edit your message…",
                    }
                }
                button {
                    class: "primary-button",
                    disabled: submitting(),
                    onclick: move |_| {
                        let form = EditPostForm {
                            post_id: id,
                            message: message(),
                        };
                        spawn(async move {
                            submitting.set(true);
                            match edit_post(form).await {
                                Ok(_) => {
                                    is_error.set(false);
                                    status.set("Post updated.".to_string());
                                    refresh.set(());
                                    navigator
                                        .push(Route::TopicPage {
                                            id: topic.id,
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
                        "Saving…"
                    } else {
                        "Save changes"
                    }
                }
            }
        }
    }
}
