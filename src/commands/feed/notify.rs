use super::channel_feed_list_autocomplete;
use crate::{Context as PoiseContext, models::feeds};
use anyhow::Result;
use poise::CreateReply;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};

#[poise::command(slash_command)]
pub async fn notify(
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

    let user_id = ctx.author().id.to_string();
    let mut notify = subscription.notify.clone();

    let reply = if let Some(index) = notify.iter().position(|value| *value == user_id) {
        notify.swap_remove(index);
        format!("Unsubscribed to notifications from {}", feed)
    } else {
        notify.push(ctx.author().id.to_string());
        format!("Subscribed to notifications from {}", feed)
    };

    let mut model = subscription.into_active_model();
    model.notify = ActiveValue::Set(notify);
    model.update(&ctx.data().db).await?;

    ctx.send(CreateReply::default().content(reply).ephemeral(true))
        .await?;

    Ok(())
}
