use std::time::{Duration, SystemTime, UNIX_EPOCH};

use topcoat::{
    Result,
    context::{Cx, app_context},
    cookie::{Cookie, Cookies, SameSite, cookies, time as cookie_time},
    router::{
        error::{SeeOther, internal_server_error, redirect, see_other},
        query_params, route,
    },
    session::{Token, TokenHash, TokenStore, TokenStoreFuture, start, stop, token_hash},
};
use tracing::{info, trace, warn};

use crate::{
    constants,
    models::web_sessions,
    web::{WebContext, discord},
};

const SESSION_COOKIE: &str = "session";
const OAUTH_STATE_COOKIE: &str = "oauth_state";

pub struct SessionCookieStore {
    secure: bool,
}

impl SessionCookieStore {
    #[must_use]
    pub fn new(secure: bool) -> Self {
        Self { secure }
    }
}

impl TokenStore for SessionCookieStore {
    fn read<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, Option<Token>> {
        Box::pin(async move {
            let Some(cookie) = cookies(cx).get(SESSION_COOKIE) else {
                return Ok(None);
            };
            Ok(Token::decode(cookie.value_trimmed()).ok())
        })
    }

    fn write<'a>(
        &'a self,
        cx: &'a Cx,
        token: Token,
        max_age: Duration,
    ) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let max_age = cookie_time::Duration::try_from(max_age)
                .map_err(|error| internal_server_error(anyhow::anyhow!("{error}")))?;
            cookies(cx)
                .override_http_only(true)
                .override_same_site(SameSite::Lax)
                .override_path("/")
                .override_secure(self.secure)
                .override_max_age(max_age)
                .add(Cookie::new(SESSION_COOKIE, token.encode()));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            cookies(cx)
                .override_http_only(true)
                .override_same_site(SameSite::Lax)
                .override_path("/")
                .override_secure(self.secure)
                .remove(Cookie::new(SESSION_COOKIE, ""));
            Ok(())
        })
    }
}

pub async fn current_user_id(cx: &Cx) -> Option<String> {
    let hash = match token_hash(cx).await {
        Ok(Some(hash)) => hash,
        Ok(None) => {
            trace!("no session token present");
            return None;
        }
        Err(error) => {
            warn!("failed to read session token: {error:?}");
            return None;
        }
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(i64::MAX);
    let mut db = app_context::<WebContext>(cx).data.db.clone();
    let record = match web_sessions::Model::filter_by_token_hash(hash_to_hex(&hash))
        .first()
        .exec(&mut db)
        .await
    {
        Ok(record) => record,
        Err(error) => {
            warn!("session lookup failed: {error:?}");
            return None;
        }
    };
    let Some(record) = record else {
        warn!(token_hash = %hash_to_hex(&hash), "no session row found");
        return None;
    };
    if record.expires_at < now {
        warn!(
            token_hash = %hash_to_hex(&hash),
            expires_at = record.expires_at,
            now,
            "session expired"
        );
        return None;
    }
    Some(record.user_id)
}

pub async fn require_auth(cx: &Cx) -> Result<String> {
    match current_user_id(cx).await {
        Some(user_id) => Ok(user_id),
        None => {
            warn!("redirecting unauthenticated request to /login");
            Err(redirect("/login").into())
        }
    }
}

fn hash_to_hex(hash: &TokenHash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[route(GET "/oauth/discord")]
pub(crate) async fn oauth_discord(cx: &Cx) -> Result<SeeOther> {
    let ctx = app_context::<WebContext>(cx);
    let state = Token::random().encode();
    cookies(cx)
        .override_http_only(true)
        .override_same_site(SameSite::Lax)
        .override_path("/")
        .override_secure(ctx.secure_cookies)
        .add(Cookie::new(OAUTH_STATE_COOKIE, state.clone()));
    let url = discord::authorize_url(&ctx.oauth, &state);
    info!("starting discord oauth, redirecting to {}", url);
    Ok(see_other(&url))
}

#[query_params(error = bad_request)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[route(GET "/oauth/callback")]
pub(crate) async fn oauth_callback(cx: &Cx) -> Result<SeeOther> {
    let query = query_params::<CallbackQuery>(cx)?;
    info!(
        code = query.code.is_some(),
        state = query.state.is_some(),
        "oauth callback received"
    );
    let ctx = app_context::<WebContext>(cx);
    let (Some(code), Some(state)) = (query.code.as_deref(), query.state.as_deref()) else {
        warn!(
            code = query.code.as_deref().is_some(),
            state = query.state.as_deref().is_some(),
            "oauth callback rejected: missing code or state"
        );
        return Ok(see_other("/login"));
    };
    let expected = cookies(cx)
        .get(OAUTH_STATE_COOKIE)
        .map(|state_cookie| state_cookie.value().to_owned());
    if expected.as_deref() != Some(state) {
        warn!(
            expected = expected.as_deref(),
            got = state,
            "oauth callback rejected: state mismatch"
        );
        return Ok(see_other("/login"));
    }
    cookies(cx)
        .override_path("/")
        .override_secure(ctx.secure_cookies)
        .remove(Cookie::new(OAUTH_STATE_COOKIE, ""));

    let identity = match discord::authenticate(&ctx.oauth, &ctx.client, code).await {
        Ok(identity) => identity,
        Err(error) => {
            warn!("oauth callback rejected: discord authenticate failed: {error:?}");
            return Ok(see_other("/login"));
        }
    };
    info!(user_id = %identity.user.id, "discord identity resolved");
    let is_member = match discord::is_guild_member(
        &ctx.client,
        &identity.access_token,
        constants::GUILD_ID.into(),
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            warn!("oauth callback rejected: guild membership check failed: {error:?}");
            return Ok(see_other("/login"));
        }
    };
    if !is_member {
        warn!(user_id = %identity.user.id, "oauth callback rejected: user is not a guild member");
        return Ok(see_other("/denied"));
    }

    let session = start(cx).await?;
    let expires_at = session
        .expires_at
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let user_id = identity.user.id;
    let mut db = ctx.data.db.clone();
    toasty::create!(web_sessions::Model {
        token_hash: hash_to_hex(&session.token_hash),
        user_id: user_id.clone(),
        expires_at,
    })
    .exec(&mut db)
    .await
    .map_err(internal_server_error)?;
    info!(
        token_hash = %hash_to_hex(&session.token_hash),
        user_id,
        expires_at,
        "session created, redirecting to /"
    );

    Ok(see_other("/"))
}

#[route(GET "/logout")]
pub(crate) async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(hash) = stop(cx).await? {
        let mut db = app_context::<WebContext>(cx).data.db.clone();
        let _ = web_sessions::Model::filter_by_token_hash(hash_to_hex(&hash))
            .delete()
            .exec(&mut db)
            .await;
    }
    Ok(see_other("/login"))
}
