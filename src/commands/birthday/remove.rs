use anyhow::Result;
use poise::serenity_prelude as serenity;
use sea_orm::{EntityTrait, ModelTrait};

use crate::{Context as PoiseContext, models::birthdays};
use serenity::all::User;

#[poise::command(slash_command)]
pub async fn remove(
    ctx: PoiseContext<'_>,
    #[description = "Member to remove the birthday for"] user: User,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = user.id.to_string();

    let birthday = birthdays::Entity::find_by_id(user_id.clone())
        .one(&ctx.data().db)
        .await?;

    match birthday {
        Some(model) => {
            model.delete(&ctx.data().db).await?;
            ctx.say(format!("Removed birthday for <@{user_id}>"))
                .await?;
        }
        None => {
            ctx.say(format!("No birthday registered for <@{user_id}>"))
                .await?;
        }
    }

    Ok(())
}
