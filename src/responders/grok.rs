use rand::random_bool;
use regex::{Regex, RegexBuilder};
use serenity::all::{Context, CreateMessage, Message};
use crate::responders::Responder;
use anyhow::Result;
use async_trait::async_trait;
use std::cell::LazyCell;

pub struct GrokResponder;

const GROK_PROMPT_REGEX: LazyCell<Regex> = LazyCell::new(|| {
    RegexBuilder::new(r"^@?grok is this true\??$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[async_trait]
impl Responder for GrokResponder {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()> {
        if GROK_PROMPT_REGEX.is_match(&message.content) {
            let response = if random_bool(0.5) {
                "yes"
            } else {
                "no"
            };
            message
                .channel_id
                .send_message(&ctx.http, CreateMessage::new().content(response).reference_message(message))
                .await?;
        }

        Ok(())
    }
}