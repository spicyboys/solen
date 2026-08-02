mod birthdays;
mod feature_toggles;
mod soundboards;

use topcoat::{
    Result,
    context::{Cx, CxBuilder},
    icon::{icon, iconify::iconify_icon},
    router::{Body, Next, Response, error::redirect, layer, layout, uri},
    view::{attributes, view},
};

use crate::{
    components::sidebar::{
        sidebar, sidebar_content, sidebar_footer, sidebar_group, sidebar_group_content,
        sidebar_group_label, sidebar_header, sidebar_inset, sidebar_menu, sidebar_menu_button,
        sidebar_menu_item,
    },
    web::auth,
};

topcoat::router::segment!(kind = Group);

#[layer]
async fn require_auth(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    if auth::current_user_id(cx).await.is_some() {
        let response = next.run(cx, body).await?;
        Ok(response)
    } else {
        Err(redirect("/login").into())
    }
}

#[layout]
async fn root_layout(cx: &Cx, slot: Result) -> Result {
    let uri = uri(cx);
    return view! {
        <div class="flex min-h-dvh">
            sidebar(
                sidebar_header(
                    <a href="/" class="flex items-center gap-2 font-semibold">
                        "Solen"
                    </a>
                )
                sidebar_content(
                    sidebar_group(
                        sidebar_group_label("General")
                        sidebar_group_content(
                            sidebar_menu(
                                sidebar_menu_item(
                                    sidebar_menu_button(
                                        attrs: attributes! { href="/soundboards" },
                                        is_active: uri.path() == "/soundboards",
                                        icon(data: iconify_icon!("feather:play"))
                                        "Soundboards"
                                    )
                                )
                                sidebar_menu_item(
                                    sidebar_menu_button(
                                        attrs: attributes! { href="/birthdays" },
                                        is_active: uri.path() == "/birthdays",
                                        icon(data: iconify_icon!("feather:calendar"))
                                        "Birthdays"
                                    )
                                )
                                sidebar_menu_item(
                                    sidebar_menu_button(
                                        attrs: attributes! { href="/feature-toggles" },
                                        is_active: uri.path() == "/feature-toggles",
                                        icon(data: iconify_icon!("feather:toggle-left"))
                                        "Feature toggles"
                                    )
                                )
                            )
                        )
                    )
                )
                sidebar_footer(
                    sidebar_menu(
                        sidebar_menu_item(
                            sidebar_menu_button(
                                attrs: attributes! { href="/logout" },
                                icon(data: iconify_icon!("feather:log-out"))
                                "Log out"
                            )
                        )
                    )
                )
            )
            sidebar_inset((slot?))
        </div>
    };
}
