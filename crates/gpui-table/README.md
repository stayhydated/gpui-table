# gpui-table

Facade crate for the `gpui-table` ecosystem. Re-exports core traits, derive macros,
and optional filter UI components so applications only need one dependency.

## Install

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component", "inventory", "fluent", "rust_decimal", "chrono"] }
```

## Features

* `derive` (default): `#[derive(GpuiTable)]` and `#[derive(TableCell)]`
* `chrono` (default): date `TableCell` support + date-range filter helpers
* `component`: filter UI components under `gpui_table::component`
* `inventory`: registers table metadata for tooling
* `fluent`: localized titles/labels via `es-fluent`
* `rust_decimal`: numeric range helpers for filters

## Quick example

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::{GpuiTable, TableLoader};

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
        // fetch + append rows
        cx.notify();
    }
}
```

## Exports

- `gpui_table_core` traits and filter types (including `TableLoader`/`TableDataLoader`)
- `gpui_table_derive` macros (with `derive`)
- `gpui_table::component` filter components + extension traits (with `component`)

Note: `TableStatusBar` lives in `gpui-table-component` and is not re-exported by
the facade crate.
