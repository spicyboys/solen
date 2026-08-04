use anyhow::Result;
use async_trait::async_trait;
use poise::serenity_prelude as serenity;
use rand::random_bool;
use regex_lite::{Regex, RegexBuilder};
use std::sync::LazyLock;

use super::Responder;
use serenity::all::{Context, CreateMessage, Message};

pub struct GrokResponder;

#[async_trait]
impl Responder for GrokResponder {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()> {
        static GROK_PROMPT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            RegexBuilder::new(r"^@?grok is this (true|real|chumble)\??$")
                .case_insensitive(true)
                .build()
                .unwrap()
        });

        if GROK_PROMPT_REGEX.is_match(&message.content) {
            let response = if random_bool(0.5) { "yes" } else { "no" };
            message
                .channel_id
                .send_message(
                    &ctx.http,
                    CreateMessage::new()
                        .content(response)
                        .reference_message(message),
                )
                .await?;
        }

        Ok(())
    }
}
