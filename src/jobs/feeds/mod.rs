pub mod ntfy;
pub mod rss;

use anyhow::{Result, bail};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serenity::{
    all::{ChannelId, prelude::Mentionable},
    builder::CreateEmbed,
    model::id::UserId,
};

use crate::{
    jobs::JobContext,
    models::feeds::{self, FeedType},
};

pub async fn sync_feed_jobs(ctx: JobContext) -> Result<()> {
    let jobs = feeds::Entity::find().all(&ctx.db).await?;

    for job in jobs {
        if let Err(err) = match job.feed_type {
            FeedType::Ntfy => sync_ntfy_job(&ctx, &job).await,
            FeedType::Rss => sync_rss_job(&ctx, &job).await,
        } {
            eprintln!("Feed job failed for {}: {:?}", job.feed, err);
        };
    }

    Ok(())
}

async fn sync_ntfy_job(ctx: &JobContext, job: &feeds::Model) -> Result<()> {
    let since = if job.latest_post.is_empty() {
        "all".to_string()
    } else {
        job.latest_post.clone()
    };

    let mut messages = ntfy::fetch_messages(&job.feed, &since).await?;
    if messages.is_empty() {
        return Ok(());
    }
    messages.sort_by_key(|message| message.time);

    let to_post = if job.latest_post.is_empty() {
        &messages[messages.len() - 1..]
    } else {
        &messages[..]
    };

    let tag_embed = create_tag_embed(job);
    let topic = job.feed.rsplit('/').next().unwrap_or_default().to_string();
    let channel_id = ChannelId::new(job.channel_id as u64);
    for message in to_post {
        let mut message = ntfy::build_message(message, &topic);
        if let Some(ref tag_embed) = tag_embed {
            message = message.add_embed(tag_embed.clone());
        }
        channel_id.send_message(&ctx.discord_http, message).await?;
    }

    if let Some(latest) = messages.last() {
        let latest_id = latest.id.clone();
        let mut model: feeds::ActiveModel = job.clone().into();
        model.latest_post = Set(latest_id);
        model.update(&ctx.db).await?;
    }

    Ok(())
}

async fn sync_rss_job(ctx: &JobContext, job: &feeds::Model) -> Result<()> {
    let content = rss::fetch_feed_bytes(&job.feed).await?;
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

    let tag_embed = create_tag_embed(job);
    let channel_id = ChannelId::new(job.channel_id as u64);
    for item in posts.iter().rev() {
        let mut message = rss::build_message(item)?;
        if let Some(ref tag_embed) = tag_embed {
            message = message.add_embed(tag_embed.clone());
        }
        channel_id.send_message(&ctx.discord_http, message).await?;
    }

    if let Some(latest_post_id) = items.first().map(|item| item_identifier(item)) {
        let mut model: feeds::ActiveModel = job.clone().into();
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

fn create_tag_embed(job: &feeds::Model) -> Option<CreateEmbed> {
    if job.notify.is_empty() {
        None
    } else {
        let mut tag_string = String::new();
        for notify in &job.notify {
            let Ok(user_id) = notify.parse::<u64>() else {
                continue;
            };
            tag_string.push_str(&UserId::new(user_id).mention().to_string());
            tag_string.push('\n');
        }
        Some(CreateEmbed::new().description(tag_string))
    }
}
