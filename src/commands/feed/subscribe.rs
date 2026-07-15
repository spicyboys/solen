use anyhow::{Context, Result};
use sea_orm::{EntityTrait, Set};
use url::Url;

use crate::{
    Context as PoiseContext,
    jobs::feeds::{ntfy, rss},
    models::feeds,
};

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn subscribe(
    ctx: PoiseContext<'_>,
    #[description = "RSS feed or ntfy.sh topic URL to subscribe to"] feed_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let normalized_url = match validate_and_normalize_feed_url(&feed_url).await {
        Ok(url) => url,
        Err(e) => {
            ctx.say(format!("Error subscribing to {feed_url} - {e}"))
                .await?;
            return Err(e.into());
        }
    };

    let channel_id = ctx.channel_id().get().to_string();
    let subscription = feeds::ActiveModel {
        channel_id: Set(channel_id.parse::<i64>().unwrap_or_default()),
        feed: Set(normalized_url.clone()),
        latest_post: Set(String::new()),
        ..Default::default()
    };

    feeds::Entity::insert(subscription)
        .exec(&ctx.data().db)
        .await
        .context("failed to save subscription")?;

    ctx.say(format!("Subscribed this channel to: {normalized_url}"))
        .await?;

    Ok(())
}

async fn validate_and_normalize_feed_url(feed_url: &str) -> Result<String> {
    let parsed = Url::parse(feed_url).context("feed URL must be a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("feed URL must use http or https");
    }

    if ntfy::is_ntfy_url(&parsed) {
        ntfy::fetch_messages(parsed.as_str(), "none")
            .await
            .context("Failed to query ntfy topic")?;
        Ok(parsed.to_string())
    } else {
        let response = rss::fetch_feed_bytes(parsed.as_str())
            .await
            .context("Failed to query URL")?;
        rss::parse_rss_feed_bytes(&response).context("Malformed RSS feed")?;
        Ok(parsed.to_string())
    }
}
