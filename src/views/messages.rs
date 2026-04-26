use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader, StatusMessage},
    data::{
        clean_error, delete_conversation, load_conversation, load_inbox, reply_message,
        send_message, ComposeMessageForm, ReplyMessageForm, SessionUser,
    },
    Route,
};

// ------------------------------------------------------------------
// Inbox
// ------------------------------------------------------------------

#[component]
pub fn Inbox() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let resource = use_resource(move || async move {
        refresh();
        load_inbox().await
    });

    let Some(user) = current_user() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Not signed in".to_string(),
                    body: "Please log in to view your messages.".to_string(),
                }
            }
        };
    };

    let Some(result) = resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading…".to_string(),
                    body: "Fetching your inbox.".to_string(),
                }
            }
        };
    };

    let Ok(data) = result else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Error".to_string(),
                    body: "Could not load inbox.".to_string(),
                }
            }
        };
    };

    struct ConvItem {
        id: i32,
        unread: bool,
        subject: String,
        participants: String,
        last_sender: Option<String>,
        last_body: Option<String>,
    }
    let conv_items: Vec<ConvItem> = data
        .conversations
        .iter()
        .map(|conv| ConvItem {
            id: conv.id,
            unread: conv.unread_count > 0,
            subject: conv.subject.clone(),
            participants: conv
                .participants
                .iter()
                .map(|p| p.username.clone())
                .collect::<Vec<_>>()
                .join(", "),
            last_sender: conv.last_message.as_ref().map(|m| m.sender_name.clone()),
            last_body: conv.last_message.as_ref().map(|m| m.body.clone()),
        })
        .collect();

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                span { "Messages" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Inbox".to_string(),
                    title: "Private Messages".to_string(),
                    subtitle: if data.unread_count > 0 {
                        format!("{} unread messages", data.unread_count)
                    } else {
                        "No new messages".to_string()
                    },
                }
            }

            div { class: "action-bar",
                Link { class: "primary-button", to: Route::ComposeMessage {}, "New Message" }
            }

            if data.conversations.is_empty() {
                EmptyState {
                    title: "No messages".to_string(),
                    body: "Your inbox is empty.".to_string(),
                }
            } else {
                for item in conv_items {
                    article {
                        class: if item.unread { "card card-unread" } else { "card" },
                        onclick: move |_| {
                            navigator.push(Route::Conversation { id: item.id });
                        },
                        h4 { class: "card-title",
                            if item.unread { span { class: "badge badge-new", "New" } }
                            " {item.subject}"
                        }
                        p { class: "card-meta", "With: {item.participants}" }
                        if let Some(ref sender) = item.last_sender {
                            if let Some(ref body) = item.last_body {
                                p { class: "card-snippet", "{sender}: {body}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------
// Conversation thread
// ------------------------------------------------------------------

#[component]
pub fn Conversation(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let resource = use_resource(move || async move {
        refresh();
        load_conversation(id).await
    });

    let mut reply_text = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut sending = use_signal(|| false);

    let Some(user) = current_user() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Not signed in".to_string(),
                    body: "Please log in to view this conversation.".to_string(),
                }
            }
        };
    };

    let Some(result) = resource() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Loading…".to_string(),
                    body: "Fetching conversation.".to_string(),
                }
            }
        };
    };

    let Ok(data) = result else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Error".to_string(),
                    body: "Could not load conversation.".to_string(),
                }
            }
        };
    };

    let participants: String = data
        .participants
        .iter()
        .map(|p| p.username.clone())
        .collect::<Vec<_>>()
        .join(", ");

    struct MsgItem {
        is_me: bool,
        author: String,
        role: String,
        body: String,
    }
    let message_list: Vec<MsgItem> = data
        .messages
        .iter()
        .map(|msg| MsgItem {
            is_me: msg.sender_id == user.id,
            author: msg.sender_name.clone(),
            role: msg.sender_title.clone(),
            body: msg.body.clone(),
        })
        .collect();

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                Link { to: Route::Inbox {}, "Messages" }
                span { "/" }
                span { "{data.conversation.subject}" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Conversation".to_string(),
                    title: data.conversation.subject.clone(),
                    subtitle: format!("With: {participants}"),
                }
            }

            div { class: "action-bar",
                Link { class: "small-button", to: Route::Inbox {}, "Back to inbox" }
                button {
                    class: "danger-button small-button",
                    onclick: move |_| {
                        let cid = id;
                        spawn(async move {
                            let _ = delete_conversation(cid).await;
                            navigator.push(Route::Inbox {});
                        });
                    },
                    "Delete"
                }
            }

            div { class: "message-list",
                for item in message_list {
                    article { class: if item.is_me { "message message-own" } else { "message" },
                        header { class: "message-header",
                            span { class: "message-author", "{item.author}" }
                            span { class: "message-role", "{item.role}" }
                        }
                        p { class: "message-body", "{item.body}" }
                    }
                }
            }

            div { class: "reply-area",
                StatusMessage { message: status(), is_error: is_error() }

                textarea {
                    class: "text-input textarea",
                    value: "{reply_text}",
                    oninput: move |e| reply_text.set(e.value()),
                    placeholder: "Write a reply…",
                }
                button {
                    class: "primary-button",
                    disabled: sending(),
                    onclick: move |_| {
                        let body = reply_text();
                        if body.trim().is_empty() {
                            is_error.set(true);
                            status.set("Message cannot be empty.".to_string());
                            return;
                        }
                        let form = ReplyMessageForm { conversation_id: id, body };
                        spawn(async move {
                            sending.set(true);
                            match reply_message(form).await {
                                Ok(_) => {
                                    reply_text.set(String::new());
                                    is_error.set(false);
                                    status.set("Sent.".to_string());
                                    refresh.set(());
                                }
                                Err(e) => {
                                    is_error.set(true);
                                    status.set(clean_error(e));
                                }
                            }
                            sending.set(false);
                        });
                    },
                    if sending() { "Sending…" } else { "Send Reply" }
                }
            }
        }
    }
}

