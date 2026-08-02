use poise::serenity_prelude::{self as serenity, GenericChannelId};

use serenity::{
    all::GuildChannel,
    model::{
        Permissions,
        channel::{PermissionOverwrite, PermissionOverwriteType},
    },
    prelude::*,
};

use crate::constants::channels::JALAPENO;

const CHANNEL_MEMBER_THRESHOLD: usize = 8;

pub async fn handle_connection(ctx: &Context, channel: GuildChannel) {
    if GenericChannelId::from(channel.id) != JALAPENO {
        return;
    }

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
                if let Err(e) = channel
                    .id
                    .create_permission(&ctx.http, overwrite, None)
                    .await
                {
                    eprintln!("Failed to update soundboard permission override: {:?}", e);
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
            if let Err(e) = channel
                .id
                .create_permission(&ctx.http, everyone_overwrite, None)
                .await
            {
                eprintln!("Failed to create soundboard permission override: {:?}", e);
            }
        }
    }
}

pub async fn handle_disconnection(ctx: &Context, channel: GuildChannel) {
    if GenericChannelId::from(channel.id) != JALAPENO {
        return;
    }

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
