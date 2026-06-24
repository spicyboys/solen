use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::all::prelude::Context;

pub struct DiscordUserLookupTool {
    pub ctx: Context,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct DiscordUserLookupParams {
    pub user_id: String,
}

impl Tool for DiscordUserLookupTool {
    fn name() -> &'static str {
        "discord_user_lookup"
    }

    fn description() -> &'static str {
        "A tool for looking up Discord user information."
    }

    type Params = DiscordUserLookupParams;

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> ollama_rs::generation::tools::Result<String> {
        println!("Looking up user with ID: {}", parameters.user_id);

        let Ok(user_id) = parameters.user_id.parse::<u64>() else {
            return Ok(format!("Malformed user ID: {}", parameters.user_id));
        };

        match self.ctx.http.get_user(user_id.into()).await {
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
