use poise::serenity_prelude::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateComponent, CreateContainer,
    CreateContainerComponent, CreateSection, CreateSectionAccessory, CreateSectionComponent,
    CreateTextDisplay, GuildId, Http, MessageFlags,
};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{
    Context as PoiseContext,
    constants::interactions::{SOUNDBOARD_PAGER_PREFIX, SOUNDBOARD_RESTORE_PREFIX},
    models::archived_soundboards,
};

const PAGE_SIZE: usize = 10;

#[poise::command(slash_command)]
pub async fn list(ctx: PoiseContext<'_>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.say("This command must be run in a guild").await?;
            return Ok(());
        }
    };

    let Some(components) = build_list_components(&ctx.data().db, ctx.http(), guild_id, 0).await?
    else {
        ctx.say("No archived soundboards").await?;
        return Ok(());
    };

    ctx.send(
        poise::CreateReply::default()
            .components(components)
            .flags(MessageFlags::IS_COMPONENTS_V2),
    )
    .await?;

    Ok(())
}

pub async fn build_list_components(
    db: &DatabaseConnection,
    http: &Http,
    guild_id: GuildId,
    page: usize,
) -> Result<Option<Vec<CreateComponent<'static>>>, Box<dyn std::error::Error + Send + Sync>> {
    let records = archived_soundboards::Entity::find().all(db).await?;
    if records.is_empty() {
        return Ok(None);
    }

    let installed_ids: std::collections::HashSet<String> = http
        .get_guild_soundboards(guild_id)
        .await?
        .into_iter()
        .map(|sb| sb.id.to_string())
        .collect();

    let total_pages = records.len().div_ceil(PAGE_SIZE);
    let page = page.min(total_pages - 1);
    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(records.len());

    let list_items: Vec<CreateContainerComponent<'static>> = records[start..end]
        .iter()
        .map(|r| {
            CreateContainerComponent::Section(CreateSection::new(
                vec![CreateSectionComponent::TextDisplay(CreateTextDisplay::new(
                    r.name.clone(),
                ))],
                CreateSectionAccessory::Button(
                    CreateButton::new(format!("{SOUNDBOARD_RESTORE_PREFIX}{}", r.sound_id))
                        .label("Restore")
                        .disabled(installed_ids.contains(&r.sound_id)),
                ),
            ))
        })
        .collect();

    let mut components: Vec<CreateComponent<'static>> =
        vec![CreateComponent::Container(CreateContainer::new(list_items))];

    if total_pages > 1 {
        let prev = CreateButton::new(format!("{SOUNDBOARD_PAGER_PREFIX}prev:{page}"))
            .label("◀")
            .style(ButtonStyle::Secondary)
            .disabled(page == 0);
        let page_indicator = CreateButton::new(format!("{SOUNDBOARD_PAGER_PREFIX}page:{page}"))
            .label(format!("Page {}/{}", page + 1, total_pages))
            .style(ButtonStyle::Secondary)
            .disabled(true);
        let next = CreateButton::new(format!("{SOUNDBOARD_PAGER_PREFIX}next:{page}"))
            .label("▶")
            .style(ButtonStyle::Secondary)
            .disabled(page + 1 >= total_pages);

        components.push(CreateComponent::ActionRow(CreateActionRow::buttons(vec![
            prev,
            page_indicator,
            next,
        ])));
    }

    Ok(Some(components))
}
