use sea_orm_migration::prelude::{sea_query::extension::postgres::Type, *};
use sea_orm_migration::sea_orm::{ConnectionTrait, DbBackend};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260712_212423_patch_notes_to_feeds"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(Table::rename().table("patch_notes", "feeds").take())
            .await?;

        let db = manager.get_connection();
        if db.get_database_backend() == DbBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(FeedType::Enum)
                        .values([FeedType::Ntfy, FeedType::Rss])
                        .to_owned(),
                )
                .await?;

            db.execute_unprepared("ALTER TABLE feeds ALTER COLUMN feed_type DROP DEFAULT")
                .await?;

            db.execute_unprepared(
                "ALTER TABLE feeds ALTER COLUMN feed_type TYPE feed_type USING feed_type::feed_type",
            )
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        if db.get_database_backend() == DbBackend::Postgres {
            db.execute_unprepared(
                "ALTER TABLE feeds ALTER COLUMN feed_type TYPE varchar USING feed_type::text",
            )
            .await?;

            db.execute_unprepared("ALTER TABLE feeds ALTER COLUMN feed_type SET DEFAULT 'rss'")
                .await?;

            manager
                .drop_type(Type::drop().name(FeedType::Enum).to_owned())
                .await?;
        }

        manager
            .rename_table(Table::rename().table("feeds", "patch_notes").take())
            .await
    }
}

#[derive(DeriveIden)]
enum FeedType {
    #[sea_orm(iden = "feed_type")]
    Enum,
    #[sea_orm(iden = "ntfy")]
    Ntfy,
    #[sea_orm(iden = "rss")]
    Rss,
}
