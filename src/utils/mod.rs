use serenity::all::{ChannelType, Context, GuildChannel, GuildId, User};

pub trait UserUtils {
    async fn get_current_voice_channel(
        &self,
        ctx: &Context,
        guild_id: GuildId,
    ) -> Option<GuildChannel>;
}

impl UserUtils for User {
    async fn get_current_voice_channel(
        &self,
        ctx: &Context,
        guild_id: GuildId,
    ) -> Option<GuildChannel> {
        let Ok(channels) = guild_id.channels(&ctx.http).await else {
            return None;
        };

        channels.into_values().find(|channel| {
            if channel.kind != ChannelType::Voice {
                return false;
            }

            let Ok(members) = channel.members(&ctx.cache) else {
                return false;
            };

            members
                .iter()
                .find(|member| member.user.id == self.id)
                .is_some()
        })
    }
}
