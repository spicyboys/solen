use poise::serenity_prelude as serenity;

use crate::{Data, Error, commands, constants::interactions::SOUNDBOARD_PAGER_PREFIX};
use serenity::{ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn handle_soundboard_pager(
    data: &Data,
    ctx: &serenity::Context,
    comp: &ComponentInteraction,
) -> Result<(), Error> {
    let Some(rest) = comp
        .data
        .custom_id
        .as_str()
        .strip_prefix(SOUNDBOARD_PAGER_PREFIX)
    else {
        return Ok(());
    };
    let Some((action, page_str)) = rest.split_once(':') else {
        return Ok(());
    };
    let page: usize = match page_str.parse() {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };
    let target_page = match action {
        "prev" if page > 0 => page - 1,
        "next" => page + 1,
        _ => return Ok(()),
    };
    let Some(guild_id) = comp.guild_id else {
        return Ok(());
    };

    let components =
        match commands::build_list_components(&data.db, &ctx.http, guild_id, target_page).await {
            Ok(Some(c)) => c,
            Ok(None) => return Ok(()),
            Err(e) => {
                eprintln!("Failed to load list page: {:?}", e);
                let message =
                    CreateInteractionResponseMessage::new().content("Failed to load page");
                comp.create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(message))
                    .await?;
                return Ok(());
            }
        };

    comp.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .components(components)
                .flags(serenity::all::MessageFlags::IS_COMPONENTS_V2),
        ),
    )
    .await?;

    Ok(())
}
