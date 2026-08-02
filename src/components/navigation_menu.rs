use topcoat::{
    Result,
    icon::{IconData, icon},
    view::{Attributes, View, attributes, class, component, svg::ViewBox, view},
};

/// The chevron-down icon that marks a [`navigation_menu_trigger`].
///
/// It is the feather `chevron-down` glyph, built by hand so the component needs
/// no iconify data downloaded at build time.
const CHEVRON_DOWN: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    "<polyline points=\"6 9 12 15 18 9\"></polyline>",
);

/// The classes for the [`navigation_menu`] container.
///
/// The menu is a horizontal bar of items, sized to its content and centered
/// against whatever space its parent gives it.
const NAVIGATION_MENU: &str = "relative flex max-w-max flex-1 items-center justify-center";

/// The classes for the [`navigation_menu_list`].
///
/// The list is a row of items with no bullets, tracking the `group` name so
/// descendants can react to open dropdowns with `group-open:` variants.
const NAVIGATION_MENU_LIST: &str =
    "group flex flex-1 list-none items-center justify-center";

/// The visual classes of a [`navigation_menu_trigger`] and of any element
/// styled like one, such as a [`navigation_menu_link`] given a trigger's look
/// via [`navigation_menu_trigger_style`].
///
/// Hover and focus tint the trigger like a ghost button, deriving the states
/// from the foreground color so they hold up in both color schemes without
/// `dark:` overrides. The `group-open:` fill keeps an open dropdown's trigger
/// visibly pressed.
const TRIGGER: &str = "inline-flex h-9 w-max items-center justify-center rounded-lg \
    px-2.5 py-1.5 text-sm font-medium whitespace-nowrap outline-none select-none \
    transition-colors hover:bg-foreground/5 focus-visible:bg-foreground/5 \
    focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none \
    disabled:opacity-50 group-open:bg-foreground/5";

/// The classes making a `<summary>` a plain clickable trigger: the default
/// disclosure marker is hidden and the cursor marks it as interactive.
const SUMMARY: &str = "cursor-pointer list-none [&::-webkit-details-marker]:hidden";

/// The classes for a [`navigation_menu_link`].
///
/// A link is a padded row that tints on hover and focus. Any icon inside is
/// forced to a consistent `1rem` square unless it already carries an explicit
/// size class.
const NAVIGATION_MENU_LINK: &str = "flex items-center gap-2 rounded-lg p-2 text-sm \
    whitespace-nowrap transition-colors outline-none select-none \
    hover:bg-foreground/5 focus-visible:bg-foreground/5 focus-visible:ring-2 \
    focus-visible:ring-ring data-active:bg-foreground/5 \
    [&_svg:not([class*='size-'])]:size-4";

/// The classes shared by the [`navigation_menu_content`] panel: a raised
/// surface styled like a card; `z-50` lifts it over later content. It sets its
/// own background and text color, so it reads the same on any ancestor.
const CONTENT: &str = "absolute top-full left-0 z-50 mt-1 min-w-40 rounded-lg border \
    border-border bg-background p-1 text-foreground shadow-sm";

/// Builds the class string that styles an element as a navigation trigger.
///
/// Use it to give a [`navigation_menu_link`] the look of a
/// [`navigation_menu_trigger`], for items that navigate rather than open a
/// dropdown:
///
/// ```ignore
/// view! {
///     navigation_menu_link(
///         attrs: attributes! {
///             href="/docs"
///             class=(navigation_menu_trigger_style())
///         },
///         "Docs"
///     )
/// }
/// ```
#[must_use]
pub fn navigation_menu_trigger_style() -> String {
    TRIGGER.to_owned()
}

