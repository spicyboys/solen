mod grok;
mod ollama;
mod slurp_enforcement;

use anyhow::Result;
use async_trait::async_trait;
use serenity::all::{Context, Message};

#[async_trait]
pub trait Responder: Send + Sync {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()>;
}

pub const RESPONDERS: [&dyn Responder; 2] = [
    &grok::GrokResponder,
    &slurp_enforcement::SlurpEnforcmentResponder,
    // &ollama::OllamaResponder::new("jaahas/qwen3.5-uncensored:4b"),
];
