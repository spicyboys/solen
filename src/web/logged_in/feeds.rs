use std::{str::FromStr, sync::Arc};

use poise::serenity_prelude::{self as serenity, Channel, GenericChannelId};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{layout, page},
    view::{attributes, component, view},
};

use crate::{
    components::table::{table, table_body, table_cell, table_head, table_header, table_row},
    models::feeds,
    web::WebContext,
};

#[layout]
async fn layout(slot: Result) -> Result {
    view! {
        <header
            class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6"
        >
            <h1 class="text-lg font-semibold">"Feeds"</h1>
            <p class="text-sm text-muted-foreground">
                "Configured RSS and ntfy subscriptions."
            </p>
        </header>
        <main class="flex-1 p-4 lg:p-6">(slot?)</main>
    }
}

#[page]
pub(crate) async fn index(cx: &Cx) -> Result {
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();

    let records = feeds::Model::all().exec(&mut db).await?;

    view! {
        <div class="flex flex-col gap-4">
            <div class="rounded-md border border-border">
                table(
                    table_header(
                        table_row(
                            table_head("Channel")
                            table_head("Feed")
                            table_head("Latest post")
                        )
                    )
                    table_body(
                        for record in records {
                            table_row(
                                table_cell(
                                    attrs: attributes! { class="font-medium" },
                                    channel_display_name(
                                        channel_id: record.channel_id,
                                        discord: ctx.http.clone()
                                    )
                                )
                                table_cell(
                                    attrs: attributes! { class="max-w-[32rem] break-all font-mono text-sm" },
                                    (record.feed)
                                )
                                table_cell(
                                    attrs: attributes! { class="max-w-[24rem] break-all text-sm" },
                                    (record.latest_post)
                                )
                            )
                        }
                    )
                )
            </div>
        </div>
    }
}

#[component]
async fn channel_display_name(channel_id: String, discord: Arc<serenity::http::Http>) -> Result {
    let channel = discord
        .get_channel(GenericChannelId::from_str(&channel_id)?)
        .await?;

    view! {
        match channel {
            Channel::Guild(guild_channel) => (guild_channel.base.name.to_string()),
            Channel::GuildThread(thread) => (thread.base.name.to_string()),
            _ => (channel_id),
        }
    }
}
