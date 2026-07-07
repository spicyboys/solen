mod birthday;
pub mod patch_notes;

use chrono_tz::US::Central;
use std::{sync::Arc, time::Duration};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

#[derive(Clone)]
pub struct JobContext {
    pub discord_http: Arc<serenity::http::Http>,
    pub db: sea_orm::DatabaseConnection,
}

const RSS_POLL_INTERVAL: Duration = Duration::from_secs(60 * 10);
const NTFY_POLL_INTERVAL: Duration = Duration::from_secs(60);

pub async fn schedule(scheduler: &JobScheduler, ctx: JobContext) -> Result<(), JobSchedulerError> {
    let rss_ctx = ctx.clone();
    scheduler
        .add(Job::new_repeated_async(RSS_POLL_INTERVAL, move |_, _| {
            let ctx = rss_ctx.clone();
            Box::pin(async move {
                if let Err(e) = patch_notes::sync_rss_jobs(ctx).await {
                    eprintln!("RSS patch note job failed: {:?}", e);
                }
            })
        })?)
        .await?;

    let ntfy_ctx = ctx.clone();
    scheduler
        .add(Job::new_repeated_async(NTFY_POLL_INTERVAL, move |_, _| {
            let ctx = ntfy_ctx.clone();
            Box::pin(async move {
                if let Err(e) = patch_notes::sync_ntfy_jobs(ctx).await {
                    eprintln!("ntfy patch note job failed: {:?}", e);
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
