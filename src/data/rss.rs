use dioxus::prelude::*;

#[cfg(feature = "server")]
use super::db::{run_json_query, server_error};
#[cfg(feature = "server")]
use super::Topic;

#[post("/api/rss")]
pub async fn generate_rss_feed() -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let meta = run_json_query::<Option<serde_json::Value>>(
            "SELECT COALESCE((SELECT row_to_json(m) FROM (SELECT title, tagline FROM board_meta LIMIT 1) m), 'null'::json);"
        )
        .await
        .map_err(server_error)?;

        let board_title = meta
            .as_ref()
            .and_then(|m| m.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("FluxBB Forum")
            .to_string();
        let board_tagline = meta
            .as_ref()
            .and_then(|m| m.get("tagline"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let topics = run_json_query::<Vec<Topic>>(
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
                SELECT id, forum_id, author_id, subject, closed, views, tags, created_at, updated_at, activity_rank, sticky, moved_to
                FROM topics ORDER BY updated_at DESC LIMIT 25
            ) t;",
        )
        .await
        .map_err(server_error)?;

        let mut xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
<title>{}</title>
<link>/</link>
<description>{}</description>
<language>en</language>
"#,
            xml_escape(&board_title),
            xml_escape(&board_tagline),
        );

        for topic in topics {
            xml.push_str(&format!(
                r#"<item>
<title>{}</title>
<link>/topic/{}</link>
<pubDate>{}</pubDate>
<guid>/topic/{}</guid>
</item>
"#,
                xml_escape(&topic.subject),
                topic.id,
                topic.updated_at,
                topic.id,
            ));
        }

        xml.push_str("</channel>\n</rss>");

        Ok(xml)
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(feature = "server")]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
