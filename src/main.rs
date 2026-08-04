mod commands;
mod components;
mod config;
mod constants;
mod events;
mod jobs;
mod models;
mod s3;
mod settings;
mod web;

use std::sync::Arc;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use serenity::{all::ClientBuilder, prelude::*};
use toasty::{Db, db::Connect};
use tokio_cron_scheduler::JobScheduler;

use crate::s3::S3Client;

pub struct DiscordClientContext {
    pub db: Db,
    pub s3: s3::S3Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, DiscordClientContext, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let config = config::AppConfig::load().expect("Failed to load config");

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install AWS-LC crypto provider");

    let db = Db::builder()
        .models(toasty::models!(
            models::birthdays::Model,
            models::feeds::Model,
            models::archived_soundboards::Model,
            models::web_sessions::Model,
            models::settings::Model,
        ))
        .build(Connect::new(&config.database_url).await.unwrap())
        .await
        .unwrap();

    let token = Token::try_from(config.discord_token.clone()).expect("Invalid bot token");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS;

    let framework = poise::Framework::new(poise::FrameworkOptions {
        commands: vec![
            commands::feed(),
            commands::birthday(),
            commands::soundboard(),
        ],
        ..Default::default()
    });

    let s3 = S3Client::new(config.s3).await;

    let data = Arc::new(DiscordClientContext {
        db: db.clone(),
        s3: s3.clone(),
    });

    let mut client = ClientBuilder::new(token, intents)
        .data(data.clone())
        .event_handler(Arc::new(events::Handler::new(data.clone())))
        .framework(Box::new(framework))
        .await
        .expect("Err creating client");

    let mut sched = JobScheduler::new().await.unwrap();
    jobs::schedule(
        &sched,
        jobs::JobContext {
            discord_http: client.http.clone(),
            db: db.clone(),
        },
    )
    .await
    .unwrap();

    println!("Starting scheduler...");

    sched.start().await.expect("Failed to start scheduler");

    let listener = tokio::net::TcpListener::bind((config.web.host.as_str(), config.web.port))
        .await
        .expect("Failed to bind web listener");
    println!(
        "Starting web server on {}:{}",
        config.web.host, config.web.port
    );
    tracing::info!(
        host = %config.web.host,
        port = config.web.port,
        secure_cookies = config.web.secure_cookies,
        oauth_redirect_uri = %config.discord_oauth.redirect_uri,
        "web server starting"
    );
    let web_ctx = web::WebContext {
        db,
        s3,
        discord_client: client.http.clone(),
        client: reqwest::Client::new(),
        web_config: config.web,
        discord_oauth_config: config.discord_oauth,
    };
    let router = web::router(web_ctx);
    tokio::spawn(async move {
        if let Err(error) = topcoat::serve(listener, router).await {
            eprintln!("Web server error: {error}");
        }
    });

    println!("Starting discord client...");

    let shutdown_discord = client.shard_manager.get_shutdown_trigger();

    tokio::select! {
        result = client.start() => {
            if let Err(why) = result {
                println!("Client error: {why:?}");
            }
        }
        _ = shutdown_signal() => {
            println!("Shutdown signal received, stopping...");
            shutdown_discord();
            if let Err(error) = sched.shutdown().await {
                eprintln!("Scheduler shutdown error: {error:?}");
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install the SIGTERM signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
