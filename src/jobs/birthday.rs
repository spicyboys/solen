use std::str::FromStr;

use anyhow::Result;
use chrono::Datelike;
use chrono_tz::US::Central;
use poise::serenity_prelude as serenity;

use crate::{constants, jobs::JobContext, models::birthdays};
use serenity::all::{CreateMessage, MessageBuilder, UserId};

pub async fn send_birthday_message(ctx: JobContext) -> Result<()> {
    let today = chrono::Utc::now().with_timezone(&Central).date_naive();
    println!(
        "Checking birthdays for {}-{}...",
        today.month(),
        today.day()
    );

    let mut db = ctx.db.clone();
    let birthdays = birthdays::Model::all()
        .filter(birthdays::Model::fields().day().eq(today.day() as i16))
        .filter(birthdays::Model::fields().month().eq(today.month() as i16))
        .exec(&mut db)
        .await?;

    if birthdays.is_empty() {
        return Ok(());
    }

    for birthday in birthdays {
        let user_id = UserId::from_str(&birthday.user_id)?;

        let content = MessageBuilder::new()
            .push("GIVE IT UP FOR ")
            .mention(&user_id)
            .build();

        constants::channels::CONFIDENTIAL
            .send_message(&ctx.discord_http, CreateMessage::new().content(content))
            .await?;
    }
    Ok(())
}
