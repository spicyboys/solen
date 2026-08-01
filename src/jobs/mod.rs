mod birthday;
pub mod feeds;

use chrono_tz::US::Central;
use poise::serenity_prelude as serenity;
use std::{sync::Arc, time::Duration};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

#[derive(Clone)]
pub struct JobContext {
    pub discord_http: Arc<serenity::http::Http>,
    pub db: sea_orm::DatabaseConnection,
}

const FEED_POLL_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub async fn schedule(scheduler: &JobScheduler, ctx: JobContext) -> Result<(), JobSchedulerError> {
    let rss_ctx = ctx.clone();
    scheduler
        .add(Job::new_repeated_async(FEED_POLL_INTERVAL, move |_, _| {
            let ctx = rss_ctx.clone();
            Box::pin(async move {
                if let Err(e) = feeds::sync_feed_jobs(ctx).await {
                    eprintln!("RSS patch note job failed: {:?}", e);
                }
            })
        })?)
        .await?;

    scheduler
        .add(Job::new_async_tz("0 0 10 * * *", Central, move |_, _| {
            Box::pin({
                let ctx = ctx.clone();
                async move {
                    if let Err(e) = birthday::send_birthday_message(ctx).await {
                        eprintln!("Failed to send birthday messages: {:?}", e);
                    }
                }
            })
        })?)
        .await?;

    Ok(())
}
