use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::{update_profile, AppData, SessionUser, UpdateProfileForm},
    Route,
};

#[component]
pub fn ProfileEdit(id: i32) -> Element {
    let board = use_context::<AppData>();
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let Some(user) = board.user(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "User not found".to_string(),
                    body: "This user does not exist.".to_string(),
                }
            }
        };
    };

    let can_edit = current_user()
        .as_ref()
        .is_some_and(|u| u.id == id || u.group_id == 1);

    if !can_edit {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Access denied".to_string(),
                    body: "You can only edit your own profile.".to_string(),
                }
            }
        };
    }

    let mut email = use_signal(|| user.email.clone());
    let mut location = use_signal(|| user.location.clone());
    let mut about = use_signal(|| user.about.clone());
    let mut title = use_signal(|| user.title.clone());
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                span { "/" }
                Link { to: Route::Users {}, "Members" }
                span { "/" }
                Link { to: Route::Profile { id }, "{user.username}" }
                span { "/" }
                span { "Edit" }
            }

            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Account".to_string(),
                    title: "Edit profile".to_string(),
                    subtitle: "Update your public information.".to_string(),
                }
            }

            article { class: "form-card",
                if !status().is_empty() {
                    p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                        "{status}"
                    }
                }

                label {
                    "Email"
                    input {
                        class: "text-input",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                        placeholder: "you@example.com",
                    }
                }
                label {
                    "Title"
                    input {
                        class: "text-input",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                        placeholder: "Member, Moderator, etc.",
                    }
                }
                label {
                    "Location"
                    input {
                        class: "text-input",
                        value: "{location}",
                        oninput: move |e| location.set(e.value()),
                        placeholder: "Optional",
                    }
                }
                label {
                    "About"
                    textarea {
                        class: "text-area",
                        rows: "5",
                        value: "{about}",
                        oninput: move |e| about.set(e.value()),
                        placeholder: "Short profile summary",
                    }
                }
                button {
                    class: "primary-button",
                    disabled: submitting(),
                    onclick: move |_| {
                        let form = UpdateProfileForm {
                            user_id: id,
                            email: email(),
                            location: location(),
                            about: about(),
                            title: title(),
                        };
                        spawn(async move {
                            submitting.set(true);
                            match update_profile(form).await {
                                Ok(_) => {
                                    is_error.set(false);
                                    status.set("Profile saved.".to_string());
                                    refresh.set(());
                                    navigator.push(Route::Profile { id });
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
                        "Saving…"
                    } else {
                        "Save profile"
                    }
                }
            }
        }
    }
}
