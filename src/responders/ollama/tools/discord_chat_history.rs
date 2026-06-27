use super::{Tool, ToolContext};
use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::http::MessagePagination;

use crate::responders::ollama::format_message;

pub struct DiscordChatHistoryTool;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct DiscordChatHistoryParams {
    channel_id: String,
    #[schemars(
        description = "The message ID (NOT a timestamp) to retrieve messages before. If not provided, retrieves the most recent messages."
    )]
    before: Option<String>,
    #[schemars(description = "The maximum number of messages to retrieve.")]
    limit: Option<u8>,
}

#[async_trait::async_trait]
impl Tool for DiscordChatHistoryTool {
    type Params = DiscordChatHistoryParams;

    const NAME: &'static str = "discord_chat_history";
    const DESCRIPTION: &'static str = "Retrieve recent Discord chat history from a channel.";

    async fn call(ctx: &ToolContext, parameters: Self::Params) -> Result<String> {
        println!(
            "Retrieving messages from channel with parameters: {:?}",
            parameters,
        );

        let channel_id = parameters
            .channel_id
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Malformed channel ID: {}", parameters.channel_id))?;
        let before = parameters
            .before
            .map(|id| id.parse::<u64>())
            .transpose()
            .map_err(|_| anyhow::anyhow!("Malformed 'before' message ID"))?;

        let messages = match ctx
            .ctx
            .http
            .get_messages(
                channel_id.into(),
                before.map(|id| MessagePagination::Before(id.into())),
                parameters.limit,
            )
            .await
        {
            Ok(e) => e,
            Err(e) => {
                return Ok(format!(
                    "Failed to retrieve messages from channel {}: {:?}",
                    channel_id, e
                ));
            }
        };

        Ok(messages
            .iter()
            .map(format_message)
            .collect::<Vec<String>>()
            .join("\n"))
    }
}
