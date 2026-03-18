# gpui-table-derive

Proc-macros for generating table columns, delegates, filters, and optional
registry metadata.

## Macros

- `#[derive(GpuiTable)]`: derive table metadata + delegate
- `#[derive(TableCell)]`: derive `TableCell` for newtypes and enums
- `#[gpui_table_impl]`: wire load-more behavior into a generated delegate

## Example

```rs
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(text()))]
    pub name: String,

    #[gpui_table(width = 120., filter(number_range(min = 0, max = 100)))]
    pub score: u8,
}
```

## Load-more wiring

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::TableLoader;

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        // fetch + append rows
        cx.notify();
    }
}
```

## Row context menu wiring

```rs
use gpui::{App, Window};
use gpui_component::menu::PopupMenu;
use gpui_table::{GpuiTable, TableRowContextMenu};

#[derive(GpuiTable)]
#[gpui_table(custom_context_menu)]
pub struct User {
    pub name: String,
}

impl TableRowContextMenu for User {
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
```

## Filter attributes

- `filter(text())`
- `filter(number_range(min = 0, max = 100, step = 5))`
- `filter(date_range())`
- `filter(faceted(searchable))`

## Generated reset bindings

When `#[gpui_table(filters)]` is enabled, generated `XxxFilterEntities` also include:

- `reset_filters(&self, window, cx)` to clear all filters and trigger one reload callback.
- `reset_button(&self)` to build a localized `ResetFilters` control.
- `all_filters_with_reset(&self)` to render filters plus the reset control.
- `build_for_table(table, cx)` to auto-wire filter changes into the generated
  `TableDelegate` (client-side filtering for `DataTable`).
- `build_for_table_loader(table, window, cx)` to auto-wire filter changes into
  delegate-owned filter state and call `TableDataLoader::load_data(...)`.
- `build_for_table_loader_with(table, before_reload, window, cx)` to customize
  delegate state reset behavior before each reload.
