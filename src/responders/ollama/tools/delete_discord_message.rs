use super::{Tool, ToolContext};
use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct DeleteDiscordMessageTool;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct DeleteDiscordMessageParams {
    channel_id: String,
    message_id: String,
    reason: Option<String>,
}

#[async_trait::async_trait]
impl Tool for DeleteDiscordMessageTool {
    type Params = DeleteDiscordMessageParams;

    const NAME: &'static str = "delete_discord_message";
    const DESCRIPTION: &'static str = "Delete a Discord message from a channel.";

    async fn call(ctx: &ToolContext, parameters: Self::Params) -> Result<String> {
        println!("Deleting message with parameters: {:?}", parameters,);

        let channel_id = parameters
            .channel_id
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Malformed channel ID: {}", parameters.channel_id))?;
        let message_id = parameters
            .message_id
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Malformed message ID: {}", parameters.message_id))?;

        let response = ctx
            .ctx
            .http
            .delete_message(
                channel_id.into(),
                message_id.into(),
                parameters.reason.as_deref(),
            )
            .await;

        Ok(match response {
            Ok(_) => format!(
                "Deleted message with ID {} in channel {}",
                message_id, channel_id
            ),
            Err(e) => format!(
                "Failed to delete message with ID {} in channel {}: {:?}",
                message_id, channel_id, e
            ),
        })
    }
}
