use std::time::Duration;

use anyhow::{Result, anyhow};
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    Context as PoiseContext, SPICY_BOYS,
    models::patch_notes,
    roles::{BOSSY_BOYS, MID_LEVEL_MANAGEMENT_BOYS},
};

pub async fn unsubscribe(
    ctx: PoiseContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !(ctx
        .author()
        .has_role(&ctx.http(), SPICY_BOYS, BOSSY_BOYS)
        .await?
        || ctx
            .author()
            .has_role(&ctx.http(), SPICY_BOYS, MID_LEVEL_MANAGEMENT_BOYS)
            .await?)
    {
        return Err(anyhow!("User is not an admin").into());
    }

    let channel_id = ctx.channel_id().get() as i64;
    let subscriptions = patch_notes::Entity::find()
        .filter(patch_notes::Column::ChannelId.eq(channel_id))
        .all(&ctx.data().db)
        .await?;

    if subscriptions.is_empty() {
        ctx.say("This channel isn't subscribed to any feeds.").await?;
        return Ok(());
    }

    if subscriptions.len() == 1 {
        let subscription = &subscriptions[0];
        patch_notes::Entity::delete_by_id(subscription.id)
            .exec(&ctx.data().db)
            .await?;
        ctx.say(format!(
            "Unsubscribed this channel from {}",
            subscription.feed
        ))
        .await?;
        return Ok(());
    }

    let ctx_id = ctx.id();
    let select_id = format!("{ctx_id}select");
    let confirm_id = format!("{ctx_id}confirm");
    let all_id = format!("{ctx_id}all");
    let cancel_id = format!("{ctx_id}cancel");

    let options: Vec<serenity::CreateSelectMenuOption> = subscriptions
        .iter()
        .map(|s| serenity::CreateSelectMenuOption::new(&s.feed, s.id.to_string()))
        .collect();

    let select_menu = serenity::CreateActionRow::SelectMenu(
        serenity::CreateSelectMenu::new(
            &select_id,
            serenity::CreateSelectMenuKind::String { options },
        )
        .placeholder("Choose a feed to unsubscribe from")
        .min_values(1)
        .max_values(1),
    );

    let buttons = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(&confirm_id)
            .label("Unsubscribe Selected")
            .style(serenity::ButtonStyle::Primary),
        serenity::CreateButton::new(&all_id)
            .label("Unsubscribe All")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new(&cancel_id)
            .label("Cancel")
            .style(serenity::ButtonStyle::Secondary),
    ]);

    let embed = serenity::CreateEmbed::new()
        .title("Unsubscribe from feeds")
        .description(
            subscriptions
                .iter()
                .map(|s| format!("- {} ({})", s.feed, s.feed_type))
                .collect::<Vec<_>>()
                .join("\n"),
        );

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .components(vec![select_menu, buttons]),
    )
    .await?;

    let mut selected_id: Option<i32> = None;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(120))
        .await
    {
        if press.data.custom_id == select_id {
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &press.data.kind
            {
                selected_id = values.first().and_then(|v| v.parse().ok());
            }
            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::Acknowledge,
                )
                .await?;
            continue;
        }

        if press.data.custom_id == confirm_id {
            let Some(id) = selected_id else {
                press
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("Select a feed first.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                continue;
            };

            let feed = subscriptions
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.feed.clone())
                .unwrap_or_default();
            patch_notes::Entity::delete_by_id(id)
                .exec(&ctx.data().db)
                .await?;

            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format!("Unsubscribed this channel from {feed}"))
                            .embeds(vec![])
                            .components(vec![]),
                    ),
                )
                .await?;
            break;
        }

        if press.data.custom_id == all_id {
            patch_notes::Entity::delete_many()
                .filter(patch_notes::Column::ChannelId.eq(channel_id))
                .exec(&ctx.data().db)
                .await?;

            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("Unsubscribed this channel from all feeds.")
                            .embeds(vec![])
                            .components(vec![]),
                    ),
                )
                .await?;
            break;
        }

        if press.data.custom_id == cancel_id {
            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("Unsubscribe cancelled.")
                            .embeds(vec![])
                            .components(vec![]),
                    ),
                )
                .await?;
            break;
        }
    }

    Ok(())
}
