use super::channel_feed_list_autocomplete;
use crate::{Context as PoiseContext, models::feeds};
use anyhow::Result;

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn unsubscribe(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_feed_list_autocomplete"] feed: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel_id = ctx.channel_id().get() as i64;
    let mut db = ctx.data().db.clone();
    let Some(subscription) = feeds::Model::filter(feeds::Model::fields().channel_id().eq(channel_id))
        .filter(feeds::Model::fields().feed().eq(feed.clone()))
        .first()
        .exec(&mut db)
        .await?
    else {
        ctx.say(format!("Unknown feed {}", feed)).await?;
        return Ok(());
    };

    subscription.delete().exec(&mut db).await?;

    ctx.say(format!("Unsubscribed this channel from {}", feed))
        .await?;

    Ok(())
}
