use std::env;

use serenity::{
    all::{
        ChannelId, ChannelType, Command, CommandInteraction, CreateCommand, CreateMessage, GuildChannel, GuildId, Interaction, MessageBuilder, Ready, User
    },
    async_trait,
    prelude::*,
};
use songbird::SerenityInit;

struct Handler;

// Spicy Boys
static SPICY_GAMES: ChannelId = ChannelId::new(1406825680741597286);
static GAMES_CHAT: ChannelId = ChannelId::new(935677093642133564);

async fn get_user_voice_channel(ctx: Context, guild_id: GuildId, user: User) -> Option<GuildChannel> {
    let Ok(channels) = guild_id.channels(&ctx.http).await else {
        return None;
    };

    let Some((_, channel)) = channels.iter().find(|(_, channel)| {
        if channel.kind != ChannelType::Voice {
            return false;
        }

        let Ok(members) = channel.members(&ctx.cache) else {
            return false;
        };

        members
            .iter()
            .find(|member| member.user.id == user.id)
            .is_some()
    }) else {
        return None;
    };

    return Some(channel.to_owned());
}

async fn sound(ctx: Context, command: CommandInteraction) {
    let Some(guild_id) = command.guild_id else {
        return;
    };

    let Some(channel) = get_user_voice_channel(ctx, guild_id, command.user).await else {
        return;
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, channel.id).await {
        let handler = handler_lock.lock().await;
        // handler.play(track);
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let sound_command = CreateCommand::new("sound").description("Play a soundboard");
        let _ = Command::create_global_command(&ctx.http, sound_command).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        match command.data.name.as_str() {
            "sound" => sound(ctx, command).await,
            _ => (),
        };
    }

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
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
