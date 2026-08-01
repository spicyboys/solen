use poise::serenity_prelude as serenity;
use serenity::{
    builder::CreateMessage,
    model::{channel::GuildThread, id::GenericChannelId},
    utils::MessageBuilder,
};

use crate::constants;

pub async fn handle_thread_create(ctx: &serenity::Context, thread: &GuildThread) {
    if GenericChannelId::from(thread.parent_id) != constants::channels::SPICY_GAMES {
        return;
    }

    let message_content = MessageBuilder::new()
        .mention(&thread.owner_id)
        .push(" has created a new thread in ")
        .channel(constants::channels::SPICY_GAMES)
        .push(": ")
        .channel(GenericChannelId::from(thread.id))
        .build();
    let message = CreateMessage::new().content(message_content);
    let _ = constants::channels::GAMES_CHAT
        .send_message(&ctx.http, message)
        .await;
}
