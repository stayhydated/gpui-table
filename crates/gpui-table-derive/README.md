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
        row_ix: usize,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> PopupMenu {
        use gpui_table::TableRowGeneratedContextMenu as _;
        self.render_generated_table_context_menu(row_ix, menu, window, cx)
            .link("Share", "https://example.com")
    }
}
```

## Row context menu link from row id

```rs
use gpui_table::GpuiTable;

#[derive(GpuiTable)]
#[gpui_table(
    context_menu_row_id = "id",
    context_menu_route = "/users/{id}",
    context_menu_label = "Open user"
)]
pub struct User {
    pub id: u64,
    pub name: String,
}
```

This generates a `TableRowContextMenu` impl that adds a menu link by replacing
`{id}` in `context_menu_route` with `self.id.to_string()`.

You can also mark the field directly and provide runtime functions:

```rs
use gpui_table::GpuiTable;

fn user_href(id: &u64) -> String {
    format!("/users/{id}")
}

fn user_label(_id: &u64) -> &'static str {
    "Open user"
}

#[derive(GpuiTable)]
#[gpui_table(
    context_menu_route_fn = user_href,
    context_menu_label_fn = user_label
)]
pub struct User {
    #[gpui_table(context_menu_id)]
    pub id: u64,
    pub name: String,
}
```

Supported context-menu derive options:

- `context_menu_row_id = "field_name"` or field-level `#[gpui_table(context_menu_id)]`
- `context_menu_route = "/path/{id}"` or `context_menu_route_fn = path::to_fn`
- `context_menu_label = "Open"` or `context_menu_label_fn = path::to_fn`

When `#[gpui_table(custom_context_menu)]` is enabled, the derive still generates
`TableRowGeneratedContextMenu` so custom implementations can compose generated
items with additional actions.

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
