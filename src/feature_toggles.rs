use std::collections::BTreeMap;

use async_trait::async_trait;
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, EvaluationResult,
    provider::{FeatureProvider, ProviderMetadata, ResolutionDetails},
    StructValue, Value,
};
use serde::{Deserialize, Serialize};
use toasty::Db;

use crate::models::feature_toggles;

/// The value of a feature toggle, mirroring OpenFeature's value types.
///
/// Stored in the `feature_toggles` table's JSONB column and round-tripped
/// through Serde by `toasty::Json<FlagValue>`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FlagValue {
    Bool { value: bool },
    Int { value: i64 },
    Float { value: f64 },
    String { value: String },
    Object { value: BTreeMap<String, FlagValue> },
}

impl FlagValue {
    /// The OpenFeature type name of this value, used as the toggle's variant.
    pub fn type_name(&self) -> &'static str {
        match self {
            FlagValue::Bool { .. } => "bool",
            FlagValue::Int { .. } => "int",
            FlagValue::Float { .. } => "float",
            FlagValue::String { .. } => "string",
            FlagValue::Object { .. } => "object",
        }
    }

    /// Parse a value typed `kind` from the string a web form submitted for it.
    pub fn from_form(kind: &str, value: &str) -> Result<Self, String> {
        match kind {
            "bool" => match value {
                "true" => Ok(FlagValue::Bool { value: true }),
                "false" => Ok(FlagValue::Bool { value: false }),
                other => Err(format!("invalid boolean value: {other:?}")),
            },
            "int" => value
                .parse::<i64>()
                .map(|value| FlagValue::Int { value })
                .map_err(|error| format!("invalid int value: {error}")),
            "float" => value
                .parse::<f64>()
                .map(|value| FlagValue::Float { value })
                .map_err(|error| format!("invalid float value: {error}")),
            "string" => Ok(FlagValue::String {
                value: value.to_owned(),
            }),
            "object" => serde_json::from_str::<serde_json::Value>(value)
                .map_err(|error| format!("invalid object JSON: {error}"))
                .and_then(|json| {
                    let object = json
                        .as_object()
                        .ok_or_else(|| "expected a JSON object".to_owned())?;
                    object
                        .iter()
                        .map(|(key, value)| {
                            FlagValue::from_json_value(value)
                                .map(|value| (key.clone(), value))
                        })
                        .collect::<Result<BTreeMap<_, _>, _>>()
                })
                .map(|value| FlagValue::Object { value }),
            other => Err(format!("unknown flag kind: {other:?}")),
        }
    }

    /// Infer a [`FlagValue`] from a raw JSON value, choosing the variant by the
    /// JSON type: booleans, strings, numbers (integers as `Int`, everything
    /// else as `Float`) and objects (recursively). `null` and arrays are
    /// rejected since they have no `FlagValue` equivalent.
    fn from_json_value(value: &serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::Bool(value) => Ok(FlagValue::Bool { value: *value }),
            serde_json::Value::String(value) => Ok(FlagValue::String { value: value.clone() }),
            serde_json::Value::Number(value) => {
                if let Some(value) = value.as_i64() {
                    Ok(FlagValue::Int { value })
                } else if let Some(value) = value.as_f64() {
                    Ok(FlagValue::Float { value })
                } else {
                    Err(format!("number out of range: {value}"))
                }
            }
            serde_json::Value::Object(value) => value
                .iter()
                .map(|(key, value)| {
                    FlagValue::from_json_value(value).map(|value| (key.clone(), value))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map(|value| FlagValue::Object { value }),
            other => Err(format!("unsupported JSON value: {other}")),
        }
    }

    /// The inverse of [`FlagValue::from_json_value`]: the plain JSON form of
    /// this value, with the type tag stripped. Used to display an object flag
    /// as the shorthand the web form accepts.
    pub fn to_shorthand(&self) -> serde_json::Value {
        match self {
            FlagValue::Bool { value } => serde_json::Value::Bool(*value),
            FlagValue::Int { value } => serde_json::json!(value),
            FlagValue::Float { value } => serde_json::json!(value),
            FlagValue::String { value } => serde_json::Value::String(value.clone()),
            FlagValue::Object { value } => serde_json::Value::Object(
                value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_shorthand()))
                    .collect(),
            ),
        }
    }

    /// Convert `self` to OpenFeature's [`Value`].
    fn to_value(&self) -> Value {
        match self {
            FlagValue::Bool { value } => Value::Bool(*value),
            FlagValue::Int { value } => Value::Int(*value),
            FlagValue::Float { value } => Value::Float(*value),
            FlagValue::String { value } => Value::String(value.clone()),
            FlagValue::Object { value } => Value::Struct(StructValue {
                fields: value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_value()))
                    .collect(),
            }),
        }
    }
}

