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
    let archived_soundboard_ids = archived_soundboards::Model::all()
        .exec(&mut db)
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
            &mut db,
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
    db: &mut toasty::Db,
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

    // let mut connection = db.connection().await?;
    toasty::create!(archived_soundboards::Model {
        sound_id: sound_id.to_string(),
        name: name.to_string(),
        s3_key: key,
        original_uploader,
    })
    .exec(db)
    .await?;
    Ok(())
}
