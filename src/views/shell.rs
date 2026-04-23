use dioxus::{document, prelude::*};

use crate::{
    data::{
        cookie_name, current_session_user, load_board, logout_account, AppData, SessionUser,
    },
    Route,
};

const HEADER_ART: Asset = asset!("/assets/header.svg");

#[component]
pub fn AppShell() -> Element {
    let mut board_resource = use_resource(|| async move { load_board().await });
    let session_resource =
        use_resource(|| async move { current_session_user().await.unwrap_or(None) });
    let mut current_user = use_signal(|| None::<SessionUser>);

    use_effect(move || {
        if let Some(user) = session_resource() {
            current_user.set(user);
        }
    });

    // Provide a refresh trigger that child views can call after mutations
    let refresh = use_signal(|| ());
    use_context_provider(|| refresh);

    let board = match board_resource() {
        Some(Ok(board)) => board,
        Some(Err(_error)) => {
            return rsx! {
                section { class: "page",
                    article { class: "empty-state",
                        h3 { "Board not installed" }
                        p { "This forum has not been set up yet." }
                        Link { class: "primary-button", to: Route::Install {}, "Run installer" }
                    }
                }
            };
        }
        None => {
            return rsx! {
                section { class: "page",
                    article { class: "empty-state",
                        h3 { "Loading…" }
                        p { "Connecting to the forum." }
                    }
                }
            };
        }
    };

    let stats = board.board_stats();
    let is_admin = current_user()
        .as_ref()
        .is_some_and(|u| u.group_id == 1);

    use_context_provider(|| board.clone());
    use_context_provider(|| current_user);

    // Watch for refresh trigger
    use_effect(move || {
        refresh();
        board_resource.restart();
    });

    rsx! {
        div { class: "site-shell",
            header { class: "masthead",
                div { class: "masthead-copy",
                    p { class: "eyebrow", "Community Forum" }
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

                if is_admin {
                    Link { class: "nav-link", to: Route::Admin {}, "Admin" }
                }

                if let Some(ref user) = current_user() {
                    Link {
                        class: "auth-chip",
                        to: Route::Profile { id: user.id },
                        "Signed in as {user.username}"
                    }
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
            }

            main { class: "page-wrap",
                Outlet::<Route> {}
            }

            footer { class: "site-footer",
                p { "Powered by FluxBB RS" }
            }
        }
    }
}
