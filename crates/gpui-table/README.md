# gpui-table

`gpui-table` is the default entry point for the ecosystem. It exposes the
derive-based table workflow, built-in filter integration, row/runtime traits,
and registry support through one dependency.

Most application code should use this crate directly.

## Installation

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["fluent", "inventory", "rust_decimal"] }
```

`derive` and `chrono` are enabled by default.

## Quick Example

`filter(number_range(...))` requires the `rust_decimal` feature.

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum Status {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(text()))]
    pub name: String,

    #[gpui_table(width = 80., filter(number_range(min = 0, max = 120)))]
    pub age: u8,

    #[gpui_table(width = 120., filter(faceted()))]
    pub status: Status,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

This single derive flow gives you:

- `UserTableDelegate` and `UserTableColumn`
- `UserFilterEntities` and `UserFilterValues` when `#[gpui_table(filters)]` is enabled
- typed client-side matching through generated `Matchable<UserFilterValues>`
- optional `GpuiTableShape` registration when the `inventory` feature is enabled

## Main Surface

The user-facing entry points are:

- `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`
- `gpui_table::runtime` for row traits, loaders, built-in filter interop, and default rendering helpers
- `gpui_table::registry` for inventory-backed table metadata when `inventory` is enabled
- root-level runtime re-exports such as `TableLoader`, `TableDataLoader`, `TableRowMeta`, `TableRowStyle`, and `FilterEntitiesExt`

## Feature Flags

- `derive` (default): enables `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`
- `chrono` (default): date cell support and `filter(date_range())`
- `fluent`: localized titles and labels through `es-fluent`
- `inventory`: inventory-backed `GpuiTableShape` registration
- `rust_decimal`: numeric range filtering and decimal-backed helpers
- `spacetimedb`: range filtering support for supported SpacetimeDB temporal types

## When To Reach For Another Crate

- Use `gpui-table-component` when you want to build filter UIs manually rather than through `#[derive(GpuiTable)]`.
- Use `gpui-table-prototyping-core` when you are generating stories or scaffolding from registered `GpuiTableShape` values.

For implementation details, generated contracts, and crate boundaries, see
`docs/ARCHITECTURE.md`.
