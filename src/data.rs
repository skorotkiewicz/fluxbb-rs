use std::{cmp::Reverse, process::Command};

use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const DATABASE_URL: &str = "postgresql://dev:password@localhost:5432/fluxbb";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub users: Vec<UserProfile>,
    pub topics: Vec<Topic>,
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardMeta {
    pub title: String,
    pub tagline: String,
    pub announcement_title: String,
    pub announcement_body: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forum {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub moderators: Vec<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub status: String,
    pub joined_at: String,
    pub post_count: i32,
    pub location: String,
    pub about: String,
    pub last_seen: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub id: i32,
    pub forum_id: i32,
    pub author_id: i32,
    pub subject: String,
    pub status: TopicStatus,
    pub views: i32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub activity_rank: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub topic_id: i32,
    pub author_id: i32,
    pub posted_at: String,
    pub edited_at: Option<String>,
    pub body: Vec<String>,
    pub signature: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicStatus {
    Pinned,
    Hot,
    Resolved,
    Fresh,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardStats {
    pub members: usize,
    pub topics: usize,
    pub posts: usize,
    pub newest_member: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForumSnapshot {
    pub forum: Forum,
    pub topic_count: usize,
    pub post_count: usize,
    pub last_topic_id: i32,
    pub last_topic_subject: String,
    pub last_post_author: String,
    pub last_posted_at: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchResults {
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
}

impl AppData {
    pub fn fallback() -> Self {
        Self {
            meta: BoardMeta {
                title: "FluxBB RS".to_string(),
                tagline: "FluxBB reimagined as a Dioxus 0.7 forum shell with Postgres-backed content.".to_string(),
                announcement_title: "Migration alpha".to_string(),
                announcement_body: "The board layout, topic browsing, member directory, and search now live in Rust. Posting and moderation flows are staged for the next slice.".to_string(),
            },
            categories: vec![
                Category {
                    id: 1,
                    name: "Announcements".to_string(),
                    description: "Project direction, release notes, and migration status.".to_string(),
                    sort_order: 1,
                },
                Category {
                    id: 2,
                    name: "Community".to_string(),
                    description: "Discussion spaces that mirror the core public FluxBB experience.".to_string(),
                    sort_order: 2,
                },
                Category {
                    id: 3,
                    name: "Workshop".to_string(),
                    description: "Implementation notes for the Dioxus and Rust rewrite.".to_string(),
                    sort_order: 3,
                },
            ],
            forums: vec![
                Forum {
                    id: 1,
                    category_id: 1,
                    name: "Release Notes".to_string(),
                    description: "Track each migration milestone and the current web parity status.".to_string(),
                    moderators: vec!["nora".to_string()],
                    sort_order: 1,
                },
                Forum {
                    id: 2,
                    category_id: 1,
                    name: "Migration Lab".to_string(),
                    description: "Patterns for moving classic FluxBB screens into RSX and fullstack Rust.".to_string(),
                    moderators: vec!["nora".to_string(), "ellis".to_string()],
                    sort_order: 2,
                },
                Forum {
                    id: 3,
                    category_id: 2,
                    name: "General Discussion".to_string(),
                    description: "Open-ended board conversation and showcase threads.".to_string(),
                    moderators: vec!["rhea".to_string()],
                    sort_order: 1,
                },
                Forum {
                    id: 4,
                    category_id: 2,
                    name: "Support".to_string(),
                    description: "Configuration, deployment, and migration troubleshooting.".to_string(),
                    moderators: vec!["kai".to_string()],
                    sort_order: 2,
                },
                Forum {
                    id: 5,
                    category_id: 3,
                    name: "Themes and Extensions".to_string(),
                    description: "Style ports, plugin ideas, and frontend experiments.".to_string(),
                    moderators: vec!["mira".to_string()],
                    sort_order: 1,
                },
                Forum {
                    id: 6,
                    category_id: 3,
                    name: "Backend Planning".to_string(),
                    description: "Database, auth, and moderation architecture for the Rust stack.".to_string(),
                    moderators: vec!["ellis".to_string()],
                    sort_order: 2,
                },
            ],
            users: vec![
                UserProfile {
                    id: 1,
                    username: "nora".to_string(),
                    title: "Administrator".to_string(),
                    status: "Online".to_string(),
                    joined_at: "2026-04-01".to_string(),
                    post_count: 482,
                    location: "Berlin".to_string(),
                    about: "Maintains the Dioxus rewrite and release cadence.".to_string(),
                    last_seen: "a minute ago".to_string(),
                },
                UserProfile {
                    id: 2,
                    username: "ellis".to_string(),
                    title: "Core Maintainer".to_string(),
                    status: "Online".to_string(),
                    joined_at: "2026-04-02".to_string(),
                    post_count: 318,
                    location: "Copenhagen".to_string(),
                    about: "Focuses on schema design, hydration safety, and deployment ergonomics.".to_string(),
                    last_seen: "5 minutes ago".to_string(),
                },
                UserProfile {
                    id: 3,
                    username: "rhea".to_string(),
                    title: "Moderator".to_string(),
                    status: "Reviewing".to_string(),
                    joined_at: "2026-04-03".to_string(),
                    post_count: 211,
                    location: "Amsterdam".to_string(),
                    about: "Moderation lead and theme curator.".to_string(),
                    last_seen: "20 minutes ago".to_string(),
                },
                UserProfile {
                    id: 4,
                    username: "kai".to_string(),
                    title: "Support Lead".to_string(),
                    status: "Available".to_string(),
                    joined_at: "2026-04-05".to_string(),
                    post_count: 177,
                    location: "Lisbon".to_string(),
                    about: "Handles onboarding, FAQ cleanup, and migration support.".to_string(),
                    last_seen: "12 minutes ago".to_string(),
                },
                UserProfile {
                    id: 5,
                    username: "mira".to_string(),
                    title: "Theme Builder".to_string(),
                    status: "Designing".to_string(),
                    joined_at: "2026-04-09".to_string(),
                    post_count: 124,
                    location: "Prague".to_string(),
                    about: "Ports classic board themes into the new CSS system.".to_string(),
                    last_seen: "an hour ago".to_string(),
                },
                UserProfile {
                    id: 6,
                    username: "sol".to_string(),
                    title: "Member".to_string(),
                    status: "Reading".to_string(),
                    joined_at: "2026-04-20".to_string(),
                    post_count: 14,
                    location: "Warsaw".to_string(),
                    about: "Testing the first public preview and reporting rough edges.".to_string(),
                    last_seen: "just now".to_string(),
                },
            ],
            topics: vec![
                Topic {
                    id: 101,
                    forum_id: 1,
                    author_id: 1,
                    subject: "0.1 migration alpha is live".to_string(),
                    status: TopicStatus::Pinned,
                    views: 934,
                    tags: vec!["release".to_string(), "migration".to_string()],
                    created_at: "2026-04-20 09:15 UTC".to_string(),
                    updated_at: "2026-04-23 19:40 UTC".to_string(),
                    activity_rank: 400,
                },
                Topic {
                    id: 102,
                    forum_id: 2,
                    author_id: 2,
                    subject: "Mapping FluxBB templates to RSX".to_string(),
                    status: TopicStatus::Hot,
                    views: 486,
                    tags: vec!["rsx".to_string(), "layout".to_string()],
                    created_at: "2026-04-19 13:10 UTC".to_string(),
                    updated_at: "2026-04-23 18:05 UTC".to_string(),
                    activity_rank: 390,
                },
                Topic {
                    id: 201,
                    forum_id: 3,
                    author_id: 3,
                    subject: "Show your forum theme experiments".to_string(),
                    status: TopicStatus::Fresh,
                    views: 212,
                    tags: vec!["theme".to_string(), "showcase".to_string()],
                    created_at: "2026-04-18 17:45 UTC".to_string(),
                    updated_at: "2026-04-22 16:55 UTC".to_string(),
                    activity_rank: 360,
                },
                Topic {
                    id: 202,
                    forum_id: 4,
                    author_id: 4,
                    subject: "Session strategy for web-only deploys".to_string(),
                    status: TopicStatus::Resolved,
                    views: 305,
                    tags: vec!["auth".to_string(), "deployment".to_string()],
                    created_at: "2026-04-17 10:20 UTC".to_string(),
                    updated_at: "2026-04-22 11:35 UTC".to_string(),
                    activity_rank: 340,
                },
                Topic {
                    id: 301,
                    forum_id: 5,
                    author_id: 5,
                    subject: "Theme pack wishlist".to_string(),
                    status: TopicStatus::Hot,
                    views: 267,
                    tags: vec!["css".to_string(), "theme".to_string()],
                    created_at: "2026-04-16 15:00 UTC".to_string(),
                    updated_at: "2026-04-21 20:20 UTC".to_string(),
                    activity_rank: 320,
                },
                Topic {
                    id: 302,
                    forum_id: 6,
                    author_id: 2,
                    subject: "Persistence layer options for a post-PHP stack".to_string(),
                    status: TopicStatus::Pinned,
                    views: 522,
                    tags: vec!["postgres".to_string(), "architecture".to_string()],
                    created_at: "2026-04-15 08:55 UTC".to_string(),
                    updated_at: "2026-04-23 17:25 UTC".to_string(),
                    activity_rank: 380,
                },
            ],
            posts: vec![
                Post {
                    id: 1001,
                    topic_id: 101,
                    author_id: 1,
                    posted_at: "2026-04-20 09:15 UTC".to_string(),
                    edited_at: Some("2026-04-23 19:05 UTC".to_string()),
                    body: vec![
                        "The first Rust and Dioxus web slice is up. Board index, forum view, topic view, search, and member directory are all wired.".to_string(),
                        "Next up are posting workflows, session-backed authentication, and moderation actions.".to_string(),
                    ],
                    signature: Some("Keep migrations boring.".to_string()),
                    position: 1,
                },
                Post {
                    id: 1002,
                    topic_id: 101,
                    author_id: 6,
                    posted_at: "2026-04-23 19:40 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "The browsing flow feels solid. The search page made it much easier to see where the PHP information architecture landed in the new app.".to_string(),
                    ],
                    signature: None,
                    position: 2,
                },
                Post {
                    id: 1101,
                    topic_id: 102,
                    author_id: 2,
                    posted_at: "2026-04-19 13:10 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "The old header and body templates map cleanly to a shared shell plus route pages. The tricky part is keeping table-heavy layouts readable on mobile.".to_string(),
                    ],
                    signature: None,
                    position: 1,
                },
                Post {
                    id: 1102,
                    topic_id: 102,
                    author_id: 5,
                    posted_at: "2026-04-23 18:05 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "I had the best results by preserving the forum-table mental model but shifting the last-post metadata into a stacked layout on narrow widths.".to_string(),
                    ],
                    signature: Some("Visual parity matters, but not at the expense of scan speed.".to_string()),
                    position: 2,
                },
                Post {
                    id: 1201,
                    topic_id: 201,
                    author_id: 3,
                    posted_at: "2026-04-18 17:45 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "If you are testing theme ideas, post screenshots and note whether you are optimizing for nostalgia, readability, or moderation-heavy workflows.".to_string(),
                    ],
                    signature: None,
                    position: 1,
                },
                Post {
                    id: 1202,
                    topic_id: 201,
                    author_id: 5,
                    posted_at: "2026-04-22 16:55 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "Copper accents plus parchment cards ended up closer to the old FluxBB tone than another dark dashboard treatment.".to_string(),
                    ],
                    signature: None,
                    position: 2,
                },
                Post {
                    id: 1301,
                    topic_id: 202,
                    author_id: 4,
                    posted_at: "2026-04-17 10:20 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "For the first web deploy, I would keep auth server-backed and session cookie based. JWT would only add edge cases we do not need yet.".to_string(),
                    ],
                    signature: None,
                    position: 1,
                },
                Post {
                    id: 1302,
                    topic_id: 202,
                    author_id: 2,
                    posted_at: "2026-04-22 11:35 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "Agreed. The app shell already centralizes enough data that classic session handling will be the lowest-risk path for the next increment.".to_string(),
                    ],
                    signature: None,
                    position: 2,
                },
                Post {
                    id: 1401,
                    topic_id: 301,
                    author_id: 5,
                    posted_at: "2026-04-16 15:00 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "Current shortlist: Air-inspired light theme, a sharper editorial variant, and a contrast-first moderation skin.".to_string(),
                    ],
                    signature: None,
                    position: 1,
                },
                Post {
                    id: 1402,
                    topic_id: 301,
                    author_id: 3,
                    posted_at: "2026-04-21 20:20 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "Please keep moderation cues strong. Reports, locked topics, and read state should stay obvious at a glance.".to_string(),
                    ],
                    signature: None,
                    position: 2,
                },
                Post {
                    id: 1501,
                    topic_id: 302,
                    author_id: 2,
                    posted_at: "2026-04-15 08:55 UTC".to_string(),
                    edited_at: Some("2026-04-23 16:10 UTC".to_string()),
                    body: vec![
                        "The migration stack needs normalized categories, forums, topics, posts, and users first. Anything more can layer on after the public flows are stable.".to_string(),
                        "Postgres is enough for the first backend pass; the schema should stay friendly to future search indexing and moderation queues.".to_string(),
                    ],
                    signature: None,
                    position: 1,
                },
                Post {
                    id: 1502,
                    topic_id: 302,
                    author_id: 1,
                    posted_at: "2026-04-23 17:25 UTC".to_string(),
                    edited_at: None,
                    body: vec![
                        "That is the direction I am landing in this slice as well. Keep the schema obvious, then layer richer server functions on top.".to_string(),
                    ],
                    signature: Some("Correctness before cleverness.".to_string()),
                    position: 2,
                },
            ],
        }
    }

    pub fn board_stats(&self) -> BoardStats {
        let newest_member = self
            .users
            .iter()
            .max_by_key(|user| user.id)
            .map(|user| user.username.clone())
            .unwrap_or_else(|| "guest".to_string());

        BoardStats {
            members: self.users.len(),
            topics: self.topics.len(),
            posts: self.posts.len(),
            newest_member,
        }
    }

    pub fn categories_sorted(&self) -> Vec<Category> {
        let mut categories = self.categories.clone();
        categories.sort_by_key(|category| category.sort_order);
        categories
    }

    pub fn forums_in_category(&self, category_id: i32) -> Vec<Forum> {
        let mut forums = self
            .forums
            .iter()
            .filter(|forum| forum.category_id == category_id)
            .cloned()
            .collect::<Vec<_>>();
        forums.sort_by_key(|forum| forum.sort_order);
        forums
    }

    pub fn forum(&self, id: i32) -> Option<Forum> {
        self.forums.iter().find(|forum| forum.id == id).cloned()
    }

    pub fn user(&self, id: i32) -> Option<UserProfile> {
        self.users.iter().find(|user| user.id == id).cloned()
    }

    pub fn topic(&self, id: i32) -> Option<Topic> {
        self.topics.iter().find(|topic| topic.id == id).cloned()
    }

    pub fn topics_for_forum(&self, forum_id: i32) -> Vec<Topic> {
        let mut topics = self
            .topics
            .iter()
            .filter(|topic| topic.forum_id == forum_id)
            .cloned()
            .collect::<Vec<_>>();
        topics.sort_by_key(|topic| Reverse(topic.activity_rank));
        topics
    }

    pub fn posts_for_topic(&self, topic_id: i32) -> Vec<Post> {
        let mut posts = self
            .posts
            .iter()
            .filter(|post| post.topic_id == topic_id)
            .cloned()
            .collect::<Vec<_>>();
        posts.sort_by_key(|post| post.position);
        posts
    }

    pub fn recent_topics(&self, limit: usize) -> Vec<Topic> {
        let mut topics = self.topics.clone();
        topics.sort_by_key(|topic| Reverse(topic.activity_rank));
        topics.into_iter().take(limit).collect()
    }

    pub fn forum_snapshot(&self, forum_id: i32) -> Option<ForumSnapshot> {
        let forum = self.forum(forum_id)?;
        let topics = self.topics_for_forum(forum_id);
        let topic_count = topics.len();
        let post_count = topics
            .iter()
            .map(|topic| self.posts_for_topic(topic.id).len())
            .sum::<usize>();
        let latest_topic = topics.first()?.clone();
        let latest_post = self.posts_for_topic(latest_topic.id).last()?.clone();
        let latest_user = self.user(latest_post.author_id)?;

        Some(ForumSnapshot {
            forum,
            topic_count,
            post_count,
            last_topic_id: latest_topic.id,
            last_topic_subject: latest_topic.subject,
            last_post_author: latest_user.username,
            last_posted_at: latest_post.posted_at,
        })
    }

    pub fn search(&self, query: &str) -> SearchResults {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return SearchResults::default();
        }

        let topics = self
            .topics
            .iter()
            .filter(|topic| {
                topic.subject.to_lowercase().contains(&needle)
                    || topic
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&needle))
                    || self
                        .posts_for_topic(topic.id)
                        .iter()
                        .flat_map(|post| post.body.iter())
                        .any(|paragraph| paragraph.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect::<Vec<_>>();

        let users = self
            .users
            .iter()
            .filter(|user| {
                user.username.to_lowercase().contains(&needle)
                    || user.title.to_lowercase().contains(&needle)
                    || user.about.to_lowercase().contains(&needle)
                    || user.location.to_lowercase().contains(&needle)
            })
            .cloned()
            .collect::<Vec<_>>();

        SearchResults { topics, users }
    }
}

