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

const JOB_REPEAT_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub async fn schedule(scheduler: &JobScheduler, ctx: JobContext) -> Result<(), JobSchedulerError> {
    let patch_ctx = ctx.clone();
    scheduler
        .add(Job::new_repeated_async(
            JOB_REPEAT_INTERVAL,
            move |_, _| {
                let ctx = patch_ctx.clone();
                Box::pin(async move {
                    if let Err(e) = patch_notes::sync_patch_note_jobs(ctx).await {
                        eprintln!("Patch note job failed: {:?}", e);
                    }
                })
            },
        )?)
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
