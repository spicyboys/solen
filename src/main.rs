mod channels;
mod emojis;
mod jobs;
mod models;
mod responders;

use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use serenity::{
    all::{CreateMessage, GuildChannel, Message, MessageBuilder},
    async_trait,
    prelude::*,
};
use std::env;
use tokio_cron_scheduler::JobScheduler;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if thread.parent_id != Some(channels::SPICY_GAMES) {
            return;
        }

        let Some(owner_id) = thread.owner_id else {
            return;
        };

        let message_content = MessageBuilder::new()
            .mention(&owner_id)
            .push(" has created a new thread in ")
            .channel(channels::SPICY_GAMES)
            .push(": ")
            .channel(thread.id)
            .build();
        let message = CreateMessage::new().content(message_content);
        let _ = channels::GAMES_CHAT.send_message(ctx.http, message).await;
    }

    async fn message(&self, ctx: Context, message: Message) {
        for responder in responders::RESPONDERS.iter() {
            if let Err(e) = responder.respond(&ctx, &message).await {
                eprintln!("Responder error: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Database::connect(db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
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
