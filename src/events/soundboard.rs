use poise::serenity_prelude::Soundboard;
use poise::serenity_prelude::all::{SoundboardSoundCreateEvent, SoundboardSoundUpdateEvent};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use sea_orm::entity::prelude::*;

use crate::Data;
use crate::commands::archive_soundboard;
use crate::models::archived_soundboards;

pub async fn handle_soundboard_sound_create(data: &Data, event: &SoundboardSoundCreateEvent) {
    handle_unarchived_soundboard(data, &event.soundboard).await;
}

pub async fn handle_soundboard_sound_update(data: &Data, event: &SoundboardSoundUpdateEvent) {
    let sound_id = event.soundboard.id.to_string();

    let entity = archived_soundboards::Entity::find_by_id(sound_id)
        .one(&data.db)
        .await
        .ok()
        .flatten();

    if let Some(entity) = entity {
        if entity.name != event.soundboard.name {
            let mut am = entity.into_active_model();
            am.name = Set(event.soundboard.name.clone());
            if let Err(e) = am.update(&data.db).await {
                eprintln!(
                    "Failed to update archived soundboard {}: {:?}",
                    event.soundboard.id, e
                );
            }
        }
    } else {
        handle_unarchived_soundboard(data, &event.soundboard).await;
    }
}

async fn handle_unarchived_soundboard(data: &Data, soundboard: &Soundboard) {
    if let Err(e) = archive_soundboard(
        &data.db,
        &data.s3,
        &soundboard.id.to_string(),
        &soundboard.name,
        soundboard.user.as_ref().map(|u| u.id.to_string()),
    )
    .await
    {
        eprintln!("Failed to archive soundboard {}: {:?}", soundboard.id, e);
    }
}
