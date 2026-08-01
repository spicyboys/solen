use anyhow::Result;
use poise::serenity_prelude as serenity;

use crate::{Context as PoiseContext, models::birthdays};
use serenity::all::User;

#[poise::command(slash_command)]
pub async fn register(
    ctx: PoiseContext<'_>,
    #[description = "Member to register the birthday for"] user: User,
    #[description = "Month (1-12)"] month: i16,
    #[description = "Day (1-31)"] day: i16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !(1..=12).contains(&month) {
        ctx.say("Month must be between 1 and 12.").await?;
        return Ok(());
    }
    if !(1..=31).contains(&day) {
        ctx.say("Day must be between 1 and 31.").await?;
        return Ok(());
    }

    let user_id = user.id.to_string();
    let mut db = ctx.data().db.clone();

    toasty::create!(birthdays::Model {
        user_id: user_id.clone(),
        month,
        day,
    })
    .exec(&mut db)
    .await?;

    ctx.say(format!(
        "Registered birthday for <@{user_id}> as {month}/{day}"
    ))
    .await?;

    Ok(())
}
