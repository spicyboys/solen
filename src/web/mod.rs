mod auth;
mod authed;
mod discord;
mod login;

use std::sync::Arc;

use poise::serenity_prelude as serenity;
use reqwest::Client as HttpClient;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    font::{Font, fontsource::fontsource_font},
    router::{Router, RouterBuilderDiscoverExt, layout, module_router},
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::view,
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

#[layout]
async fn root_layout(slot: Result) -> Result {
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
            <body>(slot?)</body>
        </html>
    }
}
