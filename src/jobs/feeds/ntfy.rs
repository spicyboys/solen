use anyhow::{Context, Result};
use serde::Deserialize;
use serenity::all::{CreateEmbed, CreateMessage};
use url::Url;

use super::rss::{EMBED_TITLE_LIMIT, FEED_USER_AGENT, truncate_description, truncate_text};

pub const NTFY_HOST: &str = "ntfy.sh";

#[derive(Debug, Clone, Deserialize)]
pub struct NtfyMessage {
    pub id: String,
    pub time: i64,
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub click: Option<String>,
}

/// A `ntfy.sh` topic URL, i.e. `https://ntfy.sh/<topic>` with no other path segments.
pub fn is_ntfy_url(url: &Url) -> bool {
    if url.host_str() != Some(NTFY_HOST) {
        return false;
    }

    match url.path_segments() {
        Some(mut segments) => {
            let topic = segments.next().unwrap_or_default();
            !topic.is_empty() && segments.next().is_none()
        }
        None => false,
    }
}

/// Polls a `ntfy.sh` topic for messages, using ntfy's `since` param (a message id, duration,
/// unix timestamp, or "all"/"none") to page through history.
pub async fn fetch_messages(topic_url: &str, since: &str) -> Result<Vec<NtfyMessage>> {
    let json_url = format!(
        "{}/json?poll=1&since={since}",
        topic_url.trim_end_matches('/')
    );

    let bytes = reqwest::Client::builder()
        .user_agent(FEED_USER_AGENT)
        .build()?
        .get(&json_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let text = String::from_utf8_lossy(&bytes);
    let mut messages = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: NtfyMessage =
            serde_json::from_str(line).context("invalid ntfy message payload")?;
        if message.event == "message" {
            messages.push(message);
        }
    }

    Ok(messages)
}

pub fn build_message(message: &NtfyMessage, topic: &str) -> CreateMessage {
    let title = message
        .title
        .clone()
        .unwrap_or_else(|| format!("ntfy: {topic}"));

    let mut embed = CreateEmbed::new().title(truncate_text(&title, EMBED_TITLE_LIMIT));

    if let Some(click) = &message.click {
        embed = embed.url(click);
    }

    if let Some(body) = &message.message {
        embed = embed.description(truncate_description(body));
    }

    CreateMessage::new().embed(embed)
}
