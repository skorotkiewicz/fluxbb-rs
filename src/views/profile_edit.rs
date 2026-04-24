use dioxus::prelude::*;

use crate::{
    components::{EmptyState, SectionHeader},
    data::{
        change_password, clean_error, load_profile_data, update_profile, ChangePasswordForm,
        SessionUser, UpdateProfileForm,
    },
    Route,
};

#[component]
pub fn ProfileEdit(id: i32) -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let mut refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_profile_data(id).await
    });

    let mut email = use_signal(String::new);
    let mut location = use_signal(String::new);
    let mut about = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut password_confirm = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);

    // Pre-fill form with existing profile data
    use_effect(move || {
        if let Some(Ok(data)) = data_resource() {
            let u = &data.user;
            if email().is_empty() && !u.email.is_empty() {
                email.set(u.email.clone());
            }
            if title().is_empty() && !u.title.is_empty() {
                title.set(u.title.clone());
            }
            if location().is_empty() && !u.location.is_empty() {
                location.set(u.location.clone());
            }
            if about().is_empty() && !u.about.is_empty() {
                about.set(u.about.clone());
            }
        }
    });

    let data = if let Some(Ok(data)) = data_resource() {
        data
    } else {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Loading profile…" }
                }
            }
        };
    };

    let user = data.user.clone();
    let can_edit = current_user()
        .as_ref()
        .is_some_and(|u| u.id == id || u.group_id == 1);

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

            if !can_edit {
                EmptyState {
                    title: "Access denied".to_string(),
                    body: "You can only edit your own profile.".to_string(),
                }
            } else {
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
                            let e = email().trim().to_string();
                            let validation = if !e.is_empty() && (!e.contains('@') || !e.contains('.')) {
                                "Please enter a valid email address."
                            } else {
                                ""
                            };
                            if !validation.is_empty() {
                                is_error.set(true);
                                status.set(validation.to_string());
                                return;
                            }
                            let form = UpdateProfileForm {
                                user_id: id,
                                email: e,
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
                                        status.set(clean_error(e));
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

                    hr {}

                    h3 { "Change password" }
                    label {
                        "New password"
                        input {
                            class: "text-input",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            placeholder: "Minimum 9 characters",
                        }
                    }
                    label {
                        "Confirm password"
                        input {
                            class: "text-input",
                            r#type: "password",
                            value: "{password_confirm}",
                            oninput: move |e| password_confirm.set(e.value()),
                            placeholder: "Repeat password",
                        }
                    }
                    button {
                        class: "primary-button",
                        disabled: submitting(),
                        onclick: move |_| {
                            let p = password();
                            let pc = password_confirm();
                            let validation = if p.len() < 9 {
                                "Password must be at least 9 characters."
                            } else if p != pc {
                                "Passwords do not match."
                            } else {
                                ""
                            };
                            if !validation.is_empty() {
                                is_error.set(true);
                                status.set(validation.to_string());
                                return;
                            }
                            let form = ChangePasswordForm {
                                user_id: id,
                                password: p,
                            };
                            spawn(async move {
                                submitting.set(true);
                                match change_password(form).await {
                                    Ok(_) => {
                                        is_error.set(false);
                                        status.set("Password changed.".to_string());
                                        password.set(String::new());
                                        password_confirm.set(String::new());
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
                            "Changing…"
                        } else {
                            "Change password"
                        }
                    }
                }
            }
        }
    }
}
