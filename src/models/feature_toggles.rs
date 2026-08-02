use toasty::{Json, Model};

use crate::feature_toggles::FlagValue;

#[derive(Clone, Debug, PartialEq, Model)]
#[table = "feature_toggles"]
pub struct Model {
    #[key]
    pub key: String,
    #[column(type = jsonb)]
    pub value: Json<FlagValue>,
}
