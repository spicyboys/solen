use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::{all::prelude::Context, http::MessagePagination};

use crate::responders::ollama::format_message;

pub struct DiscordChatHistoryTool {
    pub ctx: Context,
}

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

impl Tool for DiscordChatHistoryTool {
    fn name() -> &'static str {
        "discord_chat_history"
    }

    fn description() -> &'static str {
        "A tool for retrieving Discord chat history."
    }

    type Params = DiscordChatHistoryParams;

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> ollama_rs::generation::tools::Result<String> {
        println!(
            "Retrieving messages from channel with parameters: {:?}",
            parameters,
        );

        let Ok(channel_id) = parameters.channel_id.parse::<u64>() else {
            return Ok(format!("Malformed channel ID: {}", parameters.channel_id));
        };
        let Ok(before) = parameters.before.map(|id| id.parse::<u64>()).transpose() else {
            return Ok("Malformed 'before' message ID".to_string());
        };

        let messages = match self
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
