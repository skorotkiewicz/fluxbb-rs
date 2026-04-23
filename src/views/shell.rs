use dioxus::{document, prelude::*};

use crate::{
    data::{
        cookie_max_age, cookie_name, current_session_user, load_board, logout_account, SessionUser,
    },
    Route,
};

const HEADER_ART: Asset = asset!("/assets/header.svg");

#[component]
pub fn AppShell() -> Element {
    let board_resource = use_server_future(load_board)?;
    let session_resource =
        use_resource(|| async move { current_session_user().await.unwrap_or(None) });
    let mut current_user = use_signal(|| None::<SessionUser>);

    use_effect(move || {
        if let Some(user) = session_resource() {
            current_user.set(user);
        }
    });

    let board = match board_resource() {
        Some(Ok(board)) => board,
        Some(Err(error)) => {
            return rsx! {
                section { class: "page",
                    article { class: "empty-state",
                        h3 { "Board unavailable" }
                        p { "{error}" }
                    }
                }
            };
        }
        None => {
            return rsx! {
                section { class: "page",
                    article { class: "empty-state",
                        h3 { "Loading board" }
                        p { "Waiting for Postgres-backed board data." }
                    }
                }
            };
        }
    };

    let stats = board.board_stats();

    use_context_provider(|| board.clone());
    use_context_provider(|| current_user);

    rsx! {
        div { class: "site-shell",
            header { class: "masthead",
                div { class: "masthead-copy",
                    p { class: "eyebrow", "FluxBB -> Rust migration" }
                    h1 { "{board.meta.title}" }
                    p { class: "masthead-tagline", "{board.meta.tagline}" }
                }

                img {
                    class: "masthead-art",
                    src: HEADER_ART,
                    alt: "FluxBB RS banner",
                }
            }

            nav { class: "top-nav",
                Link { class: "nav-link", to: Route::Index {}, "Forums" }
                Link { class: "nav-link", to: Route::Search {}, "Search" }
                Link { class: "nav-link", to: Route::Users {}, "Users" }
                Link { class: "nav-link", to: Route::Admin {}, "Admin" }

                if let Some(user) = current_user() {
                    span { class: "auth-chip", "Signed in as {user.username} ({user.title})" }
                    button {
                        class: "nav-link nav-button",
                        onclick: move |_| {
                            spawn(async move {
                                let _ = logout_account().await;
                                let _ = document::eval(&format!(
                                    "document.cookie = '{}=; path=/; max-age=0; samesite=lax';",
                                    cookie_name()
                                ));
                                current_user.set(None);
                            });
                        },
                        "Logout"
                    }
                } else {
                    Link { class: "nav-link nav-link-muted", to: Route::Login {}, "Login" }
                    Link { class: "nav-link nav-link-strong", to: Route::Register {}, "Register" }
                }
            }

            div { class: "site-meta",
                p { "Members: {stats.members}" }
                p { "Topics: {stats.topics}" }
                p { "Posts: {stats.posts}" }
                p { "Newest: {stats.newest_member}" }
                p { "Session lifetime: {cookie_max_age() / 86_400} days" }
            }

            main { class: "page-wrap",
                Outlet::<Route> {}
            }

            footer { class: "site-footer",
                p { "The board data now comes from Postgres. Login and registration use server actions plus a browser cookie-backed session token." }
            }
        }
    }
}
