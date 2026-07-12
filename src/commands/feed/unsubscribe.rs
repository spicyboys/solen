use super::channel_feed_list_autocomplete;
use crate::{Context as PoiseContext, models::feeds};
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn unsubscribe(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_feed_list_autocomplete"] feed: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel_id = ctx.channel_id().get() as i64;
    let Some(subscription) = feeds::Entity::find()
        .filter(feeds::Column::ChannelId.eq(channel_id))
        .filter(feeds::Column::Feed.eq(&feed))
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
