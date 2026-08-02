use topcoat::{
    Result,
    icon::{icon, iconify::iconify_icon},
    view::{Attributes, View, class, component, view},
};

use crate::components::button::{ButtonSize, ButtonVariant, button_variants};

/// The classes for the [`sidebar`] container.
///
/// On mobile the sidebar is a fixed drawer that slides in from the left: it
/// rests off-canvas (`-translate-x-full`) and slides in when a `data-open`
/// attribute with the value `"true"` is bound to it. On medium screens and up
/// it becomes a sticky, viewport-height column (`md:sticky md:h-dvh`) that
/// stays put while the page scrolls; a `data-collapsed` attribute bound with
/// the value `"true"` narrows it to an icon rail (`md:data-[collapsed=true]:w-12`).
const SIDEBAR: &str = "fixed inset-y-0 left-0 z-40 group flex w-72 flex-col border-r \
    border-border bg-background transition-[transform,width] duration-200 ease-linear \
    -translate-x-full data-[open=true]:translate-x-0 \
    md:sticky md:top-0 md:bottom-auto md:h-dvh md:w-72 md:translate-x-0 \
    md:data-[collapsed=true]:w-12";

/// The classes for a [`sidebar_header`]: a hairline-ruled brand strip, sized
/// to match the site header it sits above.
const SIDEBAR_HEADER: &str = "flex h-14 shrink-0 items-center gap-2 border-b \
    border-border px-4 lg:h-[60px] lg:px-6";

/// The classes for a [`sidebar_content`]: the scrollable middle of the sidebar
/// between the header and footer.
const SIDEBAR_CONTENT: &str = "flex flex-1 flex-col gap-2 overflow-y-auto p-2";

/// The classes for a [`sidebar_group`]: a labelled section of menu items.
const SIDEBAR_GROUP: &str = "flex flex-col gap-1";

/// The classes for a [`sidebar_group_label`], naming its group's items.
const SIDEBAR_GROUP_LABEL: &str = "px-2 py-1.5 text-xs font-medium text-muted-foreground \
    md:group-data-[collapsed=true]:hidden";

/// The classes for a [`sidebar_group_content`], holding the group's menus.
const SIDEBAR_GROUP_CONTENT: &str = "flex flex-col gap-1";

/// The classes for a [`sidebar_menu`]: a column of items with no bullets.
const SIDEBAR_MENU: &str = "flex w-full list-none flex-col gap-1";

/// The classes for a [`sidebar_menu_item`], one row of a [`sidebar_menu`].
const SIDEBAR_MENU_ITEM: &str = "relative";

/// The classes for a [`sidebar_menu_button`].
///
/// A padded row that tints on hover and focus; an active item is filled with
/// the primary color so the current section is unmistakable. Any icon inside
/// is forced to a consistent `1rem` square unless it already carries an
/// explicit size class.
const SIDEBAR_MENU_BUTTON: &str = "flex w-full items-center gap-2 rounded-lg px-3 \
    py-2 text-sm font-medium whitespace-nowrap transition-colors outline-none \
    select-none hover:bg-foreground/5 focus-visible:bg-foreground/5 \
    focus-visible:ring-2 focus-visible:ring-ring \
    data-active:bg-primary data-active:text-primary-foreground \
    data-active:shadow-xs data-active:hover:bg-primary/90 \
    [&_svg:not([class*='size-'])]:size-4 \
    md:group-data-[collapsed=true]:justify-center md:group-data-[collapsed=true]:px-0";

/// The classes for a [`sidebar_footer`], pinned to the bottom of the sidebar.
const SIDEBAR_FOOTER: &str = "mt-auto p-2";

/// The classes for a [`sidebar_trigger`], a ghost icon button that toggles
/// the mobile drawer.
const SIDEBAR_TRIGGER: &str = "shrink-0";

