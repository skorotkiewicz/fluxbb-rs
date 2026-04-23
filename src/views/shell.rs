use dioxus::prelude::*;

use crate::{
    data::{load_board, AppData},
    Route,
};

const HEADER_ART: Asset = asset!("/assets/header.svg");

#[component]
pub fn AppShell() -> Element {
    let board_resource = use_server_future(load_board)?;
    let board = board_resource()
        .and_then(Result::ok)
        .unwrap_or_else(AppData::fallback);
    let stats = board.board_stats();

    use_context_provider(|| board.clone());

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
                Link { class: "nav-link nav-link-muted", to: Route::Login {}, "Login" }
                Link { class: "nav-link nav-link-strong", to: Route::Register {}, "Register" }
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
                p { "FluxBB RS uses Dioxus 0.7 for the web shell and a small Postgres schema for the first migration slice." }
            }
        }
    }
}
