//! `SeaORM` Entity for archived soundboards

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "archived_soundboards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sound_id: String,
    pub name: String,
    pub s3_key: String,
    pub original_uploader: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
