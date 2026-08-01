use toasty::Model;

#[derive(Clone, Debug, PartialEq, Eq, Model)]
#[table = "archived_soundboards"]
pub struct Model {
    #[key]
    pub sound_id: String,
    pub name: String,
    pub s3_key: String,
    pub original_uploader: Option<String>,
}
