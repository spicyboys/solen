
use std::{cell::LazyCell, collections::HashMap};

use chrono::DateTime;
use html2md::TagHandlerFactory;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serenity::all::{ChannelId, CreateMessage};
use async_trait::async_trait;
use anyhow::{Result, bail};

use crate::{jobs::{JobContext, patch_notes::PatchNotesJob}, models::patch_notes};

#[async_trait]
pub trait RssPatchNote: Sync {
    const FEED_URL: &'static str;
    const CHANNEL_ID: ChannelId;

    async fn parse_feed_item(&self, item: &rss::Item) -> Result<CreateMessage>;
}

#[async_trait]
impl<T: RssPatchNote + Send + Sync> PatchNotesJob for T {
    async fn fetch_latest_post(&self, ctx: JobContext) -> Result<()> {
        let content = reqwest::get(Self::FEED_URL).await?.bytes().await?;

        let channel = rss::Channel::read_from(&content[..])?;

        let mut items: Vec<_> = channel.items().iter().collect();

        if items.is_empty() {
            bail!("No items found in RSS feed");
        }

        items.sort_by(|a, b| {
            let date_a = a.pub_date().and_then(|d| DateTime::parse_from_rfc2822(d).ok());
            let date_b = b.pub_date().and_then(|d| DateTime::parse_from_rfc2822(d).ok());
            date_b.cmp(&date_a) // Descending order (most recent first)
        });

        let model = patch_notes::Entity::find()
            .filter(patch_notes::Column::Feed.eq(Self::FEED_URL))
            .one(&ctx.db)
            .await?;

        // Determine which posts are new
        let posts = if let Some(model) = &model {
            let pos = items.iter().position(|i| {
                if let Some(guid) = i.guid() {
                    guid.value() == model.latest_post
                } else {
                    false
                }
            });

            if let Some(pos) = pos {
                if pos == 0 {
                    // No new posts
                    return Ok(());
                }

                &items[0..pos]
            } else {
                // All posts are new or the latest post is not found, only send the most recent one
                &items[0..1]
            }
        } else {
            // No previous record, only send the most recent one
            &items[0..1]
        };

        for item in posts.iter().rev() {
            Self::CHANNEL_ID.send_message(&ctx.discord_http, self.parse_feed_item(item).await?).await?;
        }

        if let Some(latest_post_id) = items.first().and_then(|i| i.guid()).map(|g| g.value().to_string()) {
            if let Some(model) = model {
                let mut model: patch_notes::ActiveModel = model.into();
                model.latest_post = sea_orm::Set(latest_post_id);
                model.update(&ctx.db).await?;
            } else {
                let new_model = patch_notes::ActiveModel {
                    feed: sea_orm::Set(Self::FEED_URL.to_string()),
                    latest_post: sea_orm::Set(latest_post_id),
                    ..Default::default()
                };
                patch_notes::Entity::insert(new_model).exec(&ctx.db).await?;
            }
        }
        Ok(())
    }
}

struct DummyHandlerFactory;

impl TagHandlerFactory for DummyHandlerFactory {
    fn instantiate(&self) -> Box<dyn html2md::TagHandler> {
        Box::new(html2md::dummy::DummyHandler::default())
    }
}

const HTML2MD_TAG_FACTORIES: LazyCell<HashMap<String, Box<dyn TagHandlerFactory>>> = LazyCell::new(|| {
    let mut tag_factory: HashMap<String, Box<dyn TagHandlerFactory>> = HashMap::new();
    tag_factory.insert(String::from("img"), Box::new(DummyHandlerFactory));
    tag_factory
});

pub fn parse_html(html: &str) -> String {
    html2md::parse_html_custom(html, &HTML2MD_TAG_FACTORIES)
}