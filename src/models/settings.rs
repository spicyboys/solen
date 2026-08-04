use toasty::{Json, Model};

#[derive(Clone, Debug, PartialEq, Model)]
#[table = "settings"]
pub struct Model {
    #[key]
    pub key: String,
    #[column(type = jsonb)]
    pub value: Json<serde_json::Value>,
}
