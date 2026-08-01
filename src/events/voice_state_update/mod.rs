use poise::serenity_prelude::{
    Channel, ChannelId, ChannelType, Context, GenericChannelId, GuildChannel, GuildId, VoiceState,
};

mod soundboard_manager;

pub async fn handle_voice_state_update(ctx: &Context, old: Option<&VoiceState>, new: &VoiceState) {
    // Handle user connecting to a new channel
    if let Some(channel) = get_voice_channel(ctx, new.channel_id, new.guild_id).await {
        soundboard_manager::handle_connection(ctx, channel).await;
    }

    // Handle user disconnecting from a channel
    if let Some(old_state) = old
        && let Some(channel) =
            get_voice_channel(ctx, old_state.channel_id, old_state.guild_id).await
    {
        soundboard_manager::handle_disconnection(ctx, channel).await;
    }
}

async fn get_voice_channel(
    ctx: &Context,
    channel_id: Option<ChannelId>,
    guild_id: Option<GuildId>,
) -> Option<GuildChannel> {
    if let Some(channel_id) = channel_id.map(GenericChannelId::from)
        && channel_id != crate::constants::channels::POBLANO
        && let Ok(Channel::Guild(channel)) = channel_id.to_channel(ctx, guild_id).await
        && channel.base.kind == ChannelType::Voice
    {
        Some(channel)
    } else {
        None
    }
}
