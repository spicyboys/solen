use anyhow::Result;
use chrono::Datelike;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use chrono_tz::US::Central;
use serenity::all::{CreateMessage, MessageBuilder, UserId};
use crate::{channels, jobs::JobContext, models::birthdays};

pub async fn send_birthday_message(ctx: JobContext) -> Result<()> {
    let today = chrono::Utc::now().with_timezone(&Central).date_naive();
    println!("Checking birthdays for {}-{}...", today.month(), today.day());
    let birthdays = birthdays::Entity::find()
        .filter(birthdays::Column::Day.eq(today.day() as i16))
        .filter(birthdays::Column::Month.eq(today.month() as i16))
        .all(&ctx.db)
        .await;

    println!("{:?}", birthdays);
    let birthdays = birthdays?;

    if birthdays.is_empty() {
        return Ok(());
    }

    for birthday in birthdays {
        let user_id = UserId::new(birthday.user_id.parse::<u64>()?);

        let content = MessageBuilder::new()
            .push("GIVE IT UP FOR ")
            .mention(&user_id)
            .build();

        channels::CONFIDENTIAL.send_message(&ctx.discord_http, CreateMessage::new().content(content)).await?;
    }
    Ok(())
}