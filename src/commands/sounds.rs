use poise::CreateReply;
use serenity::all::AutocompleteChoice;
use songbird::input::cached::Memory;
use songbird::input::File;
use std::fs;

use crate::commands::{Context, Error};
use crate::utils::UserUtils;

async fn autocomplete_name<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice> {
    fs::read_dir(".")
        .unwrap()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();

            if path.is_file() {
                match path.extension().and_then(|p| p.to_str()) {
                    Some("mp3") => true,
                    _ => false,
                }
            } else {
                false
            }
        })
        .filter_map(|entry| entry.file_name().to_str().map(String::from))
        .filter(move |entry| entry.starts_with(partial))
        .map(AutocompleteChoice::from)
}

#[poise::command(slash_command)]
pub async fn sound(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_name"] name: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("Must be run from a guild".into());
    };

    let Some(channel) = ctx
        .author()
        .get_current_voice_channel(ctx.serenity_context(), guild_id)
        .await
    else {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("You must be connected to a voice channel"),
        )
        .await?;
        return Ok(());
    };

    let file = Memory::new(File::new(name).into()).await?;

    let manager = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, channel.id).await {
        let mut handler = handler_lock.lock().await;
        let sound = handler.play_input(file.into());
        let _ = sound.set_volume(0.4);
        sound.play()?;
    };
    Ok(())
}
