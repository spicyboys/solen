use serde::Deserialize;

use crate::config::DiscordOauthConfig;

pub fn authorize_url(oauth: &DiscordOauthConfig, state: &str) -> String {
    let mut url = url::Url::parse("https://discord.com/oauth2/authorize")
        .expect("static Discord authorize URL is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &oauth.client_id)
        .append_pair("redirect_uri", &oauth.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "identify guilds")
        .append_pair("state", state);
    url.to_string()
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
pub struct DiscordUser {
    pub id: String,
}

pub struct Identity {
    pub access_token: String,
    pub user: DiscordUser,
}

pub async fn authenticate(
    oauth: &DiscordOauthConfig,
    client: &reqwest::Client,
    code: &str,
) -> anyhow::Result<Identity> {
    let response = client
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&[
            ("client_id", oauth.client_id.as_str()),
            ("client_secret", oauth.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", oauth.redirect_uri.as_str()),
        ])
        .send()
        .await?;
    let token: TokenResponse = response.error_for_status()?.json().await?;

    let user = client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<DiscordUser>()
        .await?;

    Ok(Identity {
        access_token: token.access_token,
        user,
    })
}

#[derive(Deserialize)]
struct PartialGuild {
    id: String,
}

pub async fn is_guild_member(
    client: &reqwest::Client,
    access_token: &str,
    guild_id: u64,
) -> anyhow::Result<bool> {
    let guilds = client
        .get("https://discord.com/api/v10/users/@me/guilds")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<PartialGuild>>()
        .await?;
    Ok(guilds.iter().any(|guild| guild.id == guild_id.to_string()))
}