/// The classes for a [`sidebar_inset`], wrapping the content beside the
/// sidebar.
const SIDEBAR_INSET: &str = "flex flex-1 flex-col";

/// A sidebar: the application's persistent navigation rail.
///
/// On desktop the sidebar is an always-visible column at the left edge of the
/// layout. On smaller screens it is a drawer: off-canvas until the host binds
/// a `data-open` attribute with the value `"true"` to it (with a `:data-open`
/// bind attribute), sliding in over a backdrop. The `attrs` (such as the
/// `:data-open` binding, `class`, or `aria` attributes) are forwarded to the
/// underlying `<aside>`; a `class` among them is appended to the computed
/// classes. Child nodes become the sidebar's sections, in order: a
/// [`sidebar_header`], a [`sidebar_content`] of [`sidebar_group`]s, and a
/// [`sidebar_footer`].
///
/// ```ignore
/// sidebar(
///     attrs: attributes! {
///         :data-open=$(if open.get() { "true" } else { "false" })
///     },
///     sidebar_header(
///         <a href="/" class="flex items-center gap-2 font-semibold">
///             "Solen"
///         </a>
///     )
///     sidebar_content(
///         sidebar_group(
///             sidebar_menu(
///                 sidebar_menu_item(
///                     sidebar_menu_button(
///                         attrs: attributes! { href="/" data-active="" },
///                         "Soundboards"
///                     )
///                 )
///             )
///         )
///     )
/// )
/// ```
#[component]
pub async fn sidebar(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <aside class=(class!(SIDEBAR, attrs.remove("class"))) (attrs)>(child)</aside>
    }
}

/// The top strip of a [`sidebar`], for branding.
#[component]
pub async fn sidebar_header(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(class!(SIDEBAR_HEADER, attrs.remove("class"))) (attrs)>(child)</div>
    }
}

/// The scrollable middle of a [`sidebar`], between its header and footer.
#[component]
pub async fn sidebar_content(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(class!(SIDEBAR_CONTENT, attrs.remove("class"))) (attrs)>
            (child)
        </div>
    }
}

/// A labelled section of a [`sidebar_content`], grouping related menus.
#[component]
pub async fn sidebar_group(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(class!(SIDEBAR_GROUP, attrs.remove("class"))) (attrs)>(child)</div>
    }
}

/// The label of a [`sidebar_group`], naming the items it contains.
#[component]
pub async fn sidebar_group_label(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <p class=(class!(SIDEBAR_GROUP_LABEL, attrs.remove("class"))) (attrs)>
            (child)
        </p>
    }
}

/// The content of a [`sidebar_group`], holding its menus.
#[component]
pub async fn sidebar_group_content(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <div class=(class!(SIDEBAR_GROUP_CONTENT, attrs.remove("class"))) (attrs)>
            (child)
        </div>
    }
}

/// A column of [`sidebar_menu_item`]s within a [`sidebar_group`].
#[component]
pub async fn sidebar_menu(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <ul class=(class!(SIDEBAR_MENU, attrs.remove("class"))) (attrs)>(child)</ul>
    }
}

/// One row of a [`sidebar_menu`], holding a [`sidebar_menu_button`].
#[component]
pub async fn sidebar_menu_item(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <li class=(class!(SIDEBAR_MENU_ITEM, attrs.remove("class"))) (attrs)>
            (child)
        </li>
    }
}

/// A navigation link in a [`sidebar_menu`], rendered as an `<a>`.
///
/// The `attrs` (such as `href` or `data-active`) are forwarded to the
/// underlying `<a>`; a `class` among them is appended to the computed classes.
/// Child nodes become the link's label, typically an icon and a word.
#[component]
pub async fn sidebar_menu_button(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <a class=(class!(SIDEBAR_MENU_BUTTON, attrs.remove("class"))) (attrs)>
            (child)
        </a>
    }
}

