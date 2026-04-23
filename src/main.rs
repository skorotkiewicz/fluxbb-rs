use dioxus::prelude::*;

use views::{
    Admin, AppShell, EditPost, Forum, ForumPage, Help, Index, Install, Login, NewTopic, Profile,
    ProfileEdit, Register, Rules, Search, Topic, TopicPage, Users,
};

mod components;
mod data;
mod views;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

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
        #[route("/admin")]
        Admin {},
        #[route("/help")]
        Help {},
        #[route("/rules")]
        Rules {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