/// An OpenFeature provider backed by the `feature_toggles` table.
///
/// Every flag is stored in its own row, keyed by the flag key; the JSONB
/// `value` column holds the flag's typed value. Flags are resolved statically
/// and do not consult the evaluation context.
pub struct FeatureToggleProvider {
    db: Db,
    metadata: ProviderMetadata,
}

impl FeatureToggleProvider {
    /// Create a provider that reads toggles from `db`.
    pub fn new(db: Db) -> Self {
        Self {
            db,
            metadata: ProviderMetadata::new("solen_feature_toggles"),
        }
    }

    /// Look up the stored value of the flag `flag_key`.
    async fn resolve_flag(&self, flag_key: &str) -> EvaluationResult<FlagValue> {
        let mut db = self.db.clone();
        feature_toggles::Model::filter_by_key(flag_key.to_owned())
            .first()
            .exec(&mut db)
            .await
            .map_err(|error| {
                EvaluationError::builder()
                    .code(EvaluationErrorCode::General(error.to_string()))
                    .message(format!("failed to look up flag {flag_key:?}: {error}"))
                    .build()
            })?
            .map(|record| record.value.0)
            .ok_or_else(|| {
                EvaluationError::builder()
                    .code(EvaluationErrorCode::FlagNotFound)
                    .message(format!("flag {flag_key:?} not found"))
                    .build()
            })
    }
}

/// Build a `TYPE_MISMATCH` error for a flag resolved as `found` when a value of
/// type `expected` was requested.
fn type_mismatch(flag_key: &str, found: &FlagValue, expected: &str) -> EvaluationError {
    EvaluationError::builder()
        .code(EvaluationErrorCode::TypeMismatch)
        .message(format!(
            "flag {flag_key:?} is of type {}, not {expected}",
            found.type_name()
        ))
        .build()
}