/// A navigation menu: a horizontal bar of links and dropdowns.
///
/// A menu is a [`navigation_menu_list`] of [`navigation_menu_item`]s, each
/// holding either a [`navigation_menu_link`] or, wrapped in a
/// [`navigation_menu_dropdown`], a [`navigation_menu_trigger`] paired with a
/// [`navigation_menu_content`] panel. The `attrs` (such as `class` or `aria`
/// attributes) are forwarded to the underlying `<nav>`; a `class` among them is
/// appended to the computed classes. Child nodes become the menu's items.
///
/// ```ignore
/// view! {
///     navigation_menu(
///         navigation_menu_list(
///             navigation_menu_item(
///                 navigation_menu_link(
///                     attrs: attributes! { href="/" },
///                     "Soundboards"
///                 )
///             )
///             navigation_menu_item(
///                 navigation_menu_dropdown(
///                     navigation_menu_trigger("Account")
///                     navigation_menu_content(
///                         navigation_menu_link(
///                             attrs: attributes! { href="/logout" },
///                             "Log out"
///                         )
///                     )
///                 )
///             )
///         )
///     )
/// }
/// ```
#[component]
pub async fn navigation_menu(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <nav class=(class!(NAVIGATION_MENU, attrs.remove("class"))) (attrs)>
            (child)
        </nav>
    }
}

/// The row of items of a [`navigation_menu`].
#[component]
pub async fn navigation_menu_list(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <ul class=(class!(NAVIGATION_MENU_LIST, attrs.remove("class"))) (attrs)>
            (child)
        </ul>
    }
}

/// One entry of a [`navigation_menu_list`].
///
/// An item holds either a [`navigation_menu_link`] or a
/// [`navigation_menu_dropdown`] pairing a trigger with its content. The `attrs`
/// are forwarded to the underlying `<li>`; a `class` among them is appended to
/// the computed classes.
#[component]
pub async fn navigation_menu_item(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! { <li class=(class!("relative", attrs.remove("class"))) (attrs)>(child)</li> }
}

/// A link in a [`navigation_menu`], pointing to another page.
///
/// The `attrs` (such as `href`) are forwarded to the underlying `<a>`; a
/// `class` among them is appended to the computed classes. Child nodes become
/// the link's label.
#[component]
pub async fn navigation_menu_link(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <a class=(class!(NAVIGATION_MENU_LINK, attrs.remove("class"))) (attrs)>
            (child)
        </a>
    }
}

/// The trigger of a [`navigation_menu_dropdown`]: a `<summary>` that toggles
/// the dropdown's content panel.
///
/// A chevron is appended automatically; while the dropdown is open the
/// `group-open:` variant rotates it and tints the trigger. The `attrs` are
/// forwarded to the `<summary>`; a `class` among them is appended to the
/// computed classes. It must be the first child of its [`navigation_menu_dropdown`].
#[component]
pub async fn navigation_menu_trigger(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <summary class=(class!(SUMMARY, TRIGGER, attrs.remove("class"))) (attrs)>
            (child)
            icon(
                data: CHEVRON_DOWN,
                attrs: attributes! {
                    class =
                    "relative top-px ml-1 size-3 transition-colors group-open:rotate-180"
                }
            )
        </summary>
    }
}

/// The floating panel of a [`navigation_menu_dropdown`], holding links to the
/// menu item's destinations.
///
/// The panel drops directly below the trigger, aligned to its left edge. The
/// `attrs` are forwarded to the underlying `<div>`; a `class` among them is
/// appended to the computed classes.
#[component]
pub async fn navigation_menu_content(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! { <div class=(class!(CONTENT, attrs.remove("class"))) (attrs)>(child)</div> }
}

/// A dropdown in a [`navigation_menu`]: a [`navigation_menu_trigger`] and its
/// [`navigation_menu_content`] panel.
///
/// Built on `<details>`, so clicking the trigger toggles the content panel
/// without scripting. Clicking outside does not close it; that behavior needs
/// scripting. The `attrs` (such as `open`) are forwarded to the underlying
/// `<details>`; a `class` among them is appended to the computed classes. It is
/// placed inside a [`navigation_menu_item`].
///
/// ```ignore
/// view! {
///     navigation_menu_item(
///         navigation_menu_dropdown(
///             navigation_menu_trigger("Account")
///             navigation_menu_content(
///                 navigation_menu_link(
///                     attrs: attributes! { href="/logout" },
///                     "Log out"
///                 )
///             )
///         )
///     )
/// }
/// ```
#[component]
pub async fn navigation_menu_dropdown(
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <details class=(class!("group relative", attrs.remove("class"))) (attrs)>
            (child)
        </details>
    }
}
