mod commands;
mod constants;
mod events;
mod jobs;
mod models;
mod s3;
mod settings;

use std::sync::Arc;

use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude as serenity;
use sea_orm::Database;
use serenity::{all::ClientBuilder, prelude::*};
use tokio_cron_scheduler::JobScheduler;

use crate::s3::S3Client;

pub struct Data {
    pub db: sea_orm::DatabaseConnection,
    pub s3: s3::S3Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let settings = settings::Settings::load().expect("Failed to load settings");

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install AWS-LC crypto provider");

    let conn = Database::connect(settings.database_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let token = Token::try_from(settings.discord_token.clone()).expect("Invalid bot token");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::new(poise::FrameworkOptions {
        commands: vec![
            commands::feed(),
            commands::birthday(),
            commands::soundboard(),
        ],
        ..Default::default()
    });

    let data = Arc::new(Data {
        db: conn.clone(),
        s3: S3Client::new(settings.s3).await,
    });

    let mut client = ClientBuilder::new(token, intents)
        .data(data.clone())
        .event_handler(Arc::new(events::Handler::new(data)))
        .framework(Box::new(framework))
        .await
        .expect("Err creating client");

    let sched = JobScheduler::new().await.unwrap();
    jobs::schedule(
        &sched,
        jobs::JobContext {
            discord_http: client.http.clone(),
            db: conn,
        },
    )
    .await
    .unwrap();

    println!("Starting scheduler...");

    sched.start().await.expect("Failed to start scheduler");

    println!("Starting discord client...");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
