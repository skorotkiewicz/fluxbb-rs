use dioxus::{document, prelude::*};

use crate::{
    components::SectionHeader,
    data::{
        cookie_max_age, cookie_name, login_account, register_account, LoginForm, RegisterForm,
        SessionUser,
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
                    kicker: "Authentication".to_string(),
                    title: "Login".to_string(),
                    subtitle: "This now follows the core FluxBB flow: verify credentials, create a session, and move back into the forum.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Sign in" }

                    if !status().is_empty() {
                        p {
                            class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                            "{status}"
                        }
                    }

                    label { "Username"
                        input {
                            class: "text-input",
                            value: "{username}",
                            oninput: move |event| username.set(event.value()),
                            placeholder: "nora",
                        }
                    }
                    label { "Password"
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
                        span { "Remember me on this browser" }
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
                                        let script = format!(
                                            "document.cookie = '{}={}; path=/; max-age={}; samesite=lax';",
                                            cookie_name(),
                                            response.session_token,
                                            cookie_max_age(),
                                        );
                                        let _ = document::eval(&script);
                                        auth_user.set(Some(response.user));
                                        is_error.set(false);
                                        status.set(response.message);
                                        navigator.push(Route::Index {});
                                    }
                                    Err(error) => {
                                        is_error.set(true);
                                        status.set(error.to_string());
                                    }
                                }
                                submitting.set(false);
                            });
                        },
                        if submitting() { "Signing in..." } else { "Sign in" }
                    }
                }

                article { class: "panel side-note",
                    h3 { "What matches FluxBB" }
                    p { "Usernames are validated case-insensitively and the password check now happens on the server against the stored user hash." }
                    p { "Seeded local accounts use the demo password `fluxbb-demo`." }
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
                    kicker: "Registration".to_string(),
                    title: "Create account".to_string(),
                    subtitle: "The form now writes a real user row into Postgres, then signs the new user in immediately like a non-verified FluxBB registration flow.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Create account" }

                    if !status().is_empty() {
                        p {
                            class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                            "{status}"
                        }
                    }

                    label { "Username"
                        input {
                            class: "text-input",
                            value: "{username}",
                            oninput: move |event| username.set(event.value()),
                            placeholder: "sol",
                        }
                    }
                    label { "Email"
                        input {
                            class: "text-input",
                            value: "{email}",
                            oninput: move |event| email.set(event.value()),
                            placeholder: "sol@example.com",
                        }
                    }
                    label { "Password"
                        input {
                            class: "text-input",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |event| password.set(event.value()),
                            placeholder: "Minimum 9 characters",
                        }
                    }
                    label { "Location"
                        input {
                            class: "text-input",
                            value: "{location}",
                            oninput: move |event| location.set(event.value()),
                            placeholder: "Optional",
                        }
                    }
                    label { "About"
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
                            let form = RegisterForm {
                                username: username(),
                                email: email(),
                                password: password(),
                                location: location(),
                                about: about(),
                            };

                            spawn(async move {
                                submitting.set(true);
                                match register_account(form).await {
                                    Ok(response) => {
                                        let script = format!(
                                            "document.cookie = '{}={}; path=/; max-age={}; samesite=lax';",
                                            cookie_name(),
                                            response.session_token,
                                            cookie_max_age(),
                                        );
                                        let _ = document::eval(&script);
                                        auth_user.set(Some(response.user));
                                        is_error.set(false);
                                        status.set(response.message);
                                        navigator.push(Route::Index {});
                                    }
                                    Err(error) => {
                                        is_error.set(true);
                                        status.set(error.to_string());
                                    }
                                }
                                submitting.set(false);
                            });
                        },
                        if submitting() { "Creating account..." } else { "Create account" }
                    }
                }

                article { class: "panel side-note",
                    h3 { "What matches FluxBB" }
                    p { "Username length, reserved-name checks, duplicate account checks, and minimum password length are enforced on the server before the row is inserted." }
                }
            }
        }
    }
}
