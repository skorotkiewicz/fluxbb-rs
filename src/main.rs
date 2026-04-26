use dioxus::prelude::*;

use views::{
    Admin, AppShell, EditPost, ForgotPassword, Forum, ForumPage, Help, Index, Install, Login,
    NewTopic, Profile, ProfileEdit, Register, ResetPassword, Rules, Search, Topic, TopicPage,
    Users,
};

mod components;
mod data;
mod views;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/Air.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/install")]
    Install {},
    #[layout(AppShell)]
        #[route("/")]
        Index {},
        #[route("/forum/:id")]
        Forum { id: i32 },
        #[route("/forum/:id/page/:page")]
        ForumPage { id: i32, page: i32 },
        #[route("/forum/:id/new")]
        NewTopic { id: i32 },
        #[route("/topic/:id")]
        Topic { id: i32 },
        #[route("/topic/:id/page/:page")]
        TopicPage { id: i32, page: i32 },
        #[route("/post/:id/edit")]
        EditPost { id: i32 },
        #[route("/user/:id")]
        Profile { id: i32 },
        #[route("/user/:id/edit")]
        ProfileEdit { id: i32 },
        #[route("/users")]
        Users {},
        #[route("/search")]
        Search {},
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/forgot-password")]
        ForgotPassword {},
        #[route("/reset-password")]
        ResetPassword {},
        #[route("/admin")]
        Admin {},
        #[route("/help")]
        Help {},
        #[route("/rules")]
        Rules {},
}

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    {
        let _ = dotenvy::dotenv();
        // std::env::set_var("RUST_LOG", "warn");
        std::env::set_var("RUST_LOG", "sqlx=warn,info");
        dioxus::serve(|| async move {
            let router = dioxus::server::router(App)
                .route("/api/health", dioxus::server::axum::routing::get(healthcheck_handler))
                .route("/feed", dioxus::server::axum::routing::get(rss_feed_handler));
            Ok(router)
        });
    }
}

#[cfg(feature = "server")]
async fn healthcheck_handler() -> &'static str {
    "OK"
}

#[cfg(feature = "server")]
async fn rss_feed_handler() -> dioxus::server::axum::response::Response {
    use dioxus::server::axum::response::Response;

    match crate::data::generate_rss_feed().await {
        Ok(xml) => Response::builder()
            .header("content-type", "application/rss+xml; charset=utf-8")
            .body(dioxus::server::axum::body::Body::from(xml))
            .unwrap(),
        Err(_) => Response::builder()
            .status(500)
            .body(dioxus::server::axum::body::Body::from(
                "Error generating feed",
            ))
            .unwrap(),
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
