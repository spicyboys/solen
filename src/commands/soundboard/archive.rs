use poise::serenity_prelude::Soundboard;

use crate::Context as PoiseContext;
use crate::models::archived_soundboards;

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
    let mut db = ctx.data().db.clone();
    let models = archived_soundboards::Model::all().exec(&mut db).await?;

    let mut count = 0;
    for sb in sbs {
        let sb = ctx.http().get_guild_soundboard(guild_id, sb.id).await?;

        if let Some(model) = models.iter().find(|m| m.sound_id == sb.id.to_string()) {
            // We explicitly do not update the `original_uploader` here since it
            // might have been restored via the bot.
            model
                .clone()
                .update()
                .name(sb.name)
                .emoji_id(sb.emoji_id.map(|e| e.to_string()))
                .emoji_name(sb.emoji_name)
                .exec(&mut db)
                .await?;
        } else {
            archive_soundboard(&mut db, &ctx.data().s3, &sb).await?;

            count += 1;
        }
    }

    ctx.reply(format!("Successfully archived {} soundboards", count))
        .await?;
    Ok(())
}

pub async fn archive_soundboard(
    db: &mut toasty::Db,
    s3: &crate::s3::S3Client,
    soundboard: &Soundboard,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let key = format!("soundboards/{}", soundboard.id);

    let soundboard_data = reqwest::get(format!(
        "https://cdn.discordapp.com/soundboard-sounds/{}",
        soundboard.id
    ))
    .await?
    .bytes()
    .await?;

    s3.upload_bytes(&key, soundboard_data).await?;

    match toasty::create!(archived_soundboards::Model {
        s3_key: &key,
        sound_id: soundboard.id.to_string(),
        name: soundboard.name.to_string(),
        original_uploader: soundboard.user.as_ref().map(|u| u.id.to_string()),
        emoji_id: soundboard.emoji_id.map(|e| e.to_string()),
        emoji_name: soundboard.emoji_name.clone(),
    })
    .exec(db)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            s3.delete(&key).await?;
            Err(Box::new(e))
        }
    }
}
