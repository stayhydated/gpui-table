use gpui::{AnyElement, App, Div, InteractiveElement as _, IntoElement, Stateful, Window, div};
use gpui_component::{menu::PopupMenu, table::Column};
use gpui_table_schema::filter::FilterConfig;

use crate::TableCell;

/// Metadata for a table row type.
pub trait TableRowMeta {
    /// Unique identifier for this row type.
    const TABLE_ID: &'static str;

    /// Human-readable title for the table.
    const TABLE_TITLE: &'static str;

    /// Returns the table title. This can be overridden to provide dynamic titles,
    /// for example from localization libraries.
    fn table_title() -> String {
        Self::TABLE_TITLE.to_string()
    }

    /// Returns the column definitions for this row type.
    fn table_columns() -> Vec<Column>;

    /// Returns the value for a specific column index.
    fn cell_value(&self, col_ix: usize) -> Box<dyn TableCell + '_>;

    /// Returns the filter configuration for the table.
    fn table_filters() -> Vec<FilterConfig> {
        Vec::new()
    }
}

/// Styling hooks for a table row.
pub trait TableRowStyle: TableRowMeta {
    /// The type representing the columns of the table.
    type ColumnId: Into<usize> + From<usize>;

    /// Renders a single cell.
    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;

    /// Renders the row container.
    fn render_table_row(&self, row_ix: usize, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        default_render_row(row_ix, window, cx)
    }
}

/// Context-menu hooks for a table row.
pub trait TableRowContextMenu: TableRowMeta {
    /// Builds the context menu for this row.
    fn render_table_context_menu(
        &self,
        _row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut App,
    ) -> PopupMenu {
        menu
    }
}

/// Generated/default context-menu composition hooks for a table row.
pub trait TableRowGeneratedContextMenu: TableRowMeta {
    /// Builds the derive-generated/default portion of the row context menu.
    fn render_generated_table_context_menu(
        &self,
        _row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut App,
    ) -> PopupMenu {
        menu
    }
}

/// Default implementation for rendering a cell.
pub fn default_render_cell<R: TableRowMeta + ?Sized>(
    row: &R,
    col_ix: usize,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    row.cell_value(col_ix).draw(window, cx)
}

/// Default implementation for rendering a row.
pub fn default_render_row(row_ix: usize, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
    div().id(row_ix)
}
