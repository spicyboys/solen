mod auth;
mod discord;
mod soundboards;

use std::sync::Arc;

use poise::serenity_prelude as serenity;
use reqwest::Client as HttpClient;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    cookie::RouterBuilderCookieExt,
    font::{Font, fontsource::fontsource_font},
    router::{Router, RouterBuilderDiscoverExt, layout, module_router},
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::{attributes, view},
};

use topcoat::icon::{icon, iconify::iconify_icon};
use topcoat::runtime::Event;

use crate::components::separator::{SeparatorOrientation, separator};
use crate::components::sidebar::{
    sidebar, sidebar_content, sidebar_footer, sidebar_group, sidebar_group_content,
    sidebar_group_label, sidebar_header, sidebar_inset, sidebar_menu, sidebar_menu_button,
    sidebar_menu_item, sidebar_trigger,
};
use crate::settings::DiscordOauthSettings;

use auth::SessionCookieStore;

pub struct WebContext {
    pub data: Arc<crate::Data>,
    pub http: Arc<serenity::http::Http>,
    pub oauth: DiscordOauthSettings,
    pub secure_cookies: bool,
    pub client: HttpClient,
}

pub fn router(ctx: WebContext) -> Router {
    let session_config = SessionConfig::builder()
        .token_store(SessionCookieStore::new(ctx.secure_cookies))
        .build();

    module_router!()
        .discover()
        .app_context(ctx)
        .cookies()
        .sessions(session_config)
        .assets(AssetBundle::load().unwrap())
        .build()
}

const ROBOTO: Font = fontsource_font!(ROBOTO, host: Asset);

#[layout("/")]
async fn root_layout(cx: &Cx, slot: Result) -> Result {
    let authed: bool = auth::current_user_id(cx).await.is_some();
    if authed {
        view! {
            <!DOCTYPE html>
            <html>
                <head>
                    topcoat::font::link(font: ROBOTO)
                    <link rel="stylesheet" href=(tailwind::stylesheet!())>
                    <meta charset="utf-8">
                    <title>"Solen"</title>

                    // Reload the page automatically during development.
                    topcoat::dev::script()

                    // Load the browser runtime used by signals and event handlers.
                    topcoat::runtime::script()
                </head>
                <body>
                    signal mobile_open = false;
                    signal collapsed = false;

                    <div class="flex min-h-dvh">
                        // A dimmed backdrop behind the drawer on small screens;
                        // clicking it closes the drawer.
                        <div
                            :hidden=$(!mobile_open.get())
                            class="fixed inset-0 z-30 bg-black/50 md:hidden"
                            @click=$(|_e: Event| mobile_open.set(false))
                        ></div>
                        sidebar(
                            attrs: attributes! {
                                :data-open=$(if mobile_open.get() { "true" } else { "false" })
                                :data-collapsed=$(if collapsed.get() { "true" } else { "false" })
                            },
                            sidebar_header(
                                <a href="/" class="flex items-center gap-2 font-semibold md:group-data-[collapsed=true]:justify-center">
                                    <span class="md:group-data-[collapsed=true]:hidden">"Solen"</span>
                                </a>
                            )
                            sidebar_content(
                                sidebar_group(
                                    sidebar_group_label("General")
                                    sidebar_group_content(
                                        sidebar_menu(
                                            sidebar_menu_item(
                                                sidebar_menu_button(
                                                    attrs: attributes! { href="/" data-active="" },
                                                    icon(data: iconify_icon!("feather:disc"))
                                                    <span class="md:group-data-[collapsed=true]:hidden">"Soundboards"</span>
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                            sidebar_footer(
                                sidebar_menu(
                                    sidebar_menu_item(
                                        sidebar_menu_button(
                                            attrs: attributes! { href="/logout" },
                                            icon(data: iconify_icon!("feather:log-out"))
                                            <span class="md:group-data-[collapsed=true]:hidden">"Log out"</span>
                                        )
                                    )
                                )
                            )
                        )
                        sidebar_inset((slot?))
                    </div>
                </body>
            </html>
        }
    } else {
        view! {
            <!DOCTYPE html>
            <html>
                <head>
                    topcoat::font::link(font: ROBOTO)
                    <link rel="stylesheet" href=(tailwind::stylesheet!())>
                    <meta charset="utf-8">
                    <title>"Solen"</title>

                    // Reload the page automatically during development.
                    topcoat::dev::script()

                    // Load the browser runtime used by signals and event handlers.
                    topcoat::runtime::script()
                </head>
                <body>
                    <main class="flex min-h-dvh items-center justify-center p-4">
                        <div class="w-full max-w-md">(slot?)</div>
                    </main>
                </body>
            </html>
        }
    }
}