/// The bottom strip of a [`sidebar`], for account actions and the like.
#[component]
pub async fn sidebar_footer(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(class!(SIDEBAR_FOOTER, attrs.remove("class"))) (attrs)>(child)</div>
    }
}

/// A button that toggles the mobile [`sidebar`] drawer.
///
/// The trigger itself carries no behavior: the host passes an `@click` handler
/// in the `attrs` that flips the signal driving the drawer's `data-open`
/// binding. It is styled as a ghost icon button, like the theme's other icon
/// controls.
#[component]
pub async fn sidebar_trigger(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <button
            class=(class!(
                button_variants(ButtonVariant::Ghost, ButtonSize::Icon), SIDEBAR_TRIGGER,
                attrs.remove("class"),
            ))
            (attrs)
        >
            icon(data: iconify_icon!("feather:menu"))
            (child)
        </button>
    }
}

/// The content column beside a [`sidebar`]: the site header and the page.
#[component]
pub async fn sidebar_inset(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(class!(SIDEBAR_INSET, attrs.remove("class"))) (attrs)>(child)</div>
    }
}

#[cfg(test)]
mod tests {
    use topcoat::{
        context::Cx,
        icon::{icon, iconify::iconify_icon},
        runtime::Event,
        view::{attributes, view},
    };

    use crate::components::separator::{SeparatorOrientation, separator};

    use super::*;

    #[tokio::test]
    async fn renders_sidebar_layout() {
        let cx = &Cx::default();
        let html = view! {
            cx => {
                signal mobile_open = false;
                signal collapsed = false;

                sidebar(
                    attrs: attributes! {
                        :data-open=$(if mobile_open.get() { "true" } else { "false" })
                        :data-collapsed=$(if collapsed.get() { "true" } else { "false" })
                    },
                    sidebar_header(
                        <a href="/" class="flex items-center gap-2 font-semibold md:group-data-[collapsed=true]:justify-center">
                            <span class="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                                icon(data: iconify_icon!("feather:music"))
                            </span>
                            <span class="md:group-data-[collapsed=true]:hidden">"Solen"</span>
                        </a>
                    )
                    sidebar_content(
                        sidebar_group(
                            sidebar_group_label("General")
                            sidebar_group_content(
                                sidebar_menu(
                                    sidebar_menu_item(
                                        sidebar_menu_button(
                                            attrs: attributes! { href="/" data-active="" },
                                            icon(data: iconify_icon!("feather:disc"))
                                            <span class="md:group-data-[collapsed=true]:hidden">"Soundboards"</span>
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
                                    <span class="md:group-data-[collapsed=true]:hidden">"Log out"</span>
                                )
                            )
                        )
                    )
                )
                sidebar_inset(
                    <header class="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4 lg:h-[60px] lg:px-6">
                        sidebar_trigger(
                            attrs: attributes! {
                                @click=$(|_e: Event| raw!(
                                    "if (window.matchMedia('(min-width: 768px)').matches) { ${collapsed}.toggle(); } else { ${mobile_open}.toggle(); }"
                                ))
                            }
                        )
                        separator(orientation: SeparatorOrientation::Vertical, attrs: attributes! { class="h-4" })
                        <h1 class="text-lg font-semibold">"Soundboards"</h1>
                    </header>
                    <main class="flex-1 p-4 lg:p-6">"main"</main>
                )
            }
        }
        .expect("build view")
        .render(&cx);

        println!("{html}");
        assert!(html.contains("data-topcoat-bind:data-open"), "missing data-open bind");
        assert!(
            html.contains("data-topcoat-bind:data-collapsed"),
            "missing data-collapsed bind"
        );
        assert!(html.contains("matchMedia"), "missing matchMedia handler");
        assert!(html.contains("md:data-[collapsed=true]:w-12"), "missing collapsed width class");
        assert!(
            html.contains("md:group-data-[collapsed=true]:hidden"),
            "missing group label hiding"
        );
    }
}
