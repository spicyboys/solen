use serenity::{
    all::GuildChannel,
    model::{
        Permissions,
        channel::{Channel, ChannelType, PermissionOverwrite, PermissionOverwriteType},
        id::ChannelId,
        voice::VoiceState,
    },
    prelude::*,
};

const CHANNEL_MEMBER_THRESHOLD: usize = 6;

pub async fn voice_state_update(ctx: Context, old: Option<VoiceState>, new: VoiceState) {
    // Handle user connecting to a new channel
    if let Some(channel) = get_voice_channel(&ctx, new.channel_id).await {
        handle_connection(&ctx, channel).await;
    }

    // Handle user disconnecting from a channel
    if let Some(channel) = get_voice_channel(&ctx, old.and_then(|o| o.channel_id)).await {
        handle_disconnection(&ctx, channel).await;
    }
}

async fn get_voice_channel(ctx: &Context, channel_id: Option<ChannelId>) -> Option<GuildChannel> {
    if let Some(new_channel_id) = channel_id
        && let Ok(Channel::Guild(channel)) = new_channel_id.to_channel(ctx.http()).await
        && channel.kind == ChannelType::Voice
    {
        Some(channel)
    } else {
        None
    }
}

async fn handle_connection(ctx: &Context, channel: GuildChannel) {
    let Ok(members) = channel.members(&ctx.cache) else {
        return;
    };

    if members.len() < CHANNEL_MEMBER_THRESHOLD {
        return;
    }

    // Need to ensure soundboard is disabled
    match find_existing_overwrite(&channel) {
        Some(mut overwrite) => {
            // Add soundboard to deny if not already there
            if !overwrite.deny.contains(Permissions::USE_SOUNDBOARD) {
                overwrite.deny |= Permissions::USE_SOUNDBOARD;
                if let Err(e) = channel.create_permission(&ctx.http(), overwrite).await {
                    eprintln!("Failed to update soundboard permission override: {:?}", e);
                }
            }
        }
        None => {
            // Create new permission override
            let everyone_overwrite = PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::USE_SOUNDBOARD,
                kind: PermissionOverwriteType::Role(channel.guild_id.everyone_role()),
            };
            if let Err(e) = channel
                .create_permission(&ctx.http(), everyone_overwrite)
                .await
            {
                eprintln!("Failed to create soundboard permission override: {:?}", e);
            }
        }
    }
}

async fn handle_disconnection(ctx: &Context, channel: GuildChannel) {
    let Ok(members) = channel.members(&ctx.cache) else {
        return;
    };

    if members.len() >= CHANNEL_MEMBER_THRESHOLD {
        return;
    }

    // Need to remove soundboard denial if it exists
    if let Some(mut overwrite) = find_existing_overwrite(&channel)
        && overwrite.deny.contains(Permissions::USE_SOUNDBOARD)
    {
        overwrite.deny.remove(Permissions::USE_SOUNDBOARD);

        // If both allow and deny are now empty, delete the overwrite
        if overwrite.allow.is_empty() && overwrite.deny.is_empty() {
            if let Err(e) = channel
                .delete_permission(
                    &ctx.http(),
                    PermissionOverwriteType::Role(channel.guild_id.everyone_role()),
                )
                .await
            {
                eprintln!("Failed to delete permission override: {:?}", e);
            }
        } else {
            // Update the overwrite with soundboard removed
            if let Err(e) = channel.create_permission(&ctx.http(), overwrite).await {
                eprintln!("Failed to update soundboard permission override: {:?}", e);
            }
        }
    }
}

fn find_existing_overwrite(channel: &GuildChannel) -> Option<PermissionOverwrite> {
    channel
        .permission_overwrites
        .iter()
        .find(|overwrite| {
            overwrite.kind == PermissionOverwriteType::Role(channel.guild_id.everyone_role())
        })
        .cloned()
}