// ------------------------------------------------------------------
// Compose new message
// ------------------------------------------------------------------

#[component]
pub fn ComposeMessage() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();

    let mut recipient = use_signal(String::new);
    let mut subject = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut sending = use_signal(|| false);

    let Some(_user) = current_user() else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Not signed in".to_string(),
                    body: "Please log in to send messages.".to_string(),
                }
            }
        };
    };

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                Link { to: Route::Inbox {}, "Messages" }
                span { "/" }
                span { "New Message" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Compose".to_string(),
                    title: "New Message".to_string(),
                    subtitle: "Send a private message to another member.".to_string(),
                }
            }

            article { class: "form-card",
                StatusMessage { message: status(), is_error: is_error() }

                label {
                    "Recipient"
                    input {
                        class: "text-input",
                        value: "{recipient}",
                        oninput: move |e| recipient.set(e.value()),
                        placeholder: "Username",
                    }
                }
                label {
                    "Subject"
                    input {
                        class: "text-input",
                        value: "{subject}",
                        oninput: move |e| subject.set(e.value()),
                        placeholder: "Topic",
                    }
                }
                label {
                    "Message"
                    textarea {
                        class: "text-input textarea",
                        value: "{body}",
                        oninput: move |e| body.set(e.value()),
                        placeholder: "Write your message…",
                    }
                }
                div { class: "form-actions",
                    Link { class: "secondary-button", to: Route::Inbox {}, "Cancel" }
                    button {
                        class: "primary-button",
                        disabled: sending(),
                        onclick: move |_| {
                            let r = recipient().trim().to_string();
                            let s = subject().trim().to_string();
                            let b = body().trim().to_string();
                            if r.is_empty() {
                                is_error.set(true);
                                status.set("Recipient is required.".to_string());
                                return;
                            }
                            if s.is_empty() {
                                is_error.set(true);
                                status.set("Subject is required.".to_string());
                                return;
                            }
                            if b.is_empty() {
                                is_error.set(true);
                                status.set("Message body is required.".to_string());
                                return;
                            }
                            let form = ComposeMessageForm {
                                recipient_username: r,
                                subject: s,
                                body: b,
                            };
                            spawn(async move {
                                sending.set(true);
                                match send_message(form).await {
                                    Ok(result) => {
                                        is_error.set(false);
                                        status.set("Message sent.".to_string());
                                        navigator.push(Route::Conversation { id: result.conversation_id });
                                    }
                                    Err(e) => {
                                        is_error.set(true);
                                        status.set(clean_error(e));
                                    }
                                }
                                sending.set(false);
                            });
                        },
                        if sending() { "Sending…" } else { "Send Message" }
                    }
                }
            }
        }
    }
}
