pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20260104_014515_create_birthdays_table;
mod m20260629_000001_add_channel_id_to_patch_notes;
mod m20260712_205144_feed_subscription_notifications;
mod m20260712_212423_patch_notes_to_feeds;
mod m20260731_000001_create_archived_soundboards_table;
mod m20260731_000002_remove_feed_notify;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260104_014515_create_birthdays_table::Migration),
            Box::new(m20260629_000001_add_channel_id_to_patch_notes::Migration),
            Box::new(m20260712_205144_feed_subscription_notifications::Migration),
            Box::new(m20260712_212423_patch_notes_to_feeds::Migration),
            Box::new(m20260731_000001_create_archived_soundboards_table::Migration),
            Box::new(m20260731_000002_remove_feed_notify::Migration),
        ]
    }
}
