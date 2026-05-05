# gpui-table-derive

`gpui-table-derive` contains the proc macros behind the `gpui-table`
derive-based workflow.

Most application code should depend on `gpui-table` and use the macro
re-exports from there. This crate is mainly for people reading the macro docs
or integrating with the proc-macro layer directly.

## Macros

### `#[derive(GpuiTable)]`

Generates the typed table delegate, column enum, row metadata, optional filter
entities/values, and optional inventory registration for a row struct.

```rs
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(text()))]
    pub name: String,

    #[gpui_table(width = 80., filter(number_range(min = 0, max = 120)))]
    pub age: u8,

    #[gpui_table(width = 90., filter(faceted()))]
    pub active: bool,
}
```

Built-in filter syntaxes:

- `filter(text())`
- `filter(number_range(...))`
- `filter(date_range())`
- `filter(faceted(...))`

Feature requirements are validated during macro expansion:

- `number_range(...)` requires `gpui-table/rust_decimal`
- `date_range()` requires `gpui-table/chrono`
- supported SpacetimeDB range usage requires `gpui-table/spacetimedb`

### `#[derive(Filterable)]`

Generates `FilterValue`, `Filterable`, and `variant_name()` for faceted-filter
enums.

```rs
use gpui_component::IconName;
use gpui_table::Filterable;

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum Status {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(label = "Needs Review")]
    Pending,
}
```

Use enum-level `#[filter(fluent)]` when labels should come from `es-fluent`.

```rs
use gpui_table::Filterable;

#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable)]
#[filter(fluent)]
pub enum Status {
    Active,
    Pending,
}
```

Use struct-level `#[gpui_table(fluent = "label")]` with
`EsFluentLabel`/`EsFluentVariants` when generated table titles and field labels
should use typed Fluent resources.

### `#[derive(TableCell)]`

Generates a `TableCell` impl for single-field wrapper types and unit enums.
This is useful when a column should render through an inner type but you still
want a dedicated wrapper in your domain model.

### `#[gpui_table_impl]`

Attaches load-more behavior to the generated delegate.

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

## Context Menu Helpers

`#[derive(GpuiTable)]` also supports generated row-context-menu links through:

- `context_menu_row_id = "field_name"` or field-level `#[gpui_table(context_menu_id)]`
- `context_menu_route = "/users/{id}"` or `context_menu_route_fn = path::to_fn`
- `context_menu_label = "Open"` or `context_menu_label_fn = path::to_fn`
- `custom_context_menu` when you want to compose the generated link with your own menu items

## When Not To Depend On This Crate Directly

- Use `gpui-table` for the normal application-facing workflow.
- Use `gpui-table-component` if you only need the built-in filter widgets.
- Use `gpui-table-prototyping-core` if you are consuming inventory metadata for generation.

For expansion details, generated type contracts, and test coverage boundaries,
see `docs/ARCHITECTURE.md`.
