use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{layout, page},
    view::{attributes, view},
};

use crate::{
    components::table::{table, table_body, table_cell, table_head, table_header, table_row},
    constants,
    models::birthdays,
    web::WebContext,
};

#[layout]
async fn layout(slot: Result) -> Result {
    view! {
        <header
            class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6"
        >
            <h1 class="text-lg font-semibold">"Birthdays"</h1>
        </header>
        <main class="flex-1 p-4 lg:p-6">(slot?)</main>
    }
}

#[page]
pub(crate) async fn index(cx: &Cx) -> Result {
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();

    let records = birthdays::Model::all().exec(&mut db).await?;
    let guild_members = ctx
        .http
        .get_guild_members(constants::GUILD_ID, None, None)
        .await?;

    view! {
        <div class="flex flex-col gap-4">
            <div class="rounded-md border border-border">
                table(
                    table_header(
                        table_row(
                            table_head("Name")
                            table_head("Birthday")
                        )
                    )
                    table_body(
                        for record in records {
                            table_row(
                                table_cell(
                                    attrs: attributes! { class="font-medium" },
                                    (guild_members
                                        .iter()
                                        .find(|m| m.user.id.to_string() == record.user_id)
                                        .map(|u| u.display_name())
                                        .unwrap_or_else(|| record.user_id.as_ref()))
                                )
                                table_cell(
                                    (format!(
                                        "{:02}/{:02}", record.month, record.day
                                    ))
                                )
                            )
                        }
                    )
                )
            </div>
        </div>
    }
}
