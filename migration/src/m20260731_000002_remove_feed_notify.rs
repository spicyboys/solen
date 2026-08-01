use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260712_205144_feed_subscription_notifications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("patch_notes")
                    .drop_column(Alias::new("notify"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("patch_notes")
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("notify"))
                            .array(ColumnType::String(Default::default()))
                            .not_null()
                            .default::<Vec<String>>(vec![]),
                    )
                    .to_owned(),
            )
            .await
    }
}
