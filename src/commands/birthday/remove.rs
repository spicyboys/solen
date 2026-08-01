use anyhow::Result;
use poise::serenity_prelude as serenity;

use crate::{Context as PoiseContext, models::birthdays};
use serenity::all::User;

#[poise::command(slash_command)]
pub async fn remove(
    ctx: PoiseContext<'_>,
    #[description = "Member to remove the birthday for"] user: User,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = user.id.to_string();

    let mut db = ctx.data().db.clone();
    let birthday = birthdays::Model::filter_by_user_id(user_id.clone())
        .first()
        .exec(&mut db)
        .await?;

    match birthday {
        Some(model) => {
            model.delete().exec(&mut db).await?;
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
