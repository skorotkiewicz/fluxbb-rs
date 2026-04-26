//! BBCode parser and renderer for forum posts
//!
//! Supports standard FluxBB BBCode tags:
//! - [b], [i], [u], [s] - text formatting
//! - [quote], [quote=author] - quotations
//! - [code] - code blocks
//! - [url], [url=href] - links
//! - [img] - images
//! - [list], [*] - lists
//! - [email] - email links
//! - [color], [size] - styling

use std::borrow::Cow;

/// A BBCode AST node
#[derive(Debug, Clone, PartialEq)]
pub enum BBCodeNode<'a> {
    Text(Cow<'a, str>),
    Bold(Vec<BBCodeNode<'a>>),
    Italic(Vec<BBCodeNode<'a>>),
    Underline(Vec<BBCodeNode<'a>>),
    Strikethrough(Vec<BBCodeNode<'a>>),
    Quote(Option<Cow<'a, str>>, Vec<BBCodeNode<'a>>),
    Code(Option<Cow<'a, str>>, Cow<'a, str>), // language, content
    Url(Option<Cow<'a, str>>, Vec<BBCodeNode<'a>>), // href, content
    Img(Cow<'a, str>),
    Email(Option<Cow<'a, str>>, Vec<BBCodeNode<'a>>), // href, content
    Color(Cow<'a, str>, Vec<BBCodeNode<'a>>),         // color value
    Size(Cow<'a, str>, Vec<BBCodeNode<'a>>),          // size value
    Font(FontAttributes<'a>, Vec<BBCodeNode<'a>>),    // font family + size
    List(Option<Cow<'a, str>>, Vec<BBCodeNode<'a>>),  // type (1, a, A, etc), items
    ListItem(Vec<BBCodeNode<'a>>),
    LineBreak,
}

/// Font attributes for [font] tag
#[derive(Debug, Clone, PartialEq)]
pub struct FontAttributes<'a> {
    pub family: Option<Cow<'a, str>>,
    pub size: Option<Cow<'a, str>>,
}

/// Parse BBCode string into an AST
pub fn parse_bbcode(input: &str) -> Vec<BBCodeNode<'_>> {
    let mut parser = BBCodeParser::new(input);
    parser.parse_nodes_until(None)
}

/// Render BBCode AST to HTML string
pub fn render_to_html(nodes: &[BBCodeNode<'_>]) -> String {
    let mut output = String::new();
    render_nodes(&mut output, nodes);
    output
}

fn render_nodes(output: &mut String, nodes: &[BBCodeNode<'_>]) {
    for node in nodes {
        render_node(output, node);
    }
}

fn render_node(output: &mut String, node: &BBCodeNode<'_>) {
    match node {
        BBCodeNode::Text(text) => {
            output.push_str(&escape_html(text));
        }
        BBCodeNode::LineBreak => {
            output.push_str("<br>");
        }
        BBCodeNode::Bold(children) => {
            output.push_str("<strong>");
            render_nodes(output, children);
            output.push_str("</strong>");
        }
        BBCodeNode::Italic(children) => {
            output.push_str("<em>");
            render_nodes(output, children);
            output.push_str("</em>");
        }
        BBCodeNode::Underline(children) => {
            output.push_str("<u>");
            render_nodes(output, children);
            output.push_str("</u>");
        }
        BBCodeNode::Strikethrough(children) => {
            output.push_str("<del>");
            render_nodes(output, children);
            output.push_str("</del>");
        }
        BBCodeNode::Quote(author, children) => {
            output.push_str(r#"<blockquote class="bbcode-quote">"#);
            if let Some(author) = author {
                output.push_str(&format!(
                    r#"<cite class="bbcode-quote-author">{} wrote:</cite>"#,
                    escape_html(author)
                ));
            }
            output.push_str("<div class=\"bbcode-quote-content\">");
            render_nodes(output, children);
            output.push_str("</div></blockquote>");
        }
        BBCodeNode::Code(lang, code) => {
            if let Some(lang) = lang {
                let lang = escape_html(lang);
                output.push_str(&format!(
                    r#"<pre class=\"bbcode-code\"><code class=\"language-{}\">"#,
                    lang
                ));
            } else {
                output.push_str("<pre class=\"bbcode-code\"><code>");
            }
            output.push_str(&escape_html(code));
            output.push_str("</code></pre>");
        }
        BBCodeNode::Url(Some(href), children) => {
            let href = sanitize_url(href);
            output.push_str(&format!(
                r#"<a href="{}" class="bbcode-url" target="_blank" rel="noopener noreferrer">"#,
                href,
            ));
            render_nodes(output, children);
            output.push_str("</a>");
        }
        BBCodeNode::Url(None, children) => {
            let mut content = String::new();
            render_nodes(&mut content, children);
            let href = sanitize_url(&content);
            output.push_str(&format!(r#"<a href="{}" class="bbcode-url" target="_blank" rel="noopener noreferrer">{}</a>"#, href, content));
        }
        BBCodeNode::Img(src) => {
            let src = sanitize_url(src);
            output.push_str(&format!(r#"<img src="{}" class="bbcode-img" alt="">"#, src));
        }
        BBCodeNode::Email(Some(href), children) => {
            let href_lower = href.trim().to_lowercase();
            if !href_lower.starts_with("mailto:") && !href_lower.contains('@') {
                return;
            }
            let href = escape_html(href);
            output.push_str(&format!(
                r#"<a href="mailto:{}" class="bbcode-email">"#,
                href
            ));
            render_nodes(output, children);
            output.push_str("</a>");
        }
        BBCodeNode::Email(None, children) => {
            let mut content = String::new();
            render_nodes(&mut content, children);
            let href = sanitize_url(&content);
            output.push_str(&format!(
                r#"<a href="mailto:{}" class="bbcode-email">{}</a>"#,
                href, content
            ));
        }
        BBCodeNode::Color(color, children) => {
            let color = escape_html(color);
            output.push_str(&format!(
                r#"<span style="color: {}" class="bbcode-color">"#,
                color
            ));
            render_nodes(output, children);
            output.push_str("</span>");
        }
        BBCodeNode::Size(size, children) => {
            // Parse size - could be pixels or relative (1-7 like old BBCode)
            let size_value = if let Ok(num) = size.parse::<i32>() {
                // Convert 1-7 scale to pixels
                match num {
                    1 => "9px",
                    2 => "10px",
                    3 => "13px",
                    4 => "16px",
                    5 => "18px",
                    6 => "24px",
                    7 => "36px",
                    _ => "16px",
                }
                .to_string()
            } else {
                // Use as-is (e.g., "12px", "1.5em")
                escape_html(size)
            };
            output.push_str(&format!(
                r#"<span style=\"font-size: {}\" class=\"bbcode-size\">"#,
                size_value
            ));
            render_nodes(output, children);
            output.push_str("</span>");
        }
        BBCodeNode::Font(attrs, children) => {
            let mut style_parts = Vec::new();
            if let Some(family) = &attrs.family {
                let family = escape_html(family);
                style_parts.push(format!("font-family: {}", family));
            }
            if let Some(size) = &attrs.size {
                let size_value = if let Ok(num) = size.parse::<i32>() {
                    match num {
                        1 => "9px".to_string(),
                        2 => "10px".to_string(),
                        3 => "13px".to_string(),
                        4 => "16px".to_string(),
                        5 => "18px".to_string(),
                        6 => "24px".to_string(),
                        7 => "36px".to_string(),
                        _ => escape_html(size),
                    }
                } else {
                    escape_html(size)
                };
                style_parts.push(format!("font-size: {}", size_value));
            }
            let style = style_parts.join("; ");
            output.push_str(&format!(
                r#"<span style=\"{}\" class=\"bbcode-font\">"#,
                style
            ));
            render_nodes(output, children);
            output.push_str("</span>");
        }
        BBCodeNode::List(list_type, children) => match list_type.as_deref() {
            Some("1") | Some("a") | Some("A") | Some("i") | Some("I") => {
                let attr = match list_type.as_deref() {
                    Some("a") => r#" type=\"a\""#,
                    Some("A") => r#" type=\"A\""#,
                    Some("i") => r#" type=\"i\""#,
                    Some("I") => r#" type=\"I\""#,
                    _ => "",
                };
                output.push_str(&format!(
                    r#"<ol class=\"bbcode-list bbcode-ordered-list\"{}>"#,
                    attr
                ));
                render_nodes(output, children);
                output.push_str("</ol>");
            }
            _ => {
                output.push_str("<ul class=\"bbcode-list\">");
                render_nodes(output, children);
                output.push_str("</ul>");
            }
        },
        BBCodeNode::ListItem(children) => {
            output.push_str("<li>");
            render_nodes(output, children);
            output.push_str("</li>");
        }
    }
}

fn escape_html(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn sanitize_url(url: &str) -> String {
    let trimmed = url.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("javascript:") || lower.starts_with("data:") || lower.starts_with("vbscript:") {
        "#".to_string()
    } else {
        escape_html(trimmed)
    }
}

struct BBCodeParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> BBCodeParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_nodes_until(&mut self, stop_tag: Option<&str>) -> Vec<BBCodeNode<'a>> {
        let mut nodes = Vec::new();
        let mut text_start = self.pos;

        while self.pos < self.input.len() {
            if self.peek_char() == Some('\n') {
                // Add accumulated text before newline
                if self.pos > text_start {
                    let text = &self.input[text_start..self.pos];
                    if !text.is_empty() {
                        nodes.push(BBCodeNode::Text(Cow::Borrowed(text)));
                    }
                }
                self.pos += 1;
                text_start = self.pos;
                nodes.push(BBCodeNode::LineBreak);
                continue;
            }

            if self.peek_char() == Some('[') {
                // Save text before the tag
                if self.pos > text_start {
                    let text = &self.input[text_start..self.pos];
                    if !text.is_empty() {
                        nodes.push(BBCodeNode::Text(Cow::Borrowed(text)));
                    }
                }

                // Check for closing tag if we have a stop_tag
                if let Some(stop) = stop_tag {
                    if let Some(end_pos) = self.try_parse_closing_tag(stop) {
                        self.pos = end_pos;
                        return nodes;
                    }
                    // Special case: [*] inside a list acts as a list item delimiter
                    // When stop_tag is "*" (list item), we should also stop at the next [*] or [/list]
                    if stop == "*" {
                        // Check for next [*] or parent [/list]
                        if let Some((tag, _, _)) = self.try_parse_opening_tag_peek() {
                            if tag == "*" {
                                // Next item starts here - return current item's content
                                return nodes;
                            }
                        }
                        if self.try_parse_closing_tag("list").is_some() {
                            // End of list reached - return current item's content
                            return nodes;
                        }
                    }
                }

                // Try to parse an opening tag
                if let Some((tag, opt_value, content_end)) = self.try_parse_opening_tag() {
                    self.pos = content_end;
                    let children = self.parse_nodes_until(Some(&tag));

                    let node = match tag.as_str() {
                        "b" => BBCodeNode::Bold(children),
                        "i" => BBCodeNode::Italic(children),
                        "u" => BBCodeNode::Underline(children),
                        "s" => BBCodeNode::Strikethrough(children),
                        "quote" => BBCodeNode::Quote(opt_value.map(Cow::Owned), children),
                        "code" => {
                            // Code blocks don't parse children - treat as raw text
                            let lang = opt_value.map(Cow::Owned);
                            if let Some(BBCodeNode::Text(text)) = children.first() {
                                BBCodeNode::Code(lang, Cow::Owned(text.clone().into_owned()))
                            } else {
                                BBCodeNode::Code(lang, Cow::Owned(String::new()))
                            }
                        }
                        "url" => BBCodeNode::Url(opt_value.map(Cow::Owned), children),
                        "img" => {
                            if let Some(BBCodeNode::Text(text)) = children.first() {
                                BBCodeNode::Img(Cow::Owned(text.clone().into_owned()))
                            } else {
                                BBCodeNode::Img(Cow::Owned(String::new()))
                            }
                        }
                        "email" => BBCodeNode::Email(opt_value.map(Cow::Owned), children),
                        "color" => {
                            if let Some(color) = opt_value {
                                BBCodeNode::Color(Cow::Owned(color), children)
                            } else {
                                BBCodeNode::Text(Cow::Borrowed("[color]"))
                            }
                        }
                        "size" => {
                            if let Some(size) = opt_value {
                                BBCodeNode::Size(Cow::Owned(size), children)
                            } else {
                                BBCodeNode::Text(Cow::Borrowed("[size]"))
                            }
                        }
                        "list" => BBCodeNode::List(opt_value.map(Cow::Owned), children),
                        "*" => {
                            // List items are self-closing - they continue until next [*] or parent closing tag
                            // Since we passed Some("*") to parse_nodes_until, it will stop at the next [*]
                            // or when the parent [list] closing tag is encountered
                            BBCodeNode::ListItem(children)
                        }
                        "font" => {
                            // Parse font attributes: [font=Arial] or [font family=Arial size=12]
                            let mut attrs = FontAttributes {
                                family: None,
                                size: None,
                            };
                            if let Some(value) = opt_value {
                                // Check if it contains attribute pairs like "family=Arial size=12"
                                for part in value.split_whitespace() {
                                    if let Some(eq_idx) = part.find('=') {
                                        let key = part[..eq_idx].trim().to_lowercase();
                                        let val = part[eq_idx + 1..].trim().to_string();
                                        match key.as_str() {
                                            "family" => attrs.family = Some(Cow::Owned(val)),
                                            "size" => attrs.size = Some(Cow::Owned(val)),
                                            _ => {}
                                        }
                                    } else {
                                        // Simple value like [font=Arial] - treat as family
                                        attrs.family = Some(Cow::Owned(value.clone()));
                                        break;
                                    }
                                }
                            }
                            BBCodeNode::Font(attrs, children)
                        }
                        _ => {
                            // Unknown tag - treat as text
                            let mut text = String::new();
                            text.push('[');
                            text.push_str(&tag);
                            if let Some(val) = opt_value {
                                text.push('=');
                                text.push_str(&val);
                            }
                            text.push(']');
                            for child in children {
                                append_node_text(&mut text, &child);
                            }
                            text.push('[');
                            text.push('/');
                            text.push_str(&tag);
                            text.push(']');
                            BBCodeNode::Text(Cow::Owned(text))
                        }
                    };

                    nodes.push(node);
                    text_start = self.pos;
                } else {
                    // Not a valid tag - continue as text
                    self.pos += 1;
                }
            } else {
                self.pos += 1;
            }
        }

        // Add remaining text
        if self.pos > text_start {
            let text = &self.input[text_start..self.pos];
            if !text.is_empty() {
                nodes.push(BBCodeNode::Text(Cow::Borrowed(text)));
            }
        }

        nodes
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn try_parse_closing_tag(&self, expected: &str) -> Option<usize> {
        let remaining = &self.input[self.pos..];
        let expected_close = format!("[/{}]", expected);

        if remaining.starts_with(&expected_close) {
            Some(self.pos + expected_close.len())
        } else {
            None
        }
    }

    fn try_parse_opening_tag_peek(&self) -> Option<(String, Option<String>, usize)> {
        let remaining = &self.input[self.pos..];

        if !remaining.starts_with('[') {
            return None;
        }

        // Find the closing bracket
        let close_idx = remaining.find(']')?;
        let tag_content = &remaining[1..close_idx];

        // Check for closing tag marker
        if tag_content.starts_with('/') {
            return None;
        }

        // Parse tag name and optional value
        // Tag name ends at first space or '=' - everything after is attributes
        let first_space = tag_content.find(' ');
        let first_eq = tag_content.find('=');

        let (tag_name, opt_value) = match (first_space, first_eq) {
            // [tag=value] - simple attribute (no space before =)
            (None, Some(eq_idx)) => {
                let name = tag_content[..eq_idx].trim().to_lowercase();
                let value = tag_content[eq_idx + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag=value] with space after tag but = comes before the space
            (Some(s), Some(eq_idx)) if eq_idx < s => {
                let name = tag_content[..eq_idx].trim().to_lowercase();
                let value = tag_content[eq_idx + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag attr1=val1 attr2=val2] - multiple attributes, keep full value string
            (Some(s), _) => {
                let name = tag_content[..s].trim().to_lowercase();
                let value = tag_content[s + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag] - no attributes
            _ => (tag_content.trim().to_lowercase(), None),
        };

        // Validate tag name
        if tag_name.is_empty() || !is_valid_tag_name(&tag_name) {
            return None;
        }

        Some((tag_name, opt_value, self.pos + close_idx + 1))
    }

    fn try_parse_opening_tag(&mut self) -> Option<(String, Option<String>, usize)> {
        let remaining = &self.input[self.pos..];

        if !remaining.starts_with('[') {
            return None;
        }

        // Find the closing bracket
        let close_idx = remaining.find(']')?;
        let tag_content = &remaining[1..close_idx];

        // Check for closing tag marker
        if tag_content.starts_with('/') {
            return None;
        }

        // Parse tag name and optional value
        // Tag name ends at first space or '=' - everything after is attributes
        let first_space = tag_content.find(' ');
        let first_eq = tag_content.find('=');

        let (tag_name, opt_value) = match (first_space, first_eq) {
            // [tag=value] - simple attribute (no space before =)
            (None, Some(eq_idx)) => {
                let name = tag_content[..eq_idx].trim().to_lowercase();
                let value = tag_content[eq_idx + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag=value] with space after tag but = comes before the space
            (Some(s), Some(eq_idx)) if eq_idx < s => {
                let name = tag_content[..eq_idx].trim().to_lowercase();
                let value = tag_content[eq_idx + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag attr1=val1 attr2=val2] - multiple attributes, keep full value string
            (Some(s), _) => {
                let name = tag_content[..s].trim().to_lowercase();
                let value = tag_content[s + 1..].trim().to_string();
                (name, Some(value))
            }
            // [tag] - no attributes
            _ => (tag_content.trim().to_lowercase(), None),
        };

        // Validate tag name
        if tag_name.is_empty() || !is_valid_tag_name(&tag_name) {
            return None;
        }

        Some((tag_name, opt_value, self.pos + close_idx + 1))
    }
}

fn is_valid_tag_name(name: &str) -> bool {
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '*')
}

fn append_node_text(output: &mut String, node: &BBCodeNode<'_>) {
    match node {
        BBCodeNode::Text(text) => output.push_str(text),
        BBCodeNode::LineBreak => output.push('\n'),
        _ => {
            // For other nodes, just serialize them back
            // (simplified - not perfect but works for error cases)
        }
    }
}

/// Render a paragraph of BBCode to HTML
pub fn render_paragraph(input: &str) -> String {
    let nodes = parse_bbcode(input);
    render_to_html(&nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let input = "[b]bold[/b] and [i]italic[/i]";
        let nodes = parse_bbcode(input);
        let html = render_to_html(&nodes);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_quote() {
        let input = "[quote]Hello world[/quote]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains("<blockquote"));
        assert!(html.contains("Hello world"));
    }

    #[test]
    fn test_quote_with_author() {
        let input = "[quote=John]Hello world[/quote]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains("bbcode-quote-author"));
        assert!(html.contains("John wrote:"));
    }

    #[test]
    fn test_url() {
        let input = "[url=https://example.com]Click here[/url]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains(r#"href="https://example.com""#));
        assert!(html.contains("Click here"));
    }

    #[test]
    fn test_code_escapes_html() {
        let input = "[code]<script>alert('xss')[/code]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_line_breaks() {
        let input = "Line 1\nLine 2";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains("<br>"));
    }

    #[test]
    fn test_code_with_language() {
        let input = "[code=rust]fn main() {}[/code]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains(r#"class=\"language-rust\""#));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn test_font_family() {
        let input = "[font=Arial]Text with Arial font[/font]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains(r#"font-family: Arial"#));
        assert!(html.contains("bbcode-font"));
    }

    #[test]
    fn test_font_with_family_and_size() {
        let input = "[font family=Courier size=12px]Code text[/font]";
        let nodes = parse_bbcode(input);
        let html = render_to_html(&nodes);
        println!("Font family/size nodes: {:?}", nodes);
        println!("Font family/size HTML: {}", html);
        assert!(html.contains(r#"font-family: Courier"#));
        assert!(html.contains(r#"font-size: 12px"#));
    }

    #[test]
    fn test_font_with_size_number() {
        let input = "[font size=5]Big text[/font]";
        let nodes = parse_bbcode(input);
        let html = render_to_html(&nodes);
        println!("Font size=5 nodes: {:?}", nodes);
        println!("Font size=5 HTML: {}", html);
        assert!(html.contains(r#"font-size: 18px"#));
    }

    #[test]
    fn test_ordered_list_numeric() {
        let input = "[list=1][*]First[*]Second[/list]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains("<ol"));
        assert!(html.contains("bbcode-ordered-list"));
        assert!(html.contains("<li>First</li>"));
        assert!(html.contains("</ol>"));
    }

    #[test]
    fn test_ordered_list_alpha() {
        let input = "[list=a][*]First[*]Second[/list]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains(r#"type=\"a\""#));
        assert!(html.contains("<ol"));
    }

    #[test]
    fn test_ordered_list_upper_alpha() {
        let input = "[list=A][*]First[*]Second[/list]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(html.contains(r#"type=\"A\""#));
    }

    #[test]
    fn test_unordered_list() {
        let input = "[list][*]First[*]Second[/list]";
        let nodes = parse_bbcode(input);
        let html = render_to_html(&nodes);
        println!("Nodes: {:?}", nodes);
        println!("HTML: {}", html);
        assert!(html.contains("<ul"));
        assert!(!html.contains("<ol"));
        assert!(html.contains("<li>First</li>"));
    }

    #[test]
    fn test_javascript_url_sanitized() {
        let input = "[url=javascript:alert(1)]click[/url]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("href=\"#\""));
    }

    #[test]
    fn test_data_url_img_sanitized() {
        let input = "[img]data:text/html,<script>alert(1)</script>[/img]";
        let html = render_to_html(&parse_bbcode(input));
        assert!(!html.contains("data:text/html"));
        assert!(html.contains("src=\"#\""));
    }
}
