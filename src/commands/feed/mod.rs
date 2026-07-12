mod notify;
mod subscribe;
mod unsubscribe;

use anyhow::Result;
use notify::notify;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use subscribe::subscribe;
use unsubscribe::unsubscribe;

use crate::{Context as PoiseContext, models::feeds};

#[poise::command(slash_command, subcommands("subscribe", "unsubscribe", "notify"))]
pub async fn feed(_: PoiseContext<'_>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

async fn channel_feed_list_autocomplete(
    ctx: PoiseContext<'_>,
    _: &str,
) -> impl Iterator<Item = String> {
    let channel_id = ctx.channel_id().get() as i64;
    feeds::Entity::find()
        .filter(feeds::Column::ChannelId.eq(channel_id))
        .all(&ctx.data().db)
        .await
        .unwrap_or_else(|_| Vec::new())
        .into_iter()
        .map(|subscription| subscription.feed)
}
