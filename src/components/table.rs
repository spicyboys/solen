use topcoat::{
    Result,
    view::{Attributes, View, class, component, view},
};

/// The classes for the [`table`] container.
///
/// The wrapper scrolls a wide table horizontally instead of shrinking its
/// columns; the table itself spans the wrapper's width and places its caption
/// below it.
const TABLE: &str = "relative w-full overflow-x-auto";
const TABLE_INNER: &str = "w-full caption-bottom text-sm";

/// The classes for the [`table_header`].
///
/// Every row in the header carries the table's row border; the header itself
/// has none of its own.
const TABLE_HEADER: &str = "[&_tr]:border-b";

/// The classes for the [`table_body`].
///
/// The last row in the body drops the row border so the table does not end on
/// a dangling rule above an optional footer.
const TABLE_BODY: &str = "[&_tr:last-child]:border-0";

/// The classes for the [`table_footer`].
///
/// The footer rules off the last body row with its own top border and a muted
/// fill.
const TABLE_FOOTER: &str = "border-t bg-muted font-medium [&>tr]:last:border-b-0";

/// The classes for a [`table_row`].
///
/// Rows are separated by a hairline bottom border, tint on hover, and tint
/// more strongly when selected (`data-state="selected"`), the state a parent
/// component such as a data table sets on the row.
const TABLE_ROW: &str = "border-b transition-colors hover:bg-foreground/5 \
    data-[state=selected]:bg-foreground/10";

/// The classes for a [`table_head`] cell.
///
/// Header cells are medium-weight, left-aligned, and hold their height
/// regardless of the cell content they top.
const TABLE_HEAD_BASE: &str = "h-10 px-2 text-left align-middle font-medium \
    whitespace-nowrap text-foreground [&:has([role=checkbox])]:pr-0";

/// The classes for a [`table_cell`].
///
/// Cells keep a uniform padding and baseline alignment. A cell holding a
/// checkbox drops its left padding, mirroring [`table_head`].
const TABLE_CELL: &str = "p-2 align-middle whitespace-nowrap \
    [&:has([role=checkbox])]:pr-0";

/// The classes for the [`table_caption`].
///
/// The caption sits below the table, in the theme's muted text color.
const TABLE_CAPTION: &str = "mt-4 text-sm text-muted-foreground";

/// A table component: a bordered grid of rows and cells.
///
/// A table stacks a [`table_caption`], a [`table_header`] of [`table_head`]s,
/// a [`table_body`] of [`table_row`]s, and an optional [`table_footer`]. The
/// `attrs` (such as `class` or event handlers) are forwarded to the underlying
/// `<table>`; a `class` among them is appended to the computed classes. Child
/// nodes become the table's sections.
///
/// ```ignore
/// view! {
///     table(
///         table_caption("Recent invoices")
///         table_header(
///             table_row(
///                 table_head(attrs: attributes! { class="w-28" }, "Invoice")
///                 table_head("Status")
///             )
///         )
///         table_body(
///             table_row(
///                 table_cell(attrs: attributes! { class="font-medium" }, "INV001")
///                 table_cell("Paid")
///             )
///         )
///     )
/// }
/// ```
#[component]
pub async fn table(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <div class=(TABLE)>
            <table class=(class!(TABLE_INNER, attrs.remove("class"))) (attrs)>
                (child)
            </table>
        </div>
    }
}

/// The caption of a [`table`], describing its contents.
#[component]
pub async fn table_caption(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <caption class=(class!(TABLE_CAPTION, attrs.remove("class"))) (attrs)>
            (child)
        </caption>
    }
}

/// The header of a [`table`], holding a row of column headings.
#[component]
pub async fn table_header(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <thead class=(class!(TABLE_HEADER, attrs.remove("class"))) (attrs)>
            (child)
        </thead>
    }
}

/// The body of a [`table`], holding its data rows.
#[component]
pub async fn table_body(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <tbody class=(class!(TABLE_BODY, attrs.remove("class"))) (attrs)>(child)</tbody>
    }
}

/// The closing section of a [`table`], summarizing its rows.
#[component]
pub async fn table_footer(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <tfoot class=(class!(TABLE_FOOTER, attrs.remove("class"))) (attrs)>
            (child)
        </tfoot>
    }
}

/// A single row of a [`table`], in its header, body, or footer.
///
/// The `attrs` (such as `data-state="selected"`) are forwarded to the
/// underlying `<tr>`; a `class` among them is appended to the computed
/// classes. Child nodes become the row's cells.
#[component]
pub async fn table_row(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! { <tr class=(class!(TABLE_ROW, attrs.remove("class"))) (attrs)>(child)</tr> }
}

/// A heading cell of a [`table`], naming a column in the header.
#[component]
pub async fn table_head(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! {
        <th class=(class!(TABLE_HEAD_BASE, attrs.remove("class"))) (attrs)>(child)</th>
    }
}

/// A data cell of a [`table`], holding one value of a row.
///
/// The `attrs` (such as `colspan`) are forwarded to the underlying `<td>`; a
/// `class` among them is appended to the computed classes.
#[component]
pub async fn table_cell(#[default] mut attrs: Attributes, #[default] child: View) -> Result {
    view! { <td class=(class!(TABLE_CELL, attrs.remove("class"))) (attrs)>(child)</td> }
}
