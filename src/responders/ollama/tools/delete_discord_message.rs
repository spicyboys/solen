use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::all::prelude::Context;

pub struct DeleteDiscordMessageTool {
    pub ctx: Context,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct DeleteDiscordMessageParams {
    channel_id: String,
    message_id: String,
    reason: Option<String>,
}

impl Tool for DeleteDiscordMessageTool {
    fn name() -> &'static str {
        "delete_discord_message"
    }

    fn description() -> &'static str {
        "A tool for deleting a Discord message."
    }

    type Params = DeleteDiscordMessageParams;

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> ollama_rs::generation::tools::Result<String> {
        println!("Deleting message with parameters: {:?}", parameters,);

        let Ok(channel_id) = parameters.channel_id.parse::<u64>() else {
            return Ok(format!("Malformed channel ID: {}", parameters.channel_id));
        };
        let Ok(message_id) = parameters.message_id.parse::<u64>() else {
            return Ok(format!("Malformed message ID: {}", parameters.message_id));
        };

        let response = self
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
