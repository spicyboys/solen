use anyhow::Result;
use serenity::{
    all::{Context, Message},
    async_trait,
};
use url::Url;

use crate::{emojis, responders::Responder};

pub struct SlurpEnforcmentResponder;

#[async_trait]
impl Responder for SlurpEnforcmentResponder {
    async fn respond(&self, ctx: &Context, message: &Message) -> Result<()> {
        if message.channel_id != crate::channels::SLURP_SPREAD {
            // Only enforce in the slurp spread channel
            return Ok(());
        }

        if message.author.bot {
            // Ignore bot messages
            return Ok(());
        }

        if message.content.is_empty() {
            // Empty message means attachment only, which we assume is ok
            return Ok(());
        }

        if Url::parse(&message.content).is_err() {
            // Maybe something more complex later, but for now just react with the ban cat
            message.react(&ctx.http, emojis::CAT_BAN).await?;
            return Ok(());
        }

        Ok(())
    }
}
