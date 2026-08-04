use std::collections::HashMap;

use poise::serenity_prelude::{ChannelType, GuildChannel};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::Form,
        error::{SeeOther, bad_request, see_other},
        layout, page, path_param, route,
    },
    runtime::Event,
    view::{attributes, component, view},
};

use crate::{
    components::{
        button::{ButtonSize, ButtonVariant, button},
        table::{table, table_body, table_cell, table_head, table_header, table_row},
    },
    constants,
    settings::{
        self,
        soundboard_manager::{SoundboardManagerConfig, SoundboardMode},
    },
    web::{WebContext, auth},
};

const INPUT: &str = "h-8 rounded-md border border-border bg-background px-2 text-sm";

#[layout]
async fn layout(slot: Result) -> Result {
    view! {
        <header
            class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6"
        >
            <h1 class="text-lg font-semibold">"Settings"</h1>
        </header>
        <main class="flex-1 p-4 lg:p-6">(slot?)</main>
    }
}

#[page]
pub(crate) async fn index(cx: &Cx) -> Result {
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.db.clone();

    let (config_res, channels_res) = tokio::join!(
        settings::get::<SoundboardManagerConfig>(&mut db),
        ctx.discord_client.get_channels(constants::GUILD_ID),
    );
    let config = config_res?;
    let channels = channels_res?;

    let channel_names = channels
        .iter()
        .map(|channel| (channel.id.to_string(), channel.base.name.to_string()))
        .collect::<HashMap<_, _>>();
    let voice_channels = channels
        .into_iter()
        .filter(|channel| channel.base.kind == ChannelType::Voice)
        .filter(|v| !config.0.contains_key(&v.id.to_string()))
        .collect::<Vec<_>>();

    view! {
        <div class="flex flex-col gap-4">
            <div class="rounded-md border border-border">
                <div class="border-b border-border p-4">
                    <h2 class="text-lg font-semibold">"Soundboard Manager"</h2>
                    <p class="text-sm text-muted-foreground">
                        "How the soundboard manager treats each voice channel."
                    </p>
                </div>
                <div class="p-4">
                    <div class="rounded-md border border-border">
                        table(
                            table_header(
                                table_row(
                                    table_head("Channel")
                                    table_head("Mode")
                                    table_head("Actions")
                                )
                            )
                            table_body(
                                if config.0.is_empty() {
                                    table_row(
                                        table_cell(
                                            attrs: attributes! { colspan="3" class="text-center text-muted-foreground" },
                                            "No channels configured."
                                        )
                                    )
                                }
                                for (channel_id, mode) in &config.0 {
                                    table_row(
                                        table_cell(
                                            attrs: attributes! { class="font-medium" },
                                            (channel_names
                                                .get(channel_id)
                                                .cloned()
                                                .unwrap_or_else(|| { format!("{channel_id} (deleted)") }))
                                        )
                                        table_cell(
                                            mode_form(
                                                channel_id: channel_id.clone(),
                                                mode: mode.clone()
                                            )
                                        )
                                        table_cell(
                                            <form
                                                method="post"
                                                action=(format!(
                                                    "/settings/soundboard-manager/{}/remove", channel_id
                                                ))
                                            >
                                                button(
                                                    variant: ButtonVariant::Destructive,
                                                    size: ButtonSize::Sm,
                                                    attrs: attributes! { type="submit" },
                                                    "Remove"
                                                )
                                            </form>
                                        )
                                    )
                                }
                            )
                        )
                    </div>
                    add_channel_form(voice_channels: voice_channels)
                </div>
            </div>
        </div>
    }
}

/// The mode edit form for one configured channel.
#[component]
async fn mode_form(channel_id: String, mode: SoundboardMode) -> Result {
    view! {
        <form
            method="post"
            action="/settings/soundboard-manager"
            class="flex items-center gap-2"
        >
            <input type="hidden" name="channel_id" value=(channel_id)>
            <select name="mode" class=(INPUT)>
                <option
                    value="always_enabled"
                    selected=(matches!(mode, SoundboardMode::AlwaysEnabled))
                >
                    "Always enabled"
                </option>
                <option
                    value="always_disabled"
                    selected=(matches!(mode, SoundboardMode::AlwaysDisabled))
                >
                    "Always disabled"
                </option>
                <option
                    value="managed"
                    selected=(matches!(mode, SoundboardMode::Managed { .. }))
                >
                    "Managed"
                </option>
            </select>
            if matches!(mode, SoundboardMode::Managed { .. }) {
                <input
                    type="number"
                    name="threshold"
                    value=(mode.threshold().unwrap_or_default())
                    class=(INPUT)
                >
            }
            button(size: ButtonSize::Sm, attrs: attributes! { type="submit" }, "Save")
        </form>
    }
}

/// The form that adds a new channel to the config.
#[component]
async fn add_channel_form(voice_channels: Vec<GuildChannel>) -> Result {
    view! {
        signal mode = String::new();

        <form
            method="post"
            action="/settings/soundboard-manager"
            class="mt-4 flex flex-wrap items-end gap-2 border-t border-border pt-4"
        >
            <div>
                <span class="block text-xs font-medium text-muted-foreground">
                    "Channel"
                </span>
                <select name="channel_id" class=(INPUT)>
                    for channel in voice_channels {
                        <option value=(channel.id.to_string())>
                            (channel.base.name.to_string())
                        </option>
                    }
                </select>
            </div>
            <div>
                <span class="block text-xs font-medium text-muted-foreground">
                    "Mode"
                </span>
                <select
                    name="mode"
                    class=(INPUT)
                    @input=$(|e: Event| mode.set(e.target.value))
                >
                    <option value="always_enabled">"Always enabled"</option>
                    <option value="always_disabled">"Always disabled"</option>
                    <option value="managed">"Managed"</option>
                </select>
            </div>
            <div :hidden=$(mode.get() != "managed")>
                <span class="block text-xs font-medium text-muted-foreground">
                    "Threshold"
                </span>
                <input type="number" name="threshold" value=(8) class=(INPUT)>
            </div>
            button(
                size: ButtonSize::Sm,
                attrs: attributes! { type="submit" },
                "Add channel"
            )
        </form>
    }
}

#[derive(serde::Deserialize)]
struct ChannelConfigForm {
    channel_id: String,
    mode: String,
    #[serde(default)]
    threshold: Option<i64>,
}

#[route(POST "/settings/soundboard-manager")]
pub(crate) async fn upsert_channel(cx: &Cx, form: Form<ChannelConfigForm>) -> Result<SeeOther> {
    auth::require_auth(cx).await?;
    let ChannelConfigForm {
        channel_id,
        mode,
        threshold,
    } = form.0;
    let mode = SoundboardMode::from_form(&mode, threshold).map_err(bad_request)?;
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.db.clone();
    let mut config = settings::get::<SoundboardManagerConfig>(&mut db).await?;
    config.0.insert(channel_id, mode);
    settings::set(&mut db, config).await?;
    Ok(see_other("/settings"))
}

mod params {
    #[topcoat::router::path_param(error = bad_request)]
    pub struct ChannelId(String);
}

#[route(POST "/settings/soundboard-manager/{channel_id}/remove")]
pub(crate) async fn remove_channel(cx: &Cx) -> Result<SeeOther> {
    auth::require_auth(cx).await?;
    let channel_id = path_param::<params::ChannelId>(cx)?;
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.db.clone();
    let mut config = settings::get::<SoundboardManagerConfig>(&mut db).await?;
    config.0.remove(channel_id);
    settings::set(&mut db, config).await?;
    Ok(see_other("/settings"))
}
