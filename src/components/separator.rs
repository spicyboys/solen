use topcoat::{
    Result,
    view::{Attributes, View, class, component, view},
};

/// The orientation of a [`separator`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SeparatorOrientation {
    /// A horizontal rule, drawing a full-width line.
    #[default]
    Horizontal,
    /// A vertical rule, drawing a full-height line.
    Vertical,
}

/// The classes shared by every separator.
///
/// Only the hairlines are set here: the caller sizes the separator along its
/// primary axis with a `class` (for example `h-4` for a short vertical rule in
/// a header, or `h-px w-full` for a horizontal rule).
const SEPARATOR: &str = "shrink-0 bg-border";

/// A hairline rule that groups or divides content.
///
/// The `orientation` selects between a horizontal and a vertical rule,
/// reflected in a `data-orientation` attribute that styles can key on. The
/// `attrs` (such as a `class` sizing the rule) are forwarded to the underlying
/// `<div>`; a `class` among them is appended to the computed classes.
///
/// ```ignore
/// view! {
///     separator(class: "h-4")
///     separator(orientation: SeparatorOrientation::Vertical, class: "h-4")
/// }
/// ```
#[component]
pub async fn separator(
    #[default] orientation: SeparatorOrientation,
    #[default] mut attrs: Attributes,
    #[default] child: View,
) -> Result {
    let data = if orientation == SeparatorOrientation::Horizontal {
        "horizontal"
    } else {
        "vertical"
    };
    view! {
        <div
            role="separator"
            data-orientation=(data)
            class=(class!(SEPARATOR, attrs.remove("class")))
            (attrs)
        >
            (child)
        </div>
    }
}
