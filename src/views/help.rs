use dioxus::prelude::*;

use crate::components::SectionHeader;

#[component]
pub fn Help() -> Element {
    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Documentation".to_string(),
                    title: "Help".to_string(),
                    subtitle: "Formatting and community guidelines.".to_string(),
                }
            }

            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "BBCode reference" }
                }
                div { class: "help-content",
                    p { "Posts support a simple formatting syntax called BBCode." }

                    h4 { "Text styles" }
                    div { class: "help-row",
                        code { "[b]Bold[/b]" }
                        span { "→" }
                        b { "Bold" }
                    }
                    div { class: "help-row",
                        code { "[i]Italic[/i]" }
                        span { "→" }
                        i { "Italic" }
                    }
                    div { class: "help-row",
                        code { "[u]Underline[/u]" }
                        span { "→" }
                        u { "Underline" }
                    }
                    div { class: "help-row",
                        code { "[s]Strikethrough[/s]" }
                        span { "→" }
                        s { "Strikethrough" }
                    }

                    h4 { "Links and images" }
                    div { class: "help-row",
                        code { "[url=https://example.com]Link[/url]" }
                        span { "→" }
                        a { href: "#", "Link" }
                    }
                    div { class: "help-row",
                        code { "[url]https://example.com[/url]" }
                        span { "→" }
                        a { href: "#", "https://example.com" }
                    }
                    div { class: "help-row",
                        code { "[email]name@example.com[/email]" }
                        span { "→" }
                        a { href: "mailto:name@example.com", "name@example.com" }
                    }

                    h4 { "Quotes" }
                    div { class: "help-row",
                        code { "[quote=Author]Quoted text[/quote]" }
                        span { "→" }
                        blockquote { "Quoted text" }
                    }
                    div { class: "help-row",
                        code { "[quote]Quoted text[/quote]" }
                        span { "→" }
                        blockquote { "Quoted text" }
                    }

                    h4 { "Code" }
                    div { class: "help-row",
                        code { "[code]code block[/code]" }
                        span { "→" }
                        code { "code block" }
                    }

                    h4 { "Lists" }
                    div { class: "help-row",
                        code { "[list][*]Item[*]Item[/list]" }
                        span { "→" }
                        ul {
                            li { "Item" }
                            li { "Item" }
                        }
                    }

                    h4 { "Headings" }
                    div { class: "help-row",
                        code { "[h]Heading[/h]" }
                        span { "→" }
                        h5 { "Heading" }
                    }

                    h4 { "Colors" }
                    div { class: "help-row",
                        code { "[color=#FF0000]Red[/color]" }
                        span { "→" }
                        span { style: "color: #ff0000", "Red" }
                    }
                }
            }

            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "Community guidelines" }
                }
                div { class: "help-content",
                    p { "Be respectful to other members." }
                    p { "Stay on topic within each forum." }
                    p { "Do not post spam or unsolicited advertising." }
                    p { "Use clear subject lines so others can find your topic." }
                    p { "Search before starting a new topic to avoid duplicates." }
                }
            }
        }
    }
}
