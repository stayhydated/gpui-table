# gpui-table

Facade crate for the `gpui-table` ecosystem. Re-exports filter semantics,
runtime traits/helpers, schema metadata, and, with the default `derive`
feature, the derive macros through one crate.

## Install

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["inventory", "fluent", "rust_decimal", "chrono"] }
```

## Features

- `derive` (default): `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, `#[derive(TableCell)]`, and `#[gpui_table_impl]`
- `chrono` (default): date `TableCell` support + date-range filter helpers
- `inventory`: registers table metadata for tooling
- `fluent`: localized titles/labels via `es-fluent`
- `rust_decimal`: numeric range helpers for filters
- `spacetimedb`: SpacetimeDB temporal range-filter support

## Quick example

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;
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

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

## Exports

- `gpui_table::core` for filter semantics (`Matchable`, typed filter values, feature-gated conversions)
- `gpui_table::filter` as a convenience alias for `gpui_table::core::filter`
- `gpui_table::runtime` for row traits, loaders, default rendering, and built-in filter runtime helpers
- `gpui_table::schema` for registry and filter metadata
- `gpui_table::registry` as a convenience re-export of `gpui_table::schema::registry`
- root-level convenience re-exports: `TableCell`, `TableLoader`,
  `TableDataLoader`, `TableRowMeta`, `TableRowStyle`,
  `TableRowContextMenu`, `TableRowGeneratedContextMenu`, `FilterEntitiesExt`
- derive macros re-exported from `gpui-table-derive` (`GpuiTable`, `Filterable`, `TableCell`, `gpui_table_impl`) when `derive` is enabled (default)
- `gpui_table::runtime::generated_filters` for built-in filter components and
  query-string helpers when you need manual/generated filter interop
