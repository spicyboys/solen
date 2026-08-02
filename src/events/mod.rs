mod message;
mod soundboard;
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
            FullEvent::SoundboardSoundCreate { event, .. } => {
                soundboard::handle_soundboard_sound_create(&self.data, event).await;
            }
            FullEvent::SoundboardSoundUpdate { event, .. } => {
                soundboard::handle_soundboard_sound_update(&self.data, event).await;
            }
            _ => {}
        }
    }
}
