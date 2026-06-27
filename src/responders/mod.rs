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

pub const RESPONDERS: [&dyn Responder; 3] = [
    &grok::GrokResponder,
    &slurp_enforcement::SlurpEnforcmentResponder,
    &ollama::OllamaResponder::new("Qwen3.5-4B-GGUF"),
];
