use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{Context as PoiseContext, models::patch_notes};

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn unsubscribe(
    ctx: PoiseContext<'_>,
    #[autocomplete = "unsubscribe_feed_autocomplete"] feed: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel_id = ctx.channel_id().get() as i64;
    let Some(subscription) = patch_notes::Entity::find()
        .filter(patch_notes::Column::ChannelId.eq(channel_id))
        .filter(patch_notes::Column::Feed.eq(&feed))
        .one(&ctx.data().db)
        .await?
    else {
        ctx.say(format!("Unknown feed {}", feed)).await?;
        return Ok(());
    };

    subscription
        .into_active_model()
        .delete(&ctx.data().db)
        .await?;

    ctx.say(format!("Unsubscribed this channel from {}", feed))
        .await?;

    Ok(())
}

async fn unsubscribe_feed_autocomplete(
    ctx: PoiseContext<'_>,
    _: &str,
) -> impl Iterator<Item = String> {
    let channel_id = ctx.channel_id().get() as i64;
    patch_notes::Entity::find()
        .filter(patch_notes::Column::ChannelId.eq(channel_id))
        .all(&ctx.data().db)
        .await
        .unwrap_or_else(|_| Vec::new())
        .into_iter()
        .map(|subscription| subscription.feed)
}
