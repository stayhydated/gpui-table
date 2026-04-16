# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` is a Rust workspace for building strongly typed
[gpui-component](https://crates.io/crates/gpui-component) tables. It combines
derive macros, pure filter semantics, a GPUI runtime layer, UI-neutral schema
metadata, and prototyping utilities for code generation.

## Workspace layout

| Crate | Purpose |
| :---- | :------ |
| `gpui-table` | Facade crate that re-exports the core/runtime/schema layers and, with `derive`, the proc macros. |
| `gpui-table-core` | Pure filter semantics, typed filter values, and range conversion traits. |
| `gpui-table-runtime` | GPUI-facing table traits, default cell rendering, load-more wiring, and the generated-filter runtime facade. |
| `gpui-table-schema` | Static filter metadata and inventory-backed `GpuiTableShape` registry types. |
| `gpui-table-derive` | `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl` proc macros. |
| `gpui-table-component` | Built-in filter components and `TableStatusBar`. |
| `gpui-table-prototyping-core` | Code generation helpers that consume `GpuiTableShape` metadata. |

## Compatibility

| `gpui-table` | `gpui-component` | `gpui` |
| :--------------- | :--------------- | :--------------------------------------------- |
| **git** | |
| `master` | `main` | rev `15d8660748b508b3525d3403e5d172f1a557bfa5` |

## Interactive examples

```sh
cargo run
```

From the workspace root, this launches the `examples/some-lib-tables` app,
which is the default workspace member.

## Quick Example

This example uses `filter(number_range(...))`, so consumers need the
`gpui-table/rust_decimal` feature enabled.

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
        // fetch + append rows
        cx.notify();
    }
}
```

## Prototyping

Enable the `inventory` feature on `gpui-table` and use `gpui-table-prototyping-core`
to generate GPUI table scaffolding from
`gpui_table::schema::registry::GpuiTableShape` registrations.
See `examples/prototyping` for a working generator.

## Examples

- `examples/i18n`: shared i18n resources used by the example crates
- `examples/some-lib`: shared domain types, row structs, and derived filterable enums
- `examples/some-lib-tables`: storybook-style GPUI app showcasing the generated tables
- `examples/prototyping`: generator that writes table stories into `examples/prototyping/output`
