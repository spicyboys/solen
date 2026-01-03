mod patch_notes;

use std::{sync::Arc, time::Duration};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

#[derive(Clone)]
pub struct JobContext {
    pub discord_http: Arc<serenity::http::Http>,
    pub db: sea_orm::DatabaseConnection,
}

const JOB_REPEAT_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub async fn schedule(
    scheduler: &JobScheduler,
    ctx: JobContext,
) -> Result<(), JobSchedulerError> {
    for job in patch_notes::JOBS.iter() {
        let job_ctx = ctx.clone();
        scheduler
            .add(Job::new_one_shot_async(
                Duration::from_secs(1),
                move |_, _| {
                    Box::pin({
                        let job_ctx = job_ctx.clone();
                        async move {
                            if let Err(e) = job.fetch_latest_post(job_ctx).await {
                                eprintln!("Job failed: {:?}", e);
                            }
                        }
                    })
                },
            )?)
            .await?;    
    }

    Ok(())
}
