use std::sync::LazyLock;

use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;
use serenity::all::Context;

pub mod delete_discord_message;
pub mod discord_chat_history;
pub mod discord_user_lookup;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub ctx: Context,
}

#[async_trait]
trait Tool {
    type Params: DeserializeOwned + JsonSchema + Send + 'static;

    const NAME: &'static str;

    const DESCRIPTION: &'static str;

    async fn call(ctx: &ToolContext, parameters: Self::Params) -> anyhow::Result<String>;
}

macro_rules! tools {
    ($($tool:ty),+ $(,)?) => {
        pub static TOOL_DEFINITIONS: LazyLock<Vec<ChatCompletionTools>> = LazyLock::new(|| {
            vec![
                $(
                    tool_definition(
                        <$tool as Tool>::NAME,
                        <$tool as Tool>::DESCRIPTION,
                        serde_json::to_value(schema_for!(<$tool as Tool>::Params)).unwrap(),
                    ),
                )+
            ]
        });

        pub async fn execute_tool_call(
            ctx: &ToolContext,
            name: &str,
            arguments: &str,
        ) -> anyhow::Result<String> {
            match name {
                $(
                    <$tool as Tool>::NAME => {
                        let params = serde_json::from_str::<<$tool as Tool>::Params>(arguments)?;
                        <$tool>::call(ctx, params).await
                    }
                )+
                _ => Ok(format!("Unknown tool: {name}")),
            }
        }
    };
}

tools![
    discord_chat_history::DiscordChatHistoryTool,
    discord_user_lookup::DiscordUserLookupTool,
    delete_discord_message::DeleteDiscordMessageTool,
];

fn tool_definition(
    name: &str,
    description: &str,
    schema: serde_json::Value,
) -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name(name)
            .description(description)
            .parameters(Some(schema))
            .build()
            .unwrap(),
    })
}
