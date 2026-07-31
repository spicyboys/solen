use anyhow::Result;
use sea_orm::{EntityTrait, Set};
use serenity::all::User;

use crate::{
    models::birthdays,
    Context as PoiseContext,
};

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

    birthdays::Entity::insert(birthdays::ActiveModel {
        user_id: Set(user_id.clone()),
        month: Set(month),
        day: Set(day),
    })
    .exec(&ctx.data().db)
    .await?;

    ctx.say(format!("Registered birthday for <@{user_id}> as {month}/{day}"))
        .await?;

    Ok(())
}