impl Topic {
    pub fn reply_count(&self, board: &AppData) -> usize {
        board.posts_for_topic(self.id).len().saturating_sub(1)
    }
}

impl TopicStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pinned => "Pinned",
            Self::Hot => "Hot",
            Self::Resolved => "Resolved",
            Self::Fresh => "Fresh",
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Pinned => "badge badge-pinned",
            Self::Hot => "badge badge-hot",
            Self::Resolved => "badge badge-resolved",
            Self::Fresh => "badge badge-fresh",
        }
    }
}

#[server]
pub async fn load_board() -> Result<AppData, ServerFnError> {
    #[cfg(feature = "server")]
    {
        Ok(load_board_from_postgres().unwrap_or_else(|_| AppData::fallback()))
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(AppData::fallback())
    }
}

#[cfg(feature = "server")]
fn load_board_from_postgres() -> Result<AppData, String> {
    let meta = run_json_query::<Vec<BoardMeta>>(
        "SELECT COALESCE(json_agg(row_to_json(meta_row)), '[]'::json)
         FROM (
             SELECT title, tagline, announcement_title, announcement_body
             FROM board_meta
             ORDER BY id
         ) AS meta_row;",
    )?;

    let categories = run_json_query::<Vec<Category>>(
        "SELECT COALESCE(json_agg(row_to_json(category_row)), '[]'::json)
         FROM (
             SELECT id, name, description, sort_order
             FROM categories
             ORDER BY sort_order, id
         ) AS category_row;",
    )?;

    let forums = run_json_query::<Vec<Forum>>(
        "SELECT COALESCE(json_agg(row_to_json(forum_row)), '[]'::json)
         FROM (
             SELECT id, category_id, name, description, moderators, sort_order
             FROM forums
             ORDER BY category_id, sort_order, id
         ) AS forum_row;",
    )?;

    let users = run_json_query::<Vec<UserProfile>>(
        "SELECT COALESCE(json_agg(row_to_json(user_row)), '[]'::json)
         FROM (
             SELECT id, username, title, status, joined_at, post_count, location, about, last_seen
             FROM users
             ORDER BY id
         ) AS user_row;",
    )?;

    let topics = run_json_query::<Vec<Topic>>(
        "SELECT COALESCE(json_agg(row_to_json(topic_row)), '[]'::json)
         FROM (
             SELECT id, forum_id, author_id, subject, status, views, tags, created_at, updated_at, activity_rank
             FROM topics
             ORDER BY activity_rank DESC, id
         ) AS topic_row;",
    )?;

    let posts = run_json_query::<Vec<Post>>(
        "SELECT COALESCE(json_agg(row_to_json(post_row)), '[]'::json)
         FROM (
             SELECT id, topic_id, author_id, posted_at, edited_at, body, signature, position
             FROM posts
             ORDER BY topic_id, position, id
         ) AS post_row;",
    )?;

    Ok(AppData {
        meta: meta
            .into_iter()
            .next()
            .unwrap_or_else(|| AppData::fallback().meta),
        categories,
        forums,
        users,
        topics,
        posts,
    })
}

#[cfg(feature = "server")]
fn run_json_query<T>(sql: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let output = Command::new("psql")
        .arg(DATABASE_URL)
        .args(["-X", "-t", "-A", "-c", sql])
        .output()
        .map_err(|error| format!("failed to run psql: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("psql returned non-utf8 output: {error}"))?;
    let payload = stdout.trim();

    serde_json::from_str(payload).map_err(|error| format!("failed to parse postgres json: {error}"))
}
