use anyhow::{Context, Result, anyhow};
use sea_orm::{EntityTrait, Set};
use url::Url;

use crate::{
    Context as PoiseContext, SPICY_BOYS,
    jobs::patch_notes::{ntfy, rss},
    models::patch_notes,
    roles::{BOSSY_BOYS, MID_LEVEL_MANAGEMENT_BOYS},
};

const RSS_FEED_TYPE: &str = "rss";
const NTFY_FEED_TYPE: &str = "ntfy";

pub async fn subscribe(
    ctx: PoiseContext<'_>,
    feed_url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !(ctx
        .author()
        .has_role(&ctx.http(), SPICY_BOYS, BOSSY_BOYS)
        .await?
        || ctx
            .author()
            .has_role(&ctx.http(), SPICY_BOYS, MID_LEVEL_MANAGEMENT_BOYS)
            .await?)
    {
        return Err(anyhow!("User is not an admin").into());
    }

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

    let label = if feed_type == NTFY_FEED_TYPE {
        "ntfy topic"
    } else {
        "RSS feed"
    };
    ctx.say(format!(
        "Subscribed this channel to {label}: {normalized_url}"
    ))
    .await?;

    Ok(())
}

async fn validate_and_normalize_feed_url(feed_url: &str) -> Result<(String, &'static str)> {
    let parsed = Url::parse(feed_url).context("feed URL must be a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("feed URL must use http or https");
    }

    if ntfy::is_ntfy_url(&parsed) {
        ntfy::fetch_messages(parsed.as_str(), "none")
            .await
            .context("Failed to query ntfy topic")?;
        Ok((parsed.to_string(), NTFY_FEED_TYPE))
    } else {
        let response = rss::fetch_feed_bytes(parsed.as_str())
            .await
            .context("Failed to query URL")?;
        rss::parse_rss_feed_bytes(&response).context("Malformed RSS feed")?;
        Ok((parsed.to_string(), RSS_FEED_TYPE))
    }
}
