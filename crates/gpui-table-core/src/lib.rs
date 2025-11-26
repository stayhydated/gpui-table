use gpui::{AnyElement, App, Div, InteractiveElement as _, IntoElement, Stateful, Window, div};
use gpui_component::table::Column;
use paste::paste;

/// Metadata for a table row type.
///
/// This trait describes the structure of the row (columns, ID, title)
/// and provides a way to extract cell values.
pub trait TableRowMeta {
    /// Unique identifier for this row type.
    const TABLE_ID: &'static str;

    /// Human-readable title for the table.
    const TABLE_TITLE: &'static str;

    /// Returns the column definitions for this row type.
    fn table_columns() -> &'static [Column];

    /// Returns the value for a specific column index.
    fn cell_value(&self, col_ix: usize) -> TableCellValue<'_>;
}

/// Styling hooks for a table row.
///
/// This trait allows customizing how rows and cells are rendered.
/// The `NamedTableRow` derive macro generates a default implementation
/// that uses `default_render_cell` and `default_render_row`.
pub trait TableRowStyle {
    /// Renders a single cell.
    fn render_table_cell(&self, col_ix: usize, window: &mut Window, cx: &mut App) -> AnyElement;

    /// Renders the row container.
    fn render_table_row(&self, row_ix: usize, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

macro_rules! impl_table_cell_value {
    (clone: $($variant:ident($t:ty)),* $(,)?) => {
        paste! {
            $(
                impl From<$t> for TableCellValue<'_> {
                    fn from(v: $t) -> Self { TableCellValue::$variant(v) }
                }

                impl From<&$t> for TableCellValue<'_> {
                    fn from(v: &$t) -> Self { TableCellValue::$variant(v.clone()) }
                }
            )*
        }
    };
    (copy: $($variant:ident($t:ty)),* $(,)?) => {
        paste! {
            $(
                impl From<$t> for TableCellValue<'_> {
                    fn from(v: $t) -> Self { TableCellValue::$variant(v) }
                }

                impl From<&$t> for TableCellValue<'_> {
                    fn from(v: &$t) -> Self { TableCellValue::$variant(*v) }
                }
            )*
        }
    };
}

/// A value that can be displayed in a table cell.
#[derive(Clone, Debug)]
pub enum TableCellValue<'a> {
    Str(&'a str),
    String(String),
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
}

impl<'a> From<&'a str> for TableCellValue<'a> {
    fn from(s: &'a str) -> Self {
        TableCellValue::Str(s)
    }
}

impl_table_cell_value!(clone: String(String));

impl_table_cell_value!(copy:
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
);

/// Default implementation for rendering a cell.
pub fn default_render_cell<R: TableRowMeta + ?Sized>(
    row: &R,
    col_ix: usize,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    match row.cell_value(col_ix) {
        TableCellValue::Str(s) => s.to_string(),
        TableCellValue::String(s) => s,
        TableCellValue::Usize(n) => n.to_string(),
        TableCellValue::U8(n) => n.to_string(),
        TableCellValue::U16(n) => n.to_string(),
        TableCellValue::U32(n) => n.to_string(),
        TableCellValue::U64(n) => n.to_string(),
        TableCellValue::I8(n) => n.to_string(),
        TableCellValue::I16(n) => n.to_string(),
        TableCellValue::I32(n) => n.to_string(),
        TableCellValue::I64(n) => n.to_string(),
        TableCellValue::F32(n) => format!("{:.2}", n),
        TableCellValue::F64(n) => format!("{:.2}", n),
        TableCellValue::Bool(b) => if b { "✓" } else { "✗" }.to_string(),
    }
}

/// Default implementation for rendering a row.
pub fn default_render_row(row_ix: usize, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
    div().id(row_ix)
}
