use poise::serenity_prelude::Soundboard;
use poise::serenity_prelude::all::{SoundboardSoundCreateEvent, SoundboardSoundUpdateEvent};

use crate::Data;
use crate::commands::archive_soundboard;
use crate::models::archived_soundboards;

pub async fn handle_soundboard_sound_create(data: &Data, event: &SoundboardSoundCreateEvent) {
    handle_unarchived_soundboard(data, &event.soundboard).await;
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

    if let Some(mut entity) = entity
        && entity.name != event.soundboard.name
    {
        if let Err(e) = entity
            .update()
            .name(event.soundboard.name.clone())
            .exec(&mut db)
            .await
        {
            eprintln!(
                "Failed to update archived soundboard {}: {:?}",
                event.soundboard.id, e
            );
        }
    } else {
        handle_unarchived_soundboard(data, &event.soundboard).await;
    }
}

async fn handle_unarchived_soundboard(data: &Data, soundboard: &Soundboard) {
    if let Err(e) = archive_soundboard(
        &mut data.db.clone(),
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
