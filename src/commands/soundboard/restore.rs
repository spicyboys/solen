use crate::Context as PoiseContext;
use crate::models::archived_soundboards;
use poise::serenity_prelude as serenity;

use serenity::all::{CreateAttachment, CreateSoundboard, GuildId};

pub async fn perform_restore(
    db: &toasty::Db,
    s3: &crate::s3::S3Client,
    http: &serenity::http::Http,
    guild_id: GuildId,
    sound_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut db = db.clone();
    let record = archived_soundboards::Model::filter_by_sound_id(sound_id.to_string())
        .first()
        .exec(&mut db)
        .await?;
    let mut record = match record {
        Some(r) => r,
        None => return Ok("Archived soundboard not found".to_string()),
    };

    let bytes = s3.download_bytes(&record.s3_key).await?;

    // Check whether the soundboard already exists on the server (by id or name)
    if let Some(existing_soundboard) = http
        .get_guild_soundboards(guild_id)
        .await?
        .iter()
        .find(|sb| sb.id.to_string() == record.sound_id)
    {
        return Ok(format!(
            "Soundboard already exists on server as {} (id={})",
            existing_soundboard.name, existing_soundboard.id
        ));
    }

    // Create a new soundboard on Discord from the archived bytes
    let mime = detect_audio_mime(&bytes);
    let attachment = CreateAttachment::bytes(bytes, "sound");
    let sound = attachment.encode(mime).await?;
    let created = http
        .create_guild_soundboard(guild_id, &CreateSoundboard::new(&record.name, sound), None)
        .await?;

    record
        .update()
        .sound_id(created.id.to_string())
        .exec(&mut db)
        .await?;

    Ok(format!("Restored archived soundboard as {}", record.name))
}

pub fn detect_audio_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"OggS") {
        "audio/ogg"
    } else {
        "audio/mpeg"
    }
}

#[poise::command(slash_command)]
pub async fn restore(
    ctx: PoiseContext<'_>,
    #[description = "ID of the archived soundboard"] id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.say("This command must be run in a guild").await?;
            return Ok(());
        }
    };

    let message =
        perform_restore(&ctx.data().db, &ctx.data().s3, ctx.http(), guild_id, &id).await?;
    ctx.say(message).await?;
    Ok(())
}
