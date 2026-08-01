use crate::Context as PoiseContext;
use crate::models::archived_soundboards;
use sea_orm::EntityTrait;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::entity::prelude::*;

#[poise::command(slash_command)]
pub async fn archive(
    ctx: PoiseContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.say("This command must be run in a guild").await?;
            return Ok(());
        }
    };

    ctx.defer_ephemeral().await?;

    // Fetch all soundboards from the guild via the HTTP API
    let sbs = ctx.http().get_guild_soundboards(guild_id).await?;

    // Skip if already archived
    let archived_soundboard_ids = archived_soundboards::Entity::find()
        .column(archived_soundboards::Column::SoundId)
        .all(&ctx.data().db)
        .await?
        .into_iter()
        .map(|a| a.sound_id)
        .collect::<Vec<_>>();

    let mut count = 0;
    for sb in sbs {
        let sound_id = sb.id.to_string();

        if archived_soundboard_ids.contains(&sound_id) {
            continue;
        }

        archive_soundboard(
            &ctx.data().db,
            &ctx.data().s3,
            &sound_id,
            &sb.name,
            sb.user.map(|u| u.id.to_string()),
        )
        .await?;

        count += 1;
    }

    ctx.reply(format!("Successfully archived {} soundboards", count))
        .await?;
    Ok(())
}

pub async fn archive_soundboard(
    db: &sea_orm::DatabaseConnection,
    s3: &crate::s3::S3Client,
    sound_id: &str,
    name: &str,
    original_uploader: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let key = format!("soundboards/{}", sound_id);

    let soundboard_data = reqwest::get(format!(
        "https://cdn.discordapp.com/soundboard-sounds/{}",
        sound_id
    ))
    .await?
    .bytes()
    .await?;

    s3.upload_bytes(&key, soundboard_data).await?;

    let am = archived_soundboards::ActiveModel {
        sound_id: Set(sound_id.to_string()),
        name: Set(name.to_string()),
        s3_key: Set(key),
        original_uploader: Set(original_uploader),
    };

    am.insert(db).await?;
    Ok(())
}
