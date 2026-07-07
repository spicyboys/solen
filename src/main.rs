mod channels;
mod commands;
mod emojis;
mod jobs;
mod models;
mod responders;
mod roles;
mod soundboard_manager;

use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude as serenity;
use sea_orm::Database;
use serenity::{
    all::{CreateMessage, GuildChannel, Message, MessageBuilder},
    async_trait,
    model::{id::GuildId, voice::VoiceState},
    prelude::*,
};
use std::env;
use tokio_cron_scheduler::JobScheduler;

use crate::soundboard_manager::voice_state_update;

pub static SPICY_BOYS: GuildId = GuildId::new(209487220837449729);

pub struct Data {
    pub db: sea_orm::DatabaseConnection,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, rename = "subscribe")]
async fn subscribe_command(
    ctx: Context<'_>,
    #[description = "RSS feed or ntfy.sh topic URL to subscribe to"] feed_url: String,
) -> Result<(), Error> {
    commands::subscribe::subscribe(ctx, feed_url).await
}

#[poise::command(slash_command, rename = "unsubscribe")]
async fn unsubscribe_command(ctx: Context<'_>) -> Result<(), Error> {
    commands::unsubscribe::unsubscribe(ctx).await
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn thread_create(&self, ctx: serenity::prelude::Context, thread: GuildChannel) {
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

    async fn message(&self, ctx: serenity::prelude::Context, message: Message) {
        for responder in responders::RESPONDERS.iter() {
            if let Err(e) = responder.respond(&ctx, &message).await {
                eprintln!("Responder error: {:?}", e);
            }
        }
    }

    async fn voice_state_update(
        &self,
        ctx: serenity::prelude::Context,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        voice_state_update(ctx, old, new).await;
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

    let conn_for_framework = conn.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![subscribe_command(), unsubscribe_command()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let db = conn_for_framework.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db })
            })
        })
        .build();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
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
