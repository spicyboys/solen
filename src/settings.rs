use anyhow::Result;
use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub discord_token: String,
    pub database_url: String,
    pub s3: S3Settings,
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
