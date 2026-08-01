use poise::serenity_prelude as serenity;

use crate::{Data, Error, constants::interactions::SOUNDBOARD_RESTORE_PREFIX};
use serenity::{
    ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};

pub async fn handle_soundboard_restore(
    data: &Data,
    ctx: &serenity::Context,
    comp: &ComponentInteraction,
) -> Result<(), Error> {
    let Some(sound_id) = comp
        .data
        .custom_id
        .as_str()
        .strip_prefix(SOUNDBOARD_RESTORE_PREFIX)
    else {
        return Ok(());
    };
    let Some(guild_id) = comp.guild_id else {
        return Ok(());
    };

    comp.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Restoring soundboard..."),
        ),
    )
    .await?;

    let result =
        crate::commands::perform_restore(&data.db, &data.s3, &ctx.http, guild_id, sound_id).await;

    let content = match result {
        Ok(message) => message,
        Err(e) => format!("Failed to restore soundboard: {:?}", e),
    };

    comp.edit_response(&ctx.http, EditInteractionResponse::new().content(content))
        .await?;

    Ok(())
}
