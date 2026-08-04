pub mod soundboard_manager;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toasty::Db;

use crate::models::settings;

pub trait Setting: Default + for<'a> Deserialize<'a> + Serialize {
    const KEY: &'static str;
}

pub async fn get<T: Setting>(db: &mut Db) -> Result<T> {
    let Some(value) = get_value(db, T::KEY).await? else {
        return Ok(T::default());
    };
    serde_json::from_value(value).context("invalid soundboard_manager_config setting")
}

pub async fn set<T: Setting>(db: &mut Db, value: T) -> Result<()> {
    let value = serde_json::to_value(value)?;
    set_value(db, T::KEY, value).await
}

async fn get_value(db: &mut Db, key: &str) -> Result<Option<serde_json::Value>> {
    let record = settings::Model::filter_by_key(key.to_owned())
        .first()
        .exec(db)
        .await?;
    Ok(record.map(|record| record.value.0))
}

/// Store the JSON value for a setting `key`, inserting or updating the row.
async fn set_value(db: &mut Db, key: &str, value: serde_json::Value) -> Result<()> {
    settings::Model::upsert_by_key(key.to_owned())
        .value(value)
        .exec(db)
        .await?;
    Ok(())
}
