use dioxus::prelude::*;

use views::{Admin, AppShell, Forum, Index, Login, Register, Search, Topic, Users};

mod components;
mod data;
mod views;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Index {},
        #[route("/forum/:id")]
        Forum { id: i32 },
        #[route("/topic/:id")]
        Topic { id: i32 },
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
