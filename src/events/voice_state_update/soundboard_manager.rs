use std::collections::HashMap;

use open_feature::{StructValue, Value};
use poise::serenity_prelude::{self as serenity, CreateMessage};

use serenity::{
    all::GuildChannel,
    model::{
        Permissions,
        channel::{PermissionOverwrite, PermissionOverwriteType},
    },
    prelude::*,
};
use tracing::error;

const DEFAULT_MANAGED_THRESHOLD: i64 = 8;

enum SoundboardState {
    Enabled,
    Disabled,
}

pub async fn handle_connection(ctx: &Context, channel: GuildChannel) {
    manage_soundboard(ctx, channel).await;
}

pub async fn handle_disconnection(ctx: &Context, channel: GuildChannel) {
    manage_soundboard(ctx, channel).await;
}

async fn manage_soundboard(ctx: &Context, channel: GuildChannel) {
    let config = soundboard_config().await;
    let Some(channel_config) = config.0.get(&channel.id.to_string()) else {
        return;
    };

    let Ok(members) = channel.members(&ctx.cache) else {
        return;
    };

    let state = channel_config.desired_state(members.len());
    apply_soundboard_state(ctx, &channel, state).await;
}

async fn apply_soundboard_state(ctx: &Context, channel: &GuildChannel, state: SoundboardState) {
    match state {
        SoundboardState::Enabled => enable_soundboard(ctx, channel).await,
        SoundboardState::Disabled => disable_soundboard(ctx, channel).await,
    }
}

/// Ensure the everyone role does not deny soundboard usage in `channel`.
async fn enable_soundboard(ctx: &Context, channel: &GuildChannel) {
    if let Some(mut overwrite) = find_existing_overwrite(channel)
        && overwrite.deny.contains(Permissions::USE_SOUNDBOARD)
    {
        overwrite.deny.remove(Permissions::USE_SOUNDBOARD);

        // If both allow and deny are now empty, delete the overwrite
        if overwrite.allow.is_empty() && overwrite.deny.is_empty() {
            if let Err(e) = channel
                .id
                .delete_permission(
                    &ctx.http,
                    PermissionOverwriteType::Role(channel.base.guild_id.everyone_role()),
                    None,
                )
                .await
            {
                eprintln!("Failed to delete permission override: {:?}", e);
            }
        } else {
            // Update the overwrite with soundboard removed
            if let Err(e) = channel
                .id
                .create_permission(&ctx.http, overwrite, None)
                .await
            {
                eprintln!("Failed to update soundboard permission override: {:?}", e);
            }
        }

        _ = channel
            .send_message(
                ctx.http(),
                CreateMessage::new().content("Soundboard has been enabled"),
            )
            .await
    }
}

/// Ensure the everyone role denies soundboard usage in `channel`.
async fn disable_soundboard(ctx: &Context, channel: &GuildChannel) {
    let send_disabled_message = async || {
        _ = channel
            .send_message(
                ctx.http(),
                CreateMessage::new().content("Soundboard has been disabled"),
            )
            .await
    };

    match find_existing_overwrite(channel) {
        Some(mut overwrite) => {
            // Add soundboard to deny if not already there
            if !overwrite.deny.contains(Permissions::USE_SOUNDBOARD) {
                overwrite.deny |= Permissions::USE_SOUNDBOARD;
                match channel
                    .id
                    .create_permission(&ctx.http, overwrite, None)
                    .await
                {
                    Ok(_) => {
                        send_disabled_message().await;
                    }
                    Err(e) => {
                        error!("Failed to update soundboard permission override: {:?}", e)
                    }
                }
            }
        }
        None => {
            // Create new permission override
            let everyone_overwrite = PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::USE_SOUNDBOARD,
                kind: PermissionOverwriteType::Role(channel.base.guild_id.everyone_role()),
            };
            match channel
                .id
                .create_permission(&ctx.http, everyone_overwrite, None)
                .await
            {
                Ok(_) => {
                    send_disabled_message().await;
                }
                Err(e) => {
                    error!("Failed to update soundboard permission override: {:?}", e)
                }
            }
        }
    }
}

fn find_existing_overwrite(channel: &GuildChannel) -> Option<PermissionOverwrite> {
    channel
        .permission_overwrites
        .iter()
        .find(|overwrite| {
            overwrite.kind == PermissionOverwriteType::Role(channel.base.guild_id.everyone_role())
        })
        .cloned()
}

enum SoundboardManagerChannelConfig {
    AlwaysEnabled,
    AlwaysDisabled,
    Managed(i64),
}

