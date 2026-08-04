use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::settings::Setting;

const DEFAULT_MANAGED_THRESHOLD: i64 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SoundboardMode {
    AlwaysEnabled,
    AlwaysDisabled,
    Managed { threshold: i64 },
}

impl SoundboardMode {
    /// Parse a mode submitted by the settings form.
    pub fn from_form(mode: &str, threshold: Option<i64>) -> Result<Self, String> {
        match mode {
            "always_enabled" => Ok(Self::AlwaysEnabled),
            "always_disabled" => Ok(Self::AlwaysDisabled),
            "managed" => Ok(Self::Managed {
                threshold: threshold.unwrap_or(DEFAULT_MANAGED_THRESHOLD),
            }),
            other => Err(format!("unknown soundboard mode: {other:?}")),
        }
    }

    /// The threshold for a `managed` channel, if any.
    pub fn threshold(&self) -> Option<i64> {
        match self {
            Self::Managed { threshold } => Some(*threshold),
            Self::AlwaysEnabled | Self::AlwaysDisabled => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundboardManagerConfig(pub BTreeMap<String, SoundboardMode>);

impl Setting for SoundboardManagerConfig {
    const KEY: &'static str = "soundboard_manager_config";
}

#[cfg(test)]
mod tests {
    use super::{SoundboardManagerConfig, SoundboardMode};

    #[test]
    fn round_trips_through_faithful_json() {
        let config = SoundboardManagerConfig(
            [
                (
                    "1002692827312037910".to_owned(),
                    SoundboardMode::AlwaysEnabled,
                ),
                (
                    "933537038735659089".to_owned(),
                    SoundboardMode::Managed { threshold: 8 },
                ),
            ]
            .into_iter()
            .collect(),
        );

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "1002692827312037910": { "mode": "always_enabled" },
                "933537038735659089": { "mode": "managed", "threshold": 8 }
            })
        );
        assert_eq!(
            serde_json::from_value::<SoundboardManagerConfig>(json).unwrap(),
            config
        );
    }

    #[test]
    fn from_form_parses_each_mode() {
        assert_eq!(
            SoundboardMode::from_form("always_enabled", None),
            Ok(SoundboardMode::AlwaysEnabled)
        );
        assert_eq!(
            SoundboardMode::from_form("always_disabled", None),
            Ok(SoundboardMode::AlwaysDisabled)
        );
        assert_eq!(
            SoundboardMode::from_form("managed", None),
            Ok(SoundboardMode::Managed { threshold: 8 })
        );
        assert_eq!(
            SoundboardMode::from_form("managed", Some(4)),
            Ok(SoundboardMode::Managed { threshold: 4 })
        );
        assert!(SoundboardMode::from_form("bogus", None).is_err());
    }
}
