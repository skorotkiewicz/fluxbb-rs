use dioxus::{document, prelude::*};

use crate::{
    components::SectionHeader,
    data::{
        clean_error, cookie_max_age, cookie_name, login_account, register_account, LoginForm,
        RegisterForm, SessionUser,
    },
    Route,
};

#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let mut auth_user = use_context::<Signal<Option<SessionUser>>>();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut remember = use_signal(|| true);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Account".to_string(),
                    title: "Login".to_string(),
                    subtitle: "Sign in to access your account and participate in discussions.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Sign in" }

                    if !status().is_empty() {
                        p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                            "{status}"
                        }
                    }

                    label {
                        "Username"
                        input {
                            class: "text-input",
                            value: "{username}",
                            oninput: move |event| username.set(event.value()),
                            placeholder: "Your username",
                        }
                    }
                    label {
                        "Password"
                        input {
                            class: "text-input",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |event| password.set(event.value()),
                            placeholder: "Password",
                        }
                    }
                    label { class: "checkbox-row",
                        input {
                            r#type: "checkbox",
                            checked: remember(),
                            onchange: move |_| remember.toggle(),
                        }
                        span { "Remember me on this browser for 14 days" }
                    }
                    button {
                        class: "primary-button",
                        disabled: submitting(),
                        onclick: move |_| {
                            let login = LoginForm {
                                username: username(),
                                password: password(),
                                remember: remember(),
                            };

                            spawn(async move {
                                submitting.set(true);
                                match login_account(login).await {
                                    Ok(response) => {
                                        let max_age_clause = if remember() {
                                            format!("; max-age={}", cookie_max_age())
                                        } else {
                                            String::new()
                                        };
                                        let script = format!(
                                            "document.cookie = '{}={}; path=/; samesite=strict{max_age_clause}'; document.cookie = '{}={}; path=/; samesite=strict{max_age_clause}';",
                                            cookie_name(), response.session_token,
                                            crate::data::csrf_cookie_name(), response.user.csrf_token,
                                        );
                                        let _ = document::eval(&script);
                                        auth_user.set(Some(response.user));
                                        is_error.set(false);
                                        status.set(response.message);
                                        navigator.push(Route::Index {});
                                    }
                                    Err(error) => {
                                        is_error.set(true);
                                        status.set(clean_error(error));
                                    }
                                }
                                submitting.set(false);
                            });
                        },
                        if submitting() {
                            "Signing in…"
                        } else {
                            "Sign in"
                        }
                    }
                }

            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let mut auth_user = use_context::<Signal<Option<SessionUser>>>();
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut location = use_signal(String::new);
    let mut about = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Account".to_string(),
                    title: "Create account".to_string(),
                    subtitle: "Join the community to start posting.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Create account" }

                    if !status().is_empty() {
                        p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                            "{status}"
                        }
                    }

                    label {
                        "Username"
                        input {
                            class: "text-input",
                            value: "{username}",
                            oninput: move |event| username.set(event.value()),
                            placeholder: "Pick a username",
                        }
                    }
                    label {
                        "Email"
                        input {
                            class: "text-input",
                            value: "{email}",
                            oninput: move |event| email.set(event.value()),
                            placeholder: "you@example.com",
                        }
                    }
                    label {
                        "Password"
                        input {
                            class: "text-input",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |event| password.set(event.value()),
                            placeholder: "Minimum 9 characters",
                        }
                    }
                    label {
                        "Location"
                        input {
                            class: "text-input",
                            value: "{location}",
                            oninput: move |event| location.set(event.value()),
                            placeholder: "Optional",
                        }
                    }
                    label {
                        "About"
                        textarea {
                            class: "text-area",
                            rows: "5",
                            value: "{about}",
                            oninput: move |event| about.set(event.value()),
                            placeholder: "Short profile summary",
                        }
                    }
                    button {
                        class: "primary-button",
                        disabled: submitting(),
                        onclick: move |_| {
                            let u = username();
                            let e = email();
                            let p = password();

                            let validation = if u.trim().len() < 2 {
                                "Username must be at least 2 characters."
                            } else if u.trim().len() > 25 {
                                "Username must be at most 25 characters."
                            } else if !e.contains('@') || !e.contains('.') {
                                "Please enter a valid email address."
                            } else if p.len() < 9 {
                                "Password must be at least 9 characters."
                            } else {
                                ""
                            };

                            if !validation.is_empty() {
                                is_error.set(true);
                                status.set(validation.to_string());
                                return;
                            }

                            let form = RegisterForm {
                                username: u,
                                email: e,
                                password: p,
                                location: location(),
                                about: about(),
                            };

                            spawn(async move {
                                submitting.set(true);
                                match register_account(form).await {
                                    Ok(response) => {
                                        let script = format!(
                                            "document.cookie = '{}={}; path=/; max-age={}; samesite=strict'; document.cookie = '{}={}; path=/; max-age={}; samesite=strict';",
                                            cookie_name(), response.session_token, cookie_max_age(),
                                            crate::data::csrf_cookie_name(), response.user.csrf_token, cookie_max_age(),
                                        );
                                        let _ = document::eval(&script);
                                        auth_user.set(Some(response.user));
                                        is_error.set(false);
                                        status.set(response.message);
                                        navigator.push(Route::Index {});
                                    }
                                    Err(error) => {
                                        is_error.set(true);
                                        status.set(clean_error(error));
                                    }
                                }
                                submitting.set(false);
                            });
                        },
                        if submitting() {
                            "Creating account…"
                        } else {
                            "Create account"
                        }
                    }
                }

                article { class: "panel side-note",
                    h3 { "Registration rules" }
                    p { "Username must be 2–25 characters. Password must be at least 9 characters." }
                    p { "Duplicate usernames and email addresses are not allowed." }
                }
            }
        }
    }
}
