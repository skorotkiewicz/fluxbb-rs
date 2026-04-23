use dioxus::{document, prelude::*};

use crate::{
    data::{check_installed, cookie_max_age, cookie_name, install_board, InstallForm},
    Route, MAIN_CSS,
};

#[component]
pub fn Install() -> Element {
    let navigator = use_navigator();
    let mut board_title = use_signal(|| "My Forum".to_string());
    let mut board_tagline = use_signal(|| "A modern forum built with Rust".to_string());
    let mut admin_username = use_signal(String::new);
    let mut admin_email = use_signal(String::new);
    let mut admin_password = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_error = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut already_installed = use_signal(|| false);

    let installed_check = use_resource(|| async move { check_installed().await.unwrap_or(false) });

    use_effect(move || {
        if let Some(true) = installed_check() {
            already_installed.set(true);
        }
    });

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div { class: "site-shell",
            header { class: "masthead",
                div { class: "masthead-copy",
                    p { class: "eyebrow", "Installation" }
                    h1 { "FluxBB RS Setup" }
                    p { class: "masthead-tagline",
                        "Configure your forum and create the administrator account."
                    }
                }
            }

            main { class: "page-wrap",
                section { class: "page",
                    if already_installed() {
                        article { class: "panel",
                            div { class: "panel-heading",
                                h3 { "Already installed" }
                                p { "This forum has already been set up." }
                            }
                            Link { to: Route::Index {}, class: "primary-button", "Go to forum" }
                        }
                    } else {
                        div { class: "auth-grid",
                            article { class: "form-card",
                                h3 { "Board setup" }

                                if !status().is_empty() {
                                    p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                                        "{status}"
                                    }
                                }

                                label {
                                    "Board title"
                                    input {
                                        class: "text-input",
                                        value: "{board_title}",
                                        oninput: move |e| board_title.set(e.value()),
                                    }
                                }
                                label {
                                    "Board tagline"
                                    input {
                                        class: "text-input",
                                        value: "{board_tagline}",
                                        oninput: move |e| board_tagline.set(e.value()),
                                    }
                                }
                                label {
                                    "Admin username"
                                    input {
                                        class: "text-input",
                                        value: "{admin_username}",
                                        oninput: move |e| admin_username.set(e.value()),
                                        placeholder: "admin",
                                    }
                                }
                                label {
                                    "Admin email"
                                    input {
                                        class: "text-input",
                                        value: "{admin_email}",
                                        oninput: move |e| admin_email.set(e.value()),
                                        placeholder: "admin@example.com",
                                    }
                                }
                                label {
                                    "Admin password"
                                    input {
                                        class: "text-input",
                                        r#type: "password",
                                        value: "{admin_password}",
                                        oninput: move |e| admin_password.set(e.value()),
                                        placeholder: "Minimum 9 characters",
                                    }
                                }
                                button {
                                    class: "primary-button",
                                    disabled: submitting(),
                                    onclick: move |_| {
                                        let form = InstallForm {
                                            board_title: board_title(),
                                            board_tagline: board_tagline(),
                                            admin_username: admin_username(),
                                            admin_email: admin_email(),
                                            admin_password: admin_password(),
                                        };
                                        spawn(async move {
                                            submitting.set(true);
                                            match install_board(form).await {
                                                Ok(resp) => {
                                                    let script = format!(
                                                        "document.cookie = '{}={}; path=/; max-age={}; samesite=lax';",
                                                        cookie_name(),
                                                        resp.session_token,
                                                        cookie_max_age(),
                                                    );
                                                    let _ = document::eval(&script);
                                                    is_error.set(false);
                                                    status.set(resp.message);
                                                    navigator.push(Route::Index {});
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
                                        "Installing…"
                                    } else {
                                        "Install forum"
                                    }
                                }
                            }

                            article { class: "panel side-note",
                                h3 { "What happens" }
                                p {
                                    "This will create the database tables, an administrator account, and a default forum category."
                                }
                                p {
                                    "You can add more categories and forums from the admin panel after installation."
                                }
                            }
                        }
                    }
                }
            }

            footer { class: "site-footer",
                p { "Powered by FluxBB RS" }
            }
        }
    }
}
