use dioxus::prelude::*;

use crate::components::ConfirmButton;
use crate::data::{delete_post, report_post, Post, ReportPostForm, SessionUser};
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
pub fn PostCard(
    author_name: String,
    author_role: String,
    author_id: i32,
    post: Post,
    current_user: Option<SessionUser>,
    forum_id: i32,
) -> Element {
    let mut refresh = use_context::<Signal<()>>();
    let navigator = use_navigator();

    let can_edit = current_user
        .as_ref()
        .is_some_and(|u| (u.id == author_id && u.edit_posts) || u.is_moderator || u.is_admin);
    let can_delete = current_user
        .as_ref()
        .is_some_and(|u| (u.id == author_id && u.delete_posts) || u.is_moderator || u.is_admin);
    let can_report = current_user
        .as_ref()
        .is_some_and(|u| u.id != author_id && !u.is_admin);

    let post_id = post.id;
    let mut reporting = use_signal(|| false);
    let mut report_reason = use_signal(String::new);
    let mut report_status = use_signal(String::new);

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

                if can_edit || can_delete {
                    div { class: "post-actions",
                        if can_edit {
                            Link {
                                class: "small-button",
                                to: Route::EditPost { id: post_id },
                                "Edit"
                            }
                        }
                        if can_delete {
                            ConfirmButton {
                                label: "Delete",
                                class: "danger-button small-button",
                                on_confirm: move |_| {
                                    spawn(async move {
                                        match delete_post(post_id).await {
                                            Ok(0) => {
                                                refresh.set(());
                                            }
                                            Ok(_) => {
                                                navigator
                                                    .push(Route::ForumPage {
                                                        id: forum_id,
                                                        page: 1,
                                                    });
                                            }
                                            Err(_) => {}
                                        }
                                    });
                                },
                            }
                        }
                    }
                }

                if can_report {
                    if reporting() {
                        div { class: "report-form",
                            input {
                                class: "text-input",
                                value: "{report_reason}",
                                oninput: move |e| report_reason.set(e.value()),
                                placeholder: "Reason for reporting...",
                            }
                            button {
                                class: "small-button",
                                onclick: move |_| {
                                    let reason = report_reason().trim().to_string();
                                    if reason.is_empty() {
                                        report_status.set("Reason is required.".to_string());
                                        return;
                                    }
                                    spawn(async move {
                                        match report_post(ReportPostForm { post_id, reason }).await {
                                            Ok(_) => {
                                                reporting.set(false);
                                                report_reason.set(String::new());
                                                report_status.set(String::new());
                                            }
                                            Err(_) => {
                                                report_status.set("Failed to submit report.".to_string());
                                            }
                                        }
                                    });
                                },
                                "Submit"
                            }
                            button {
                                class: "small-button",
                                onclick: move |_| {
                                    reporting.set(false);
                                    report_reason.set(String::new());
                                    report_status.set(String::new());
                                },
                                "Cancel"
                            }
                        }
                    } else {
                        button {
                            class: "small-button",
                            onclick: move |_| reporting.set(true),
                            "Report"
                        }
                    }
                    if !report_status().is_empty() {
                        p { class: "form-message form-error", "{report_status}" }
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

#[component]
pub fn StatusMessage(message: String, is_error: bool) -> Element {
    if message.is_empty() {
        return rsx! {};
    }

    rsx! {
        p { class: if is_error { "form-message form-error" } else { "form-message form-success" },
            "{message}"
        }
    }
}

#[component]
pub fn Pagination(
    current_page: i32,
    total_pages: i32,
    prev_route: Option<Route>,
    next_route: Option<Route>,
    page_routes: Vec<(i32, Route)>,
) -> Element {
    if total_pages <= 1 {
        return rsx! {};
    }

    rsx! {
        nav { class: "pagination",
            if let Some(route) = prev_route {
                Link { class: "page-button", to: route, "← Prev" }
            }
            for (page_number, route) in page_routes {
                if page_number == current_page {
                    span { class: "page-button active", "{page_number}" }
                } else {
                    Link { class: "page-button", to: route, "{page_number}" }
                }
            }
            if let Some(route) = next_route {
                Link { class: "page-button", to: route, "Next →" }
            }
        }
    }
}
