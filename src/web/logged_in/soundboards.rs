use topcoat::{
    Result,
    context::{Cx, app_context},
    icon::{icon, iconify::iconify_icon},
    router::{
        error::{SeeOther, not_found, see_other},
        {Body, Response, header, layout, page, path_param, route},
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
    models::archived_soundboards,
    web::{WebContext, auth},
};

#[path_param(error = not_found)]
struct SoundId(String);

#[page]
pub(crate) async fn index(cx: &Cx) -> Result {
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();

    let records = archived_soundboards::Model::all().exec(&mut db).await?;
    let installed_soundboards = ctx.http.get_guild_soundboards(constants::GUILD_ID).await?;
    let guild_members = ctx
        .http
        .get_guild_members(constants::GUILD_ID, None, None)
        .await?;

    view! {
        <div class="flex flex-col gap-4">
            <div class="rounded-md border border-border">
                table(
                    table_header(
                        table_row(
                            table_head("Emoji")
                            table_head("Name")
                            table_head("Uploaded by")
                            table_head("Preview")
                            table_head("Actions")
                        )
                    )
                    table_body(
                        for record in records {
                            table_row(
                                table_cell(
                                    soundboard_emoji(
                                        emoji_id: record.emoji_id.clone(),
                                        emoji_name: record.emoji_name.clone()
                                    )
                                )
                                table_cell(
                                    attrs: attributes! { class="font-medium" },
                                    (record.name)
                                )
                                table_cell(
                                    (record
                                        .original_uploader
                                        .and_then(|u| {
                                            guild_members.iter().find(|g| g.user.id.to_string() == u)
                                        })
                                        .map(|u| u.display_name())
                                        .unwrap_or_else(|| "unknown"))
                                )
                                table_cell(
                                    preview_soundboard(sound_id: record.sound_id.clone())
                                )
                                table_cell(
                                    <form
                                        id=(format!("restore-{}", record.sound_id))
                                        method="post"
                                        action=(format!(
                                            "/soundboards/{}/restore", record.sound_id
                                        ))
                                    ></form>
                                    button(
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! {
                                            type="submit"
                                            form=(format!("restore-{}", record.sound_id))
                                            disabled=(installed_soundboards
                                                .iter()
                                                .any(|s| s.id.to_string() == record.sound_id))
                                        },
                                        "Restore"
                                    )
                                )
                            )
                        }
                    )
                )
            </div>
        </div>
    }
}

#[layout]
async fn layout(slot: Result) -> Result {
    view! {
        <header
            class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6"
        >
            <h1 class="text-lg font-semibold">"Soundboards"</h1>
            <p class="text-sm text-muted-foreground">
                "Archived soundboards from the server."
            </p>
        </header>
        <main class="flex-1 p-4 lg:p-6">(slot?)</main>
    }
}

/// A play button for a soundboard's preview audio.
#[component]
async fn preview_soundboard(sound_id: String) -> Result {
    view! {
        button(
            variant: ButtonVariant::Outline,
            size: ButtonSize::Icon,
            attrs: attributes! {
                @click=$(|_e: Event| {
                    raw!("new Audio('/soundboards/' + ${sound_id} + '/preview').play()")
                })
            },
            icon(data: iconify_icon!("feather:play"))
        )
    }
}

/// The emoji for a soundboard: a custom emoji image, a unicode character,
/// or a placeholder when neither was archived.
#[component]
async fn soundboard_emoji(emoji_id: Option<String>, emoji_name: Option<String>) -> Result {
    if let Some(emoji_id) = emoji_id {
        let alt = emoji_name.unwrap_or_default();
        return view! {
            <img
                src=(format!(
                    "https://cdn.discordapp.com/emojis/{emoji_id}.png?size=32"
                ))
                alt=(alt.clone())
                title=(alt)
                class="size-5"
            />
        };
    }
    view! { (emoji_name.unwrap_or_else(|| "\u{2013}".to_owned())) }
}

#[route(POST "/soundboards/{sound_id}/restore")]
pub(crate) async fn restore(cx: &Cx) -> Result<SeeOther> {
    auth::require_auth(cx).await?;
    let sound_id = path_param::<SoundId>(cx)?;
    let ctx = app_context::<WebContext>(cx);
    crate::commands::perform_restore(
        &ctx.data.db,
        &ctx.data.s3,
        &ctx.http,
        constants::GUILD_ID,
        sound_id,
    )
    .await
    .map_err(|error| anyhow::anyhow!("{error}"))?;
    Ok(see_other("/dashboard"))
}

#[route(GET "/soundboards/{sound_id}/preview")]
pub(crate) async fn preview(cx: &Cx) -> Result<Response> {
    let sound_id = path_param::<SoundId>(cx)?;
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();
    let record = archived_soundboards::Model::filter_by_sound_id(sound_id.clone())
        .first()
        .exec(&mut db)
        .await?
        .ok_or_else(not_found)?;

    let bytes = ctx.data.s3.download_bytes(&record.s3_key).await?;
    let mime = crate::commands::detect_audio_mime(&bytes);

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(bytes))
        .expect("preview response should build"))
}
