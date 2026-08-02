mod auth;
mod denied;
mod discord;
mod logged_in;
mod login;

use std::sync::Arc;

use poise::serenity_prelude as serenity;
use reqwest::Client as HttpClient;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::CxBuilder,
    cookie::RouterBuilderCookieExt,
    font::{Font, fontsource::fontsource_font},
    router::{
        Body, Next, Response, Router, RouterBuilderDiscoverExt, error::redirect, layer, layout,
        module_router, uri,
    },
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

// We don't serve anything on root right now, this just sends logged in people
// to soundboards so they don't get a 404
#[layer]
async fn redirect_logged_in(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    let uri = uri(cx);
    if uri.path() != "/" {
        Ok(next.run(cx, body).await?)
    } else if auth::current_user_id(cx).await.is_some() {
        Err(redirect("/soundboards").into())
    } else {
        Err(redirect("/login").into())
    }
}