impl SoundboardManagerChannelConfig {
    fn desired_state(&self, member_count: usize) -> SoundboardState {
        match self {
            SoundboardManagerChannelConfig::AlwaysEnabled => SoundboardState::Enabled,
            SoundboardManagerChannelConfig::AlwaysDisabled => SoundboardState::Disabled,
            SoundboardManagerChannelConfig::Managed(threshold) => {
                if member_count as i64 >= *threshold {
                    SoundboardState::Disabled
                } else {
                    SoundboardState::Enabled
                }
            }
        }
    }
}

#[derive(Default)]
struct SoundboardManagerConfig(HashMap<String, SoundboardManagerChannelConfig>);

impl From<StructValue> for SoundboardManagerConfig {
    fn from(value: StructValue) -> Self {
        let channels = value
            .fields
            .into_iter()
            .filter_map(|(channel_id, value)| {
                let config = match value {
                    Value::String(mode) => parse_mode(&mode),
                    Value::Struct(config) => {
                        let mode = config.fields.get("mode").and_then(Value::as_str)?;
                        match mode {
                            "always_enabled" => Some(SoundboardManagerChannelConfig::AlwaysEnabled),
                            "always_disabled" => {
                                Some(SoundboardManagerChannelConfig::AlwaysDisabled)
                            }
                            "managed" => Some(SoundboardManagerChannelConfig::Managed(
                                config
                                    .fields
                                    .get("threshold")
                                    .and_then(Value::as_i64)
                                    .unwrap_or(DEFAULT_MANAGED_THRESHOLD),
                            )),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                config.map(|config| (channel_id, config))
            })
            .collect();
        SoundboardManagerConfig(channels)
    }
}

fn parse_mode(mode: &str) -> Option<SoundboardManagerChannelConfig> {
    match mode {
        "always_enabled" => Some(SoundboardManagerChannelConfig::AlwaysEnabled),
        "always_disabled" => Some(SoundboardManagerChannelConfig::AlwaysDisabled),
        "managed" => Some(SoundboardManagerChannelConfig::Managed(
            DEFAULT_MANAGED_THRESHOLD,
        )),
        _ => None,
    }
}

async fn soundboard_config() -> SoundboardManagerConfig {
    let client = open_feature::OpenFeature::singleton().await.create_client();
    client
        .get_struct_value::<SoundboardManagerConfig>("soundboard_manager_config", None, None)
        .await
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{SoundboardManagerChannelConfig, SoundboardState, parse_mode};

    #[test]
    fn parse_mode_recognizes_each_mode() {
        assert!(matches!(
            parse_mode("always_enabled"),
            Some(SoundboardManagerChannelConfig::AlwaysEnabled)
        ));
        assert!(matches!(
            parse_mode("always_disabled"),
            Some(SoundboardManagerChannelConfig::AlwaysDisabled)
        ));
        assert!(matches!(
            parse_mode("managed"),
            Some(SoundboardManagerChannelConfig::Managed(8))
        ));
        assert!(parse_mode("unknown").is_none());
    }

    #[test]
    fn desired_state_respects_each_mode() {
        let enabled = SoundboardManagerChannelConfig::AlwaysEnabled;
        let disabled = SoundboardManagerChannelConfig::AlwaysDisabled;
        let managed = SoundboardManagerChannelConfig::Managed(8);

        assert!(matches!(
            enabled.desired_state(100),
            SoundboardState::Enabled
        ));
        assert!(matches!(
            disabled.desired_state(0),
            SoundboardState::Disabled
        ));
        assert!(matches!(managed.desired_state(7), SoundboardState::Enabled));
        assert!(matches!(
            managed.desired_state(8),
            SoundboardState::Disabled
        ));
        assert!(matches!(
            managed.desired_state(9),
            SoundboardState::Disabled
        ));
    }

    #[test]
    fn config_from_struct_value_parses_channels() {
        let mut fields = HashMap::new();
        fields.insert(
            "always_channel".to_owned(),
            open_feature::Value::String("always_enabled".to_owned()),
        );
        fields.insert(
            "managed_channel".to_owned(),
            open_feature::Value::Struct(open_feature::StructValue {
                fields: HashMap::from([
                    (
                        "mode".to_owned(),
                        open_feature::Value::String("managed".to_owned()),
                    ),
                    ("threshold".to_owned(), open_feature::Value::Int(4)),
                ]),
            }),
        );
        fields.insert(
            "unknown_channel".to_owned(),
            open_feature::Value::String("bogus".to_owned()),
        );

        let config = super::SoundboardManagerConfig::from(open_feature::StructValue { fields });

        assert!(matches!(
            config.0.get("always_channel"),
            Some(SoundboardManagerChannelConfig::AlwaysEnabled)
        ));
        assert!(matches!(
            config.0.get("managed_channel"),
            Some(SoundboardManagerChannelConfig::Managed(4))
        ));
        assert!(!config.0.contains_key("unknown_channel"));
    }
}
