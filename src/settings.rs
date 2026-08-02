use anyhow::Result;
use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub discord_token: String,
    pub database_url: String,
    pub s3: S3Settings,
    #[serde(default)]
    pub web: WebSettings,
    pub discord_oauth: DiscordOauthSettings,
}

#[derive(Debug, Deserialize)]
pub struct WebSettings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub secure_cookies: bool,
}

impl Default for WebSettings {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            secure_cookies: default_true(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    59137
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct DiscordOauthSettings {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
pub struct S3Settings {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
    pub bucket: String,
}