#[async_trait]
impl FeatureProvider for FeatureToggleProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        match self.resolve_flag(flag_key).await? {
            FlagValue::Bool { value } => Ok(ResolutionDetails::builder()
                .value(value)
                .variant("bool")
                .reason(EvaluationReason::Static)
                .build()),
            found => Err(type_mismatch(flag_key, &found, "bool")),
        }
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        match self.resolve_flag(flag_key).await? {
            FlagValue::Int { value } => Ok(ResolutionDetails::builder()
                .value(value)
                .variant("int")
                .reason(EvaluationReason::Static)
                .build()),
            found => Err(type_mismatch(flag_key, &found, "int")),
        }
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        match self.resolve_flag(flag_key).await? {
            FlagValue::Float { value } => Ok(ResolutionDetails::builder()
                .value(value)
                .variant("float")
                .reason(EvaluationReason::Static)
                .build()),
            found => Err(type_mismatch(flag_key, &found, "float")),
        }
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        match self.resolve_flag(flag_key).await? {
            FlagValue::String { value } => Ok(ResolutionDetails::builder()
                .value(value)
                .variant("string")
                .reason(EvaluationReason::Static)
                .build()),
            found => Err(type_mismatch(flag_key, &found, "string")),
        }
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        _evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        match self.resolve_flag(flag_key).await? {
            FlagValue::Object { value } => {
                let fields = value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_value()))
                    .collect();
                Ok(ResolutionDetails::builder()
                    .value(StructValue { fields })
                    .variant("object")
                    .reason(EvaluationReason::Static)
                    .build())
            }
            found => Err(type_mismatch(flag_key, &found, "object")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::FlagValue;

    #[test]
    fn round_trips_through_shorthand() {
        let map = BTreeMap::from([
            ("enabled".to_owned(), FlagValue::Bool { value: true }),
            ("retries".to_owned(), FlagValue::Int { value: 3 }),
        ]);
        let flag = FlagValue::Object { value: map.clone() };
        assert_eq!(
            FlagValue::from_form("object", &flag.to_shorthand().to_string()),
            Ok(flag)
        );
    }

    #[test]
    fn object_from_form_matches_the_ui_example() {
        let input = r#"{
            "1002692827312037910": {
                "mode": "always_enabled"
            },
            "933537038735659089": {
                "mode": "managed",
                "threshold": 8
            }
        }"#;
        let expected = FlagValue::Object {
            value: BTreeMap::from([
                (
                    "1002692827312037910".to_owned(),
                    FlagValue::Object {
                        value: BTreeMap::from([(
                            "mode".to_owned(),
                            FlagValue::String {
                                value: "always_enabled".to_owned(),
                            },
                        )]),
                    },
                ),
                (
                    "933537038735659089".to_owned(),
                    FlagValue::Object {
                        value: BTreeMap::from([
                            (
                                "mode".to_owned(),
                                FlagValue::String {
                                    value: "managed".to_owned(),
                                },
                            ),
                            ("threshold".to_owned(), FlagValue::Int { value: 8 }),
                        ]),
                    },
                ),
            ]),
        };
        assert_eq!(FlagValue::from_form("object", input), Ok(expected));
    }

    #[test]
    fn object_from_form_infers_scalar_types() {
        let input = r#"{"s":"8","b":true,"i":8,"f":8.0}"#;
        let expected = FlagValue::Object {
            value: BTreeMap::from([
                ("s".to_owned(), FlagValue::String { value: "8".to_owned() }),
                ("b".to_owned(), FlagValue::Bool { value: true }),
                ("i".to_owned(), FlagValue::Int { value: 8 }),
                ("f".to_owned(), FlagValue::Float { value: 8.0 }),
            ]),
        };
        assert_eq!(FlagValue::from_form("object", input), Ok(expected));
    }

    #[test]
    fn object_from_form_rejects_unsupported_json() {
        assert!(FlagValue::from_form("object", "not json").is_err());
        assert!(FlagValue::from_form("object", "[1, 2]").is_err());
        assert!(FlagValue::from_form("object", "null").is_err());
        assert!(FlagValue::from_form("object", "\"just a string\"").is_err());
    }

    #[test]
    fn from_form_parses_each_kind() {
        assert_eq!(
            FlagValue::from_form("bool", "true"),
            Ok(FlagValue::Bool { value: true })
        );
        assert_eq!(
            FlagValue::from_form("int", "-42"),
            Ok(FlagValue::Int { value: -42 })
        );
        assert_eq!(
            FlagValue::from_form("float", "1.5"),
            Ok(FlagValue::Float { value: 1.5 })
        );
        assert_eq!(
            FlagValue::from_form("string", "hello"),
            Ok(FlagValue::String {
                value: "hello".to_owned()
            })
        );
    }

    #[test]
    fn from_form_rejects_invalid_input() {
        assert!(FlagValue::from_form("bool", "maybe").is_err());
        assert!(FlagValue::from_form("int", "1.5").is_err());
        assert!(FlagValue::from_form("float", "not-a-number").is_err());
        assert!(FlagValue::from_form("object", "not json").is_err());
        assert!(FlagValue::from_form("unknown", "x").is_err());
    }
}
