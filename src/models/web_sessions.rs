use toasty::Model;

#[derive(Clone, Debug, PartialEq, Eq, Model)]
#[table = "web_sessions"]
pub struct Model {
    #[key]
    pub token_hash: String,
    pub user_id: String,
    pub expires_at: i64,
}
