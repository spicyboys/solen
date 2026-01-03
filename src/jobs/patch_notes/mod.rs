pub mod rss;

use serenity::all::{ChannelId, CreateEmbed, CreateMessage};
use crate::jobs::{JobContext, patch_notes::rss::RssPatchNote};
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait PatchNotesJob: Send + Sync {
    async fn fetch_latest_post(&self, ctx: JobContext) -> Result<()>;
}

struct DeadlockPatchNotes;

#[async_trait]
impl RssPatchNote for DeadlockPatchNotes {
    const FEED_URL: &'static str = "https://forums.playdeadlock.com/forums/changelog.10/~/index.rss";
    const CHANNEL_ID: ChannelId = crate::channels::DEADLOCK;

    async fn parse_feed_item(
        &self,
        item: &::rss::Item,
    ) -> Result<CreateMessage> {
        let mut embed = CreateEmbed::new();

        if let Some(title) = item.title() {
            embed = embed.title(title);
        }

        if let Some(link) = item.link() {
            embed = embed.url(link);
        }

        if let Some(description) = item.content() {
            embed = embed.description(html2md::parse_html(description));
        }

        Ok(CreateMessage::new().embed(embed))
    }
}

// Disabled for now - the feed is behind cloudflare.
#[allow(dead_code)]
struct VintageStoryPatchNotes;

#[async_trait]
impl RssPatchNote for VintageStoryPatchNotes {
    const FEED_URL: &'static str = "https://www.vintagestory.at/blog.html/news?rss=1";
    const CHANNEL_ID: ChannelId = crate::channels::VINTAGE_STORY;

    async fn parse_feed_item(
        &self,
        item: &::rss::Item,
    ) -> Result<CreateMessage> {
        let mut embed = CreateEmbed::new();

        if let Some(title) = item.title() {
            embed = embed.title(title);
        }

        if let Some(link) = item.link() {
            embed = embed.url(link);
        }

        if let Some(description) = item.description() {
            embed = embed.description(html2md::parse_html(description));
        }

        Ok(CreateMessage::new().embed(embed))
    }
}


pub const JOBS: [&dyn PatchNotesJob; 1] = [
    &DeadlockPatchNotes,
    // &VintageStoryPatchNotes,
];