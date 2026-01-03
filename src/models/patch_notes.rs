use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "patch_notes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub feed: String,
    pub latest_post: String,
}

impl ActiveModelBehavior for ActiveModel {}