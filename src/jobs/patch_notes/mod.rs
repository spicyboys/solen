pub mod rss;

use anyhow::{Result, bail};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serenity::all::ChannelId;

use crate::{jobs::JobContext, models::patch_notes};

pub async fn sync_patch_note_jobs(ctx: JobContext) -> Result<()> {
    let jobs = patch_notes::Entity::find().all(&ctx.db).await?;

    for job in jobs {
        if let Err(err) = sync_patch_note_job(&ctx, job.clone()).await {
            eprintln!("Patch note job failed for {}: {:?}", job.feed, err);
        }
    }

    Ok(())
}

async fn sync_patch_note_job(ctx: &JobContext, job: patch_notes::Model) -> Result<()> {
    let content = reqwest::get(&job.feed).await?.bytes().await?;
    let channel = ::rss::Channel::read_from(&content[..])?;

    let mut items: Vec<_> = channel.items().iter().collect();
    if items.is_empty() {
        bail!("No items found in RSS feed");
    }

    items.sort_by(|a, b| {
        let date_a = a
            .pub_date()
            .and_then(|d| DateTime::parse_from_rfc2822(d).ok());
        let date_b = b
            .pub_date()
            .and_then(|d| DateTime::parse_from_rfc2822(d).ok());
        date_b.cmp(&date_a)
    });

    let posts: Vec<_> = if job.latest_post.is_empty() {
        vec![items[0]]
    } else if let Some(pos) = items
        .iter()
        .position(|item| item_identifier(item) == job.latest_post)
    {
        if pos == 0 {
            return Ok(());
        }

        items[0..pos].to_vec()
    } else {
        vec![items[0]]
    };

    let channel_id = ChannelId::new(job.channel_id as u64);
    for item in posts.iter().rev() {
        channel_id
            .send_message(&ctx.discord_http, rss::build_message(item)?)
            .await?;
    }

    if let Some(latest_post_id) = items.first().map(|item| item_identifier(item)) {
        let mut model: patch_notes::ActiveModel = job.into();
        model.latest_post = Set(latest_post_id);
        model.update(&ctx.db).await?;
    }

    Ok(())
}

fn item_identifier(item: &::rss::Item) -> String {
    item.guid()
        .map(|guid| guid.value().to_string())
        .or_else(|| item.link().map(str::to_string))
        .unwrap_or_default()
}
