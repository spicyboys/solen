use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("archived_soundboards")
                    .if_not_exists()
                    .col(string("sound_id").primary_key())
                    .col(string("name"))
                    .col(string("s3_key"))
                    .col(string("original_uploader").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("archived_soundboards").to_owned())
            .await
    }
}
