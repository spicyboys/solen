mod grok;
mod slurp_enforcement;

use async_trait::async_trait;
use serenity::all::{Context, Message};
use anyhow::Result;

#[async_trait]
pub trait Responder: Send + Sync {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()>;
}

pub const RESPONDERS: [&dyn Responder; 2] = [
    &grok::GrokResponder,
    &slurp_enforcement::SlurpEnforcmentResponder,
];