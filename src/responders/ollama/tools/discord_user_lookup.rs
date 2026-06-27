use super::{Tool, ToolContext};
use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct DiscordUserLookupTool;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct DiscordUserLookupParams {
    pub user_id: String,
}

#[async_trait::async_trait]
impl Tool for DiscordUserLookupTool {
    type Params = DiscordUserLookupParams;

    const NAME: &'static str = "discord_user_lookup";
    const DESCRIPTION: &'static str = "Look up a Discord user by ID.";

    async fn call(ctx: &ToolContext, parameters: Self::Params) -> Result<String> {
        println!("Looking up user with ID: {}", parameters.user_id);

        let user_id = parameters
            .user_id
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Malformed user ID: {}", parameters.user_id))?;

        match ctx.ctx.http.get_user(user_id.into()).await {
            Ok(user) => Ok(format!(
                "User ID: {}\nUsername: {}\nBot: {}",
                user.id, user.name, user.bot
            )),
            Err(e) => Ok(format!(
                "Failed to look up user with ID {}: {:?}",
                user_id, e
            )),
        }
    }
}
