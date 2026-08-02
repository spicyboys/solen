use poise::serenity_prelude::Soundboard;
use poise::serenity_prelude::all::{SoundboardSoundCreateEvent, SoundboardSoundUpdateEvent};
use tracing::{debug, error};

use crate::Data;
use crate::commands::archive_soundboard;
use crate::constants::BOT_ID;
use crate::models::archived_soundboards;

pub async fn handle_soundboard_sound_create(data: &Data, event: &SoundboardSoundCreateEvent) {
    if event
        .soundboard
        .user
        .as_ref()
        .is_none_or(|u| u.id != BOT_ID)
    {
        // It's possible there's already an entry for a new soundboard if we're
        // restoring one from the archive.
        debug!("Archiving new soundboard {:?}", event.soundboard);
        handle_unarchived_soundboard(data, &event.soundboard).await;
    }
}

pub async fn handle_soundboard_sound_update(data: &Data, event: &SoundboardSoundUpdateEvent) {
    let sound_id = event.soundboard.id.to_string();

    let mut db = data.db.clone();
    let entity = archived_soundboards::Model::filter_by_sound_id(sound_id)
        .first()
        .exec(&mut db)
        .await
        .ok()
        .flatten();

    if let Some(mut entity) = entity {
        if (entity.name != event.soundboard.name
            || entity.emoji_id != event.soundboard.emoji_id.map(|id| id.to_string())
            || entity.emoji_name != event.soundboard.emoji_name)
            && let Err(e) = entity
                .update()
                .name(event.soundboard.name.clone())
                .emoji_id(event.soundboard.emoji_id.map(|id| id.to_string()))
                .emoji_name(event.soundboard.emoji_name.clone())
                .exec(&mut db)
                .await
        {
            error!(
                "Failed to update archived soundboard {}: {:?}",
                event.soundboard.id, e
            );
        }
    } else {
        debug!("Archiving from update {:?}", event.soundboard);
        handle_unarchived_soundboard(data, &event.soundboard).await;
    }
}

async fn handle_unarchived_soundboard(data: &Data, soundboard: &Soundboard) {
    if let Err(e) = archive_soundboard(&mut data.db.clone(), &data.s3, soundboard).await {
        error!("Failed to archive soundboard {}: {:?}", soundboard.id, e);
    }
}
