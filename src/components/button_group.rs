use topcoat::{
    Result,
    view::{Attributes, View, class, component, view},
};

/// The layout direction of a [`button_group`].
///
/// [`Default`] is `ButtonGroupOrientation::Horizontal`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonGroupOrientation {
    /// Buttons laid out in a row, joined at the sides.
    #[default]
    Horizontal,
    /// Buttons laid out in a column, joined at the top and bottom.
    Vertical,
}

impl ButtonGroupOrientation {
    /// The `data-orientation` value for this orientation.
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    /// The Tailwind classes for this orientation.
    ///
    /// The group removes the shared border and rounds the shared corners of
    /// every button but the outer pair: the first button keeps its leading
    /// corners, the last keeps its trailing corners, and every button between
    /// is squared off on both sides. The selectors target direct children so
    /// the group accepts `<button>`s, links styled with [`button_variants`],
    /// and inputs alike.
    fn classes(self) -> &'static str {
        match self {
            Self::Horizontal => {
                "[&>*:not(:first-child)]:rounded-l-none \
                 [&>*:not(:first-child)]:border-l-0 \
                 [&>*:not(:last-child)]:rounded-r-none"
            }
            Self::Vertical => {
                "flex-col [&>*:not(:first-child)]:rounded-t-none \
                 [&>*:not(:first-child)]:border-t-0 \
                 [&>*:not(:last-child)]:rounded-b-none"
            }
        }
    }
}

/// The layout direction of a [`button_group_separator`].
///
/// [`Default`] is `SeparatorOrientation::Vertical`, standing between
/// side-by-side buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SeparatorOrientation {
    /// A line running across a row of buttons.
    Horizontal,
    /// A line running between buttons in a row.
    #[default]
    Vertical,
}

impl SeparatorOrientation {
    /// The `data-orientation` value for this orientation.
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// The classes shared by every [`button_group`], regardless of orientation.
///
/// The group is sized to its content, has no gap so children sit flush, and
/// stretches every child to a common height. Focused children are lifted above
/// their neighbors so the focus ring draws over the join. A nested group (which
/// carries `data-slot="button-group"`) is given a gap so it reads as a separate
/// cluster. Inputs stretch to fill the row.
const BUTTON_GROUP: &str = "flex w-fit items-stretch \
    has-[>[data-slot=button-group]]:gap-2 \
    [&>*]:focus-visible:relative [&>*]:focus-visible:z-10 [&>input]:flex-1";

/// A container that groups related buttons together with consistent styling.
///
/// A group lays its direct children out flush, joining them by removing the
/// border and rounding between neighbors. Each child is a real control: a
/// [`button`], a link styled with [`button_variants`], or an input. The
/// `orientation` selects a row or a column, defaulting to `Horizontal`. The
/// `attrs` (such as `class` or `aria-label`) are forwarded to the underlying
/// `<div>`, which carries `role="group"`; a `class` among them is appended to
/// the computed classes. Child nodes become the grouped controls.
///
/// ```ignore
/// view! {
///     button_group(
///         attrs: attributes! { class="justify-end" },
///         <a
///             href="/login"
///             class=(button_variants(ButtonVariant::Outline, ButtonSize::Sm))
///         >
///             "Sign in"
///         </a>
///         button(
///             variant: ButtonVariant::Outline,
///             size: ButtonSize::Sm,
///             "Sign up"
///         )
///     )
/// }
/// ```
///
/// To divide buttons that share a fill, drop a [`button_group_separator`]
/// between them.
#[component]
pub async fn button_group(
    #[default] orientation: ButtonGroupOrientation,
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <div
            role="group"
            data-slot="button-group"
            data-orientation=(orientation.as_str())
            class=(class!(
                BUTTON_GROUP, orientation.classes(), attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    }
}

/// The classes for a [`button_group_separator`].
///
/// The separator is a hairline in the theme's border color. It stretches across
/// the group's cross axis and sizes itself by orientation: vertical separators
/// (the default, between buttons in a row) are a 1-pixel-wide line inset by a
/// margin each side, horizontal ones a 1-pixel-tall line.
const BUTTON_GROUP_SEPARATOR: &str = "shrink-0 bg-border relative self-stretch \
    data-[orientation=horizontal]:h-px data-[orientation=horizontal]:mx-px \
    data-[orientation=horizontal]:w-auto \
    data-[orientation=vertical]:w-px data-[orientation=vertical]:my-px \
    data-[orientation=vertical]:h-auto";

/// A visual divider between buttons in a [`button_group`].
///
/// Use it when the grouped buttons share a fill and would otherwise blend
/// together; buttons with the `Outline` variant already draw their own border
/// and need no separator. The `orientation` defaults to `Vertical`, standing
/// between side-by-side buttons. The `attrs` are forwarded to the underlying
/// `<div>`, which carries `role="separator"`; a `class` among them is appended
/// to the computed classes.
///
/// ```ignore
/// view! {
///     button_group(
///         button(
///             variant: ButtonVariant::Secondary,
///             size: ButtonSize::Sm,
///             "Copy"
///         )
///         button_group_separator()
///         button(
///             variant: ButtonVariant::Secondary,
///             size: ButtonSize::Sm,
///             "Paste"
///         )
///     )
/// }
/// ```
#[component]
pub async fn button_group_separator(
    #[default] orientation: SeparatorOrientation,
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    view! {
        <div
            role="separator"
            data-slot="button-group-separator"
            data-orientation=(orientation.as_str())
            class=(class!(BUTTON_GROUP_SEPARATOR, attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    }
}
