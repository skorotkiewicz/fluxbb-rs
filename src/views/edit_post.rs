use dioxus::prelude::*;

use crate::{
    components::{AttachmentItem, EmptyState, SectionHeader, StatusMessage},
    data::{
        clean_error, edit_post, load_attachments, load_post, load_topic_data, upload_attachment,
        EditPostForm, SessionUser,
    },
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
    let mut selected_files = use_signal(Vec::<(String, Vec<u8>)>::new);
    let mut is_uploading = use_signal(|| false);

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

    // Load attachments for this post
    let attachments_resource = use_resource(move || async move { load_attachments(id).await });

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
                StatusMessage { message: status(), is_error: is_error() }

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

                // Existing attachments
                if let Some(Ok(attachments)) = attachments_resource() {
                    if !attachments.is_empty() {
                        div { class: "post-attachments",
                            h4 { class: "attachments-title", "Current Attachments" }
                            div { class: "attachments-list",
                                for attachment in attachments {
                                    AttachmentItem {
                                        attachment: attachment.clone(),
                                        current_user: current_user().clone(),
                                        author_id: post.author_id,
                                    }
                                }
                            }
                        }
                    }
                }

                // Attachment management
                div { class: "file-upload",
                    h4 { class: "attachments-title", "Add Attachments" }

                    label {
                        class: "file-upload-label",
                        r#for: "file-input-edit",
                        "Select files to upload"
                        span { class: "file-upload-hint", "Max 10MB • jpg, png, gif, pdf, txt, zip, mp4" }
                        input {
                            id: "file-input-edit",
                            class: "file-upload-input",
                            r#type: "file",
                            multiple: true,
                            accept: ".jpg,.jpeg,.png,.gif,.pdf,.txt,.zip,.mp4",
                            onchange: move |e| {
                                let files: Vec<dioxus::html::FileData> = e.files();
                                if !files.is_empty() {
                                    spawn(async move {
                                        let mut files_vec = selected_files();
                                        for file_data in files {
                                            let file_name = file_data.name();
                                            match file_data.read_bytes().await {
                                                Ok(bytes) => {
                                                    files_vec.push((file_name, bytes.to_vec()));
                                                }
                                                Err(err) => {
                                                    files_vec.push((format!("{} (read error: {})", file_name, err), Vec::new()));
                                                }
                                            }
                                        }
                                        selected_files.set(files_vec);
                                    });
                                }
                            },
                        }
                    }

                    if !selected_files().is_empty() {
                        div { class: "file-upload-list",
                            for (i, (file_name, _content)) in selected_files().iter().enumerate() {
                                div { class: "file-upload-item",
                                    span { class: "file-upload-name", "{file_name}" }
                                    button {
                                        class: "file-upload-remove",
                                        onclick: move |_| {
                                            let mut files = selected_files();
                                            if i < files.len() {
                                                files.remove(i);
                                                selected_files.set(files);
                                            }
                                        },
                                        "Remove"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: "primary-button",
                        disabled: submitting() || is_uploading(),
                        onclick: move |_| {
                            let m = message().trim().to_string();
                            if m.is_empty() {
                                is_error.set(true);
                                status.set("Message required.".to_string());
                                return;
                            }

                            let pid = id;
                            let tid = topic.id;
                            spawn(async move {
                                is_uploading.set(true);
                                submitting.set(true);
                                is_error.set(false);
                                status.set(String::new());

                                // First save the post
                                let form = EditPostForm {
                                    post_id: pid,
                                    message: m,
                                };

                                if let Err(e) = edit_post(form).await {
                                    is_error.set(true);
                                    status.set(clean_error(e));
                                    is_uploading.set(false);
                                    submitting.set(false);
                                    return;
                                }

                                // Upload attachments
                                let files_to_upload = selected_files();
                                let mut upload_errors = Vec::new();
                                for (file_name, content) in &files_to_upload {
                                    if content.is_empty() {
                                        continue;
                                    }
                                    if let Err(e) = upload_attachment(pid, file_name.clone(), content.clone()).await {
                                        upload_errors.push(format!("{}: {}", file_name, clean_error(e)));
                                    }
                                }
                                selected_files.set(Vec::new());

                                if upload_errors.is_empty() {
                                    status.set("Post updated.".to_string());
                                } else {
                                    is_error.set(true);
                                    status.set(format!("Post saved, but some uploads failed: {}", upload_errors.join(", ")));
                                }

                                refresh.set(());
                                navigator.push(Route::TopicPage {
                                    id: tid,
                                    page: 1,
                                });

                                is_uploading.set(false);
                                submitting.set(false);
                            });
                        },
                        if submitting() || is_uploading() {
                            if is_uploading() { "Saving…" } else { "Saving…" }
                        } else {
                            "Save changes"
                        }
                    }
                }
            }
        }
    }
}
