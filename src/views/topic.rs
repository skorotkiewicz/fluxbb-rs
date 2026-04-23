use dioxus::prelude::*;

use crate::{
    components::{EmptyState, PostCard, TopicStatusBadge},
    data::AppData,
    Route,
};

#[component]
pub fn Topic(id: i32) -> Element {
    let board = use_context::<AppData>();

    let Some(topic) = board.topic(id) else {
        return rsx! {
            section { class: "page",
                EmptyState {
                    title: "Topic not found".to_string(),
                    body: "This topic id is not present in the Postgres-backed board data.".to_string(),
                }
            }
        };
    };

    let forum = board.forum(topic.forum_id);
    let posts = board.posts_for_topic(id);

    rsx! {
        section { class: "page",
            nav { class: "breadcrumbs",
                Link { to: Route::Index {}, "Forums" }
                if let Some(forum) = forum.clone() {
                    span { "/" }
                    Link { to: Route::Forum { id: forum.id }, "{forum.name}" }
                }
                span { "/" }
                span { "{topic.subject}" }
            }

            article { class: "hero-card compact-hero",
                div { class: "topic-hero-topline",
                    TopicStatusBadge { status: topic.status.clone() }
                    p { class: "topic-tags", "{topic.tags.join(\" | \")}" }
                }
                h2 { class: "topic-title", "{topic.subject}" }
                p { class: "topic-summary", "Views: {topic.views} | Replies: {topic.reply_count(&board)} | Updated: {topic.updated_at}" }
            }

            for post in posts {
                if let Some(author) = board.user(post.author_id) {
                    PostCard {
                        author_name: author.username,
                        author_role: author.title,
                        post,
                    }
                }
            }
        }
    }
}
