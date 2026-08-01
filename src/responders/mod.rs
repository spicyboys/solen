mod grok;
mod slurp_enforcement;

use anyhow::Result;
use async_trait::async_trait;
use poise::serenity_prelude as serenity;

use serenity::all::{Context, Message};

#[async_trait]
pub trait Responder: Send + Sync {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()>;
}

pub const RESPONDERS: [&dyn Responder; 2] = [
    &grok::GrokResponder,
    &slurp_enforcement::SlurpEnforcmentResponder,
];
