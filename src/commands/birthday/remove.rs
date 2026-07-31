use anyhow::Result;
use sea_orm::{EntityTrait, ModelTrait};
use serenity::all::User;

use crate::{
    models::birthdays,
    Context as PoiseContext,
};

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
            ctx.say(format!(
                "No birthday registered for <@{user_id}>"
            ))
            .await?;
        }
    }

    Ok(())
}
