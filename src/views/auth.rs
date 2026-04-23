use dioxus::prelude::*;

use crate::components::SectionHeader;

#[component]
pub fn Login() -> Element {
    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Authentication".to_string(),
                    title: "Login flow staging".to_string(),
                    subtitle: "The UI is in place so the route tree matches FluxBB. Session-backed authentication will connect to this form in the next backend slice.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Sign in" }
                    label { "Username"
                        input { class: "text-input", placeholder: "nora" }
                    }
                    label { "Password"
                        input { class: "text-input", r#type: "password", placeholder: "password" }
                    }
                    label { class: "checkbox-row",
                        input { r#type: "checkbox" }
                        span { "Remember me on this device" }
                    }
                    button { class: "primary-button", disabled: true, "Session wiring next" }
                }

                article { class: "panel side-note",
                    h3 { "Planned integration" }
                    p { "The current Postgres schema is ready for a user table-backed auth layer. The missing pieces are password hashing, session storage, and profile editing routes." }
                }
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Registration".to_string(),
                    title: "Account creation staging".to_string(),
                    subtitle: "This preserves the FluxBB entry point while the real validation and moderation rules are migrated into Rust.".to_string(),
                }
            }

            div { class: "auth-grid",
                article { class: "form-card",
                    h3 { "Create account" }
                    label { "Username"
                        input { class: "text-input", placeholder: "sol" }
                    }
                    label { "Email"
                        input { class: "text-input", placeholder: "sol@example.com" }
                    }
                    label { "Password"
                        input { class: "text-input", r#type: "password", placeholder: "Minimum 12 characters" }
                    }
                    label { "Bio"
                        textarea { class: "text-area", rows: "5", placeholder: "Optional profile summary" }
                    }
                    button { class: "primary-button", disabled: true, "Validation layer next" }
                }

                article { class: "panel side-note",
                    h3 { "Migration note" }
                    p { "FluxBB's original registration rules can be reintroduced as server validation once the user and session tables are extended beyond the current read-only board slice." }
                }
            }
        }
    }
}
