mod interaction_create;
mod message;
mod thread_create;
mod voice_state_update;

use std::sync::Arc;

use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, EventHandler, FullEvent};

use crate::Data;

pub struct Handler {
    data: Arc<Data>,
}

impl Handler {
    pub fn new(data: Arc<Data>) -> Self {
        Handler { data }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
        match event {
            FullEvent::Message { new_message, .. } => {
                message::handle_message(ctx, new_message).await;
            }
            FullEvent::ThreadCreate { thread, .. } => {
                thread_create::handle_thread_create(ctx, thread).await;
            }
            FullEvent::VoiceStateUpdate { old, new, .. } => {
                voice_state_update::handle_voice_state_update(ctx, old.as_ref(), new).await;
            }
            FullEvent::InteractionCreate {
                interaction: serenity::all::Interaction::Component(comp),
                ..
            } => {
                interaction_create::handle_interaction_create(ctx, &self.data, comp).await;
            }
            _ => {}
        }
    }
}
