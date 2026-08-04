use poise::serenity_prelude::{self as serenity, CreateMessage};
use toasty::Db;
use tracing::error;

use serenity::{
    all::GuildChannel,
    model::{
        Permissions,
        channel::{PermissionOverwrite, PermissionOverwriteType},
    },
    prelude::*,
};

use crate::settings::{
    self,
    soundboard_manager::{SoundboardManagerConfig, SoundboardMode},
};

enum SoundboardState {
    Enabled,
    Disabled,
}

pub async fn handle_connection(ctx: &Context, db: &mut Db, channel: GuildChannel) {
    manage_soundboard(ctx, db, channel).await;
}

pub async fn handle_disconnection(ctx: &Context, db: &mut Db, channel: GuildChannel) {
    manage_soundboard(ctx, db, channel).await;
}

async fn manage_soundboard(ctx: &Context, db: &mut Db, channel: GuildChannel) {
    let Ok(config) = settings::get::<SoundboardManagerConfig>(db).await else {
        return;
    };

    let Some(mode) = config.0.get(&channel.id.to_string()) else {
        return;
    };

    let Ok(members) = channel.members(&ctx.cache) else {
        return;
    };

    let state = desired_state(mode, members.len());
    apply_soundboard_state(ctx, &channel, state).await;
}

fn desired_state(mode: &SoundboardMode, member_count: usize) -> SoundboardState {
    match mode {
        SoundboardMode::AlwaysEnabled => SoundboardState::Enabled,
        SoundboardMode::AlwaysDisabled => SoundboardState::Disabled,
        SoundboardMode::Managed { threshold } => {
            if member_count as i64 >= *threshold {
                SoundboardState::Disabled
            } else {
                SoundboardState::Enabled
            }
        }
    }
}

async fn apply_soundboard_state(ctx: &Context, channel: &GuildChannel, state: SoundboardState) {
    match state {
        SoundboardState::Enabled => enable_soundboard(ctx, channel).await,
        SoundboardState::Disabled => disable_soundboard(ctx, channel).await,
    }
}

/// Ensure the everyone role does not deny soundboard usage in `channel`.
async fn enable_soundboard(ctx: &Context, channel: &GuildChannel) {
    let Some(mut overwrite) = find_existing_overwrite(channel) else {
        return;
    };

    if !overwrite.deny.contains(Permissions::USE_SOUNDBOARD) {
        return;
    }

    overwrite.deny.remove(Permissions::USE_SOUNDBOARD);

    // If both allow and deny are now empty, delete the overwrite
    let res = if overwrite.allow.is_empty() && overwrite.deny.is_empty() {
        channel
            .id
            .delete_permission(
                &ctx.http,
                PermissionOverwriteType::Role(channel.base.guild_id.everyone_role()),
                None,
            )
            .await
    } else {
        // Update the overwrite with soundboard removed
        channel
            .id
            .create_permission(&ctx.http, overwrite, None)
            .await
    };

    match res {
        Ok(_) => {
            _ = channel
                .send_message(
                    ctx.http(),
                    CreateMessage::new().content("Soundboard has been enabled"),
                )
                .await
        }
        Err(e) => {
            error!("Failed to update channel permission override: {:?}", e)
        }
    };
}

/// Ensure the everyone role denies soundboard usage in `channel`.
async fn disable_soundboard(ctx: &Context, channel: &GuildChannel) {
    let res = match find_existing_overwrite(channel) {
        Some(mut overwrite) => {
            if overwrite.deny.contains(Permissions::USE_SOUNDBOARD) {
                return;
            }

            // Add soundboard to deny if not already there
            overwrite.deny |= Permissions::USE_SOUNDBOARD;
            channel
                .id
                .create_permission(&ctx.http, overwrite, None)
                .await
        }
        None => {
            // Create new permission override
            let everyone_overwrite = PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::USE_SOUNDBOARD,
                kind: PermissionOverwriteType::Role(channel.base.guild_id.everyone_role()),
            };
            channel
                .id
                .create_permission(&ctx.http, everyone_overwrite, None)
                .await
        }
    };

    match res {
        Ok(_) => {
            _ = channel
                .send_message(
                    ctx.http(),
                    CreateMessage::new().content("Soundboard has been disabled"),
                )
                .await
        }
        Err(e) => {
            error!("Failed to update channel permission override: {:?}", e)
        }
    };
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

#[cfg(test)]
mod tests {
    use super::{SoundboardState, desired_state};
    use crate::settings::soundboard_manager::SoundboardMode;

    #[test]
    fn desired_state_respects_each_mode() {
        let enabled = SoundboardMode::AlwaysEnabled;
        let disabled = SoundboardMode::AlwaysDisabled;
        let managed = SoundboardMode::Managed { threshold: 8 };

        assert!(matches!(
            desired_state(&enabled, 100),
            SoundboardState::Enabled
        ));
        assert!(matches!(
            desired_state(&disabled, 0),
            SoundboardState::Disabled
        ));
        assert!(matches!(
            desired_state(&managed, 7),
            SoundboardState::Enabled
        ));
        assert!(matches!(
            desired_state(&managed, 8),
            SoundboardState::Disabled
        ));
        assert!(matches!(
            desired_state(&managed, 9),
            SoundboardState::Disabled
        ));
    }
}
