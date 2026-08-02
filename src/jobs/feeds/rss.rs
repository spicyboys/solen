use anyhow::{Context, Result};
use bytes::Bytes;
use html_to_markdown_rs::{ConversionOptions, convert};
use poise::serenity_prelude as serenity;

use serenity::all::{CreateEmbed, CreateMessage};

pub(super) const EMBED_TITLE_LIMIT: usize = 256;
const EMBED_DESCRIPTION_WARNING_THRESHOLD: usize = 2048;
const EMBED_DESCRIPTION_TRUNCATION_LIMIT: usize = 1024;
const EMBED_DESCRIPTION_TRUNCATION_NOTICE: &str = "\n\n[truncated]";

// Make silly lil feedhosts like wikipedia respect our authority
// (and not think we're a bot even tho we are lamo)
pub const FEED_USER_AGENT: &str = "solen-discord-bot (+https://github.com/spicyboys/solen)";

pub async fn fetch_feed_bytes(url: &str) -> reqwest::Result<Bytes> {
    reqwest::Client::builder()
        .user_agent(FEED_USER_AGENT)
        .build()?
        .get(url)
        .send()
        .await?
        .bytes()
        .await
}

pub fn parse_rss_feed_bytes(bytes: &[u8]) -> Result<rss::Channel> {
    rss::Channel::read_from(bytes).context("invalid RSS feed")
}

pub fn build_message<'a>(item: &'a rss::Item) -> Result<CreateMessage<'a>> {
    let mut embed = CreateEmbed::new();

    if let Some(title) = item.title() {
        embed = embed.title(truncate_text(title, EMBED_TITLE_LIMIT));
    }

    if let Some(link) = item.link() {
        embed = embed.url(link);
    }

    let description = item.content().or_else(|| item.description());
    if let Some(description) = description {
        embed = embed.description(truncate_description(&parse_html(description)?));
    }

    Ok(CreateMessage::new().embed(embed))
}

pub(super) fn truncate_text(text: &str, limit: usize) -> String {
    let mut chars = text.chars();
    let mut truncated = String::new();

    for ch in chars.by_ref() {
        if truncated.chars().count() >= limit {
            break;
        }
        truncated.push(ch);
    }

    truncated
}

pub(super) fn truncate_description(description: &str) -> String {
    if description.chars().count() <= EMBED_DESCRIPTION_WARNING_THRESHOLD {
        return description.to_string();
    }

    let truncated = truncate_text(description, EMBED_DESCRIPTION_TRUNCATION_LIMIT);
    format!("{truncated}{EMBED_DESCRIPTION_TRUNCATION_NOTICE}")
}

pub fn parse_html(html: &str) -> Result<String> {
    let options = ConversionOptions::builder().skip_images(true).build();
    Ok(convert(html, Some(options))?
        .content
        .unwrap_or_default()
        .trim_end()
        .to_string())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_html() {
        let html = r#"<p>This is a <strong>test</strong> description with an image: <img src="image.png" alt="An image"></p>"#;
        let md = parse_html(html).unwrap();
        assert_eq!(md, "This is a **test** description with an image:");
    }

    #[test]
    fn test_parse_rss_feed_bytes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0"><channel><title>Example</title><link>https://example.com</link><description>Test</description><item><title>Item</title><link>https://example.com/item</link><description>Example</description></item></channel></rss>"#;

        let parsed = parse_rss_feed_bytes(xml.as_bytes()).unwrap();
        assert_eq!(parsed.title(), "Example");
    }

    #[test]
    fn test_truncate_text_enforces_title_limit() {
        let long_title = "a".repeat(300);

        assert_eq!(truncate_text(&long_title, 256).chars().count(), 256);
    }

    #[test]
    fn test_truncate_description_leaves_short_content_full() {
        let description = "short description".to_string();

        assert_eq!(truncate_description(&description), description);
    }

    #[test]
    fn test_truncate_description_truncates_long_content() {
        let long_description = "a".repeat(3000);
        let truncated = truncate_description(&long_description);

        assert!(truncated.chars().count() < long_description.chars().count());
        assert!(truncated.ends_with(EMBED_DESCRIPTION_TRUNCATION_NOTICE));
    }
}
