mod subscribe;
mod unsubscribe;

use anyhow::Result;
use subscribe::subscribe;
use unsubscribe::unsubscribe;

use crate::{Context as PoiseContext, models::feeds};

#[poise::command(slash_command, subcommands("subscribe", "unsubscribe"))]
pub async fn feed(_: PoiseContext<'_>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

async fn channel_feed_list_autocomplete<'a>(
    ctx: PoiseContext<'a>,
    _: &'a str,
) -> poise::serenity_prelude::CreateAutocompleteResponse<'a> {
    let channel_id = ctx.channel_id().to_string();
    let mut db = ctx.data().db.clone();
    let feeds = feeds::Model::all()
        .filter(feeds::Model::fields().channel_id().eq(channel_id))
        .exec(&mut db)
        .await
        .unwrap_or_else(|_| Vec::new());
    poise::serenity_prelude::CreateAutocompleteResponse::new().set_choices(
        feeds
            .into_iter()
            .map(|subscription| {
                poise::serenity_prelude::AutocompleteChoice::from(subscription.feed)
            })
            .collect::<Vec<_>>(),
    )
}
