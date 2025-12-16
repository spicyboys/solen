use std::env;

use rand::random_bool;
use serenity::{
    all::{ChannelId, CreateMessage, GuildChannel, Message, MessageBuilder},
    async_trait,
    prelude::*,
};

struct Handler;

// Spicy Boys
static SPICY_GAMES: ChannelId = ChannelId::new(1406825680741597286);
static GAMES_CHAT: ChannelId = ChannelId::new(935677093642133564);

#[async_trait]
impl EventHandler for Handler {
    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if thread.parent_id != Some(SPICY_GAMES) {
            return;
        }

        let Some(owner_id) = thread.owner_id else {
            return;
        };

        let message_content = MessageBuilder::new()
            .mention(&owner_id)
            .push(" has created a new thread in ")
            .channel(SPICY_GAMES)
            .push(": ")
            .channel(thread.id)
            .build();
        let message = CreateMessage::new().content(message_content);
        let _ = GAMES_CHAT.send_message(ctx.http, message).await;
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.content == "@grok is this true" {
            if random_bool(0.5) {
                let _ = message
                    .channel_id
                    .send_message(ctx.http, CreateMessage::new().content("yes").reference_message(&message))
                    .await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
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

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
