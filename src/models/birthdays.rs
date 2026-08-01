use toasty::Model;

#[derive(Clone, Debug, PartialEq, Eq, Model)]
#[table = "birthdays"]
pub struct Model {
    #[key]
    pub user_id: String,
    pub month: i16,
    pub day: i16,
}
