use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub discord_token: String,

    pub database_url: String,

    pub s3: S3Config,

    #[serde(default)]
    pub web: WebServerConfig,

    pub discord_oauth: DiscordOauthConfig,
}

#[derive(Debug, Deserialize)]
pub struct WebServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_true")]
    pub secure_cookies: bool,
}

impl Default for WebServerConfig {
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
pub struct DiscordOauthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
    pub bucket: String,
}
