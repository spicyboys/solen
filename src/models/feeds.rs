use toasty::Model;

#[derive(Clone, Debug, PartialEq, Eq, Model)]
#[table = "feeds"]
pub struct Model {
    #[key]
    pub id: i32,
    pub channel_id: String,
    pub feed: String,
    pub latest_post: String,
}
