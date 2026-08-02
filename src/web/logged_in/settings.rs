use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::Form,
        error::{SeeOther, bad_request, see_other},
        layout, page, route,
    },
    view::{attributes, component, view},
};

use crate::{
    components::button::{ButtonSize, button},
    components::table::{table, table_body, table_cell, table_head, table_header, table_row},
    feature_toggles::FlagValue,
    models::feature_toggles,
    web::{WebContext, auth},
};

const INPUT: &str = "h-8 rounded-md border border-border bg-background px-2 text-sm";
const TEXTAREA: &str =
    "min-w-64 rounded-md border border-border bg-background p-2 font-mono text-xs";

#[layout]
async fn layout(slot: Result) -> Result {
    view! {
        <header
            class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6"
        >
            <h1 class="text-lg font-semibold">"Settings"</h1>
        </header>
        <main class="flex-1 p-4 lg:p-6">(slot?)</main>
    }
}

#[page]
pub(crate) async fn index(cx: &Cx) -> Result {
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();

    let records = feature_toggles::Model::all().exec(&mut db).await?;

    view! {
        <div class="flex flex-col gap-4">
            <div class="rounded-md border border-border">
                table(
                    table_header(
                        table_row(
                            table_head("Key")
                            table_head("Type")
                            table_head("Value")
                        )
                    )
                    table_body(
                        for record in records {
                            table_row(
                                table_cell(
                                    attrs: attributes! { class="font-medium" },
                                    (record.key.clone())
                                )
                                table_cell((record.value.type_name()))
                                table_cell(
                                    toggle_value_form(
                                        key: record.key.clone(),
                                        value: record.value.0.clone()
                                    )
                                )
                            )
                        }
                    )
                )
            </div>
        </div>
    }
}

/// An edit form for one toggle: a type-specific input and a save button.
#[component]
async fn toggle_value_form(key: String, value: FlagValue) -> Result {
    view! {
        <form
            method="post"
            action="/settings/update"
            class="flex items-center gap-2"
        >
            <input type="hidden" name="key" value=(key)>
            <input type="hidden" name="kind" value=(value.type_name())>
            value_input(value: value)
            button(size: ButtonSize::Sm, attrs: attributes! { type="submit" }, "Save")
        </form>
    }
}

/// The input that edits a toggle's value, matching its type.
#[component]
async fn value_input(value: FlagValue) -> Result {
    match value {
        FlagValue::Bool { value } => view! {
            <select name="value" class=(INPUT)>
                <option value="true" selected=(value)>"true"</option>
                <option value="false" selected=(!value)>"false"</option>
            </select>
        },
        FlagValue::Int { value } => view! {
            <input type="number" name="value" value=(value) class=(INPUT)>
        },
        FlagValue::Float { value } => view! {
            <input type="number" step="any" name="value" value=(value) class=(INPUT)>
        },
        FlagValue::String { value } => view! {
            <input type="text" name="value" value=(value) class=(INPUT)>
        },
        FlagValue::Object { value } => {
            let json = serde_json::to_string_pretty(&FlagValue::Object { value }.to_shorthand())
                .unwrap_or_default();
            view! { <textarea name="value" rows="4" class=(TEXTAREA)>(json)</textarea> }
        }
    }
}

#[derive(serde::Deserialize)]
struct UpdateFlagForm {
    key: String,
    kind: String,
    value: String,
}

#[route(POST "/settings/update")]
pub(crate) async fn update(cx: &Cx, form: Form<UpdateFlagForm>) -> Result<SeeOther> {
    auth::require_auth(cx).await?;
    let flag = FlagValue::from_form(&form.kind, &form.value).map_err(bad_request)?;
    let ctx = app_context::<WebContext>(cx);
    let mut db = ctx.data.db.clone();
    feature_toggles::Model::filter_by_key(form.key.clone())
        .update()
        .value(flag)
        .exec(&mut db)
        .await?;
    Ok(see_other("/settings"))
}
