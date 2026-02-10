# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

A struct derive macro for deriving [gpui-component](https://crates.io/crates/gpui-component) tables. It autogenerates column definitions, renderers, and delegate glue (i18n, sorting, load-more, ...).

## Compatibility

| `gpui-table` | `gpui-component` |
| :------------ | :--------------- |
| **git** | |
| `master` | `main` |
| **crates.io** | |
| `0.6.x` | `0.6.x` |

## Interactive examples

```sh
cargo run
```

## Quick Example

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

## Prototyping

Enable the `inventory` feature on `gpui-table` and use `gpui-table-prototyping-core`
to generate gpui form scaffolding from `GpuiTableShape` registrations.
See `examples/prototyping` for a working generator.

## Examples

- `examples/i18n`: i18n resources
- `examples/some-lib`: crate types shared by examples
- `examples/some-lib-tables`: storybook-like GPUI app showcasing tables
- `examples/prototyping`: generator for table stories and scaffolding
