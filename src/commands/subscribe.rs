use std::fmt::Display;

use anyhow::{Context, Result};
use sea_orm::{EntityTrait, Set};
use url::Url;

use crate::{
    Context as PoiseContext,
    jobs::patch_notes::{ntfy, rss},
    models::patch_notes,
};

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn subscribe(
    ctx: PoiseContext<'_>,
    #[description = "RSS feed or ntfy.sh topic URL to subscribe to"] feed_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (normalized_url, feed_type) = match validate_and_normalize_feed_url(&feed_url).await {
        Ok(f) => f,
        Err(e) => {
            ctx.say(format!("Error subscribing to {feed_url} - {e}"))
                .await?;
            return Err(e.into());
        }
    };

    let channel_id = ctx.channel_id().get().to_string();
    let subscription = patch_notes::ActiveModel {
        channel_id: Set(channel_id.parse::<i64>().unwrap_or_default()),
        feed: Set(normalized_url.clone()),
        latest_post: Set(String::new()),
        feed_type: Set(feed_type.to_string()),
        ..Default::default()
    };

    patch_notes::Entity::insert(subscription)
        .exec(&ctx.data().db)
        .await
        .context("failed to save subscription")?;

    ctx.say(format!(
        "Subscribed this channel to {feed_type}: {normalized_url}"
    ))
    .await?;

    Ok(())
}

enum FeedType {
    Ntfy,
    Rss,
}

impl Display for FeedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedType::Ntfy => write!(f, "ntfy"),
            FeedType::Rss => write!(f, "rss"),
        }
    }
}

async fn validate_and_normalize_feed_url(feed_url: &str) -> Result<(String, FeedType)> {
    let parsed = Url::parse(feed_url).context("feed URL must be a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("feed URL must use http or https");
    }

    if ntfy::is_ntfy_url(&parsed) {
        ntfy::fetch_messages(parsed.as_str(), "none")
            .await
            .context("Failed to query ntfy topic")?;
        Ok((parsed.to_string(), FeedType::Ntfy))
    } else {
        let response = rss::fetch_feed_bytes(parsed.as_str())
            .await
            .context("Failed to query URL")?;
        rss::parse_rss_feed_bytes(&response).context("Malformed RSS feed")?;
        Ok((parsed.to_string(), FeedType::Rss))
    }
}
