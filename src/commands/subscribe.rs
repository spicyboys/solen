use anyhow::{Context, Result};
use sea_orm::{EntityTrait, Set};
use url::Url;

use crate::{Context as PoiseContext, jobs::patch_notes::rss, models::patch_notes};

pub async fn subscribe(
    ctx: PoiseContext<'_>,
    feed_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let feed_url = validate_and_normalize_feed_url(&feed_url)?;

    let response = reqwest::get(&feed_url).await?.bytes().await?;
    let _channel = rss::parse_rss_feed_bytes(&response)?;

    let channel_id = ctx.channel_id().get().to_string();
    let subscription = patch_notes::ActiveModel {
        channel_id: Set(channel_id.parse::<i64>().unwrap_or_default()),
        feed: Set(feed_url.clone()),
        latest_post: Set(String::new()),
        ..Default::default()
    };

    patch_notes::Entity::insert(subscription)
        .exec(&ctx.data().db)
        .await
        .context("failed to save subscription")?;

    ctx.say(format!("Subscribed this channel to RSS feed: {feed_url}"))
        .await?;

    Ok(())
}

fn validate_and_normalize_feed_url(feed_url: &str) -> Result<String> {
    let parsed = Url::parse(feed_url).context("feed URL must be a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("feed URL must use http or https");
    }

    Ok(parsed.to_string())
}
