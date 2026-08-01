mod soundboard_pager;
mod soundboard_restore;

use poise::serenity_prelude::{self as serenity, ComponentInteraction};

use crate::{Data, constants};
use soundboard_pager::handle_soundboard_pager;
use soundboard_restore::handle_soundboard_restore;

pub async fn handle_interaction_create(
    ctx: &serenity::Context,
    data: &Data,
    interaction: &ComponentInteraction,
) {
    let interaction_id = interaction.data.custom_id.as_str();
    let result = if interaction_id.starts_with(constants::interactions::SOUNDBOARD_RESTORE_PREFIX) {
        handle_soundboard_restore(data, ctx, interaction).await
    } else if interaction_id.starts_with(constants::interactions::SOUNDBOARD_PAGER_PREFIX) {
        handle_soundboard_pager(data, ctx, interaction).await
    } else {
        Ok(())
    };
    if let Err(e) = result {
        eprintln!("Soundboard interaction error: {:?}", e);
    }
}
