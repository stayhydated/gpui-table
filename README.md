# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` is a Rust workspace for building strongly typed GPUI tables.
It combines derive macros, typed filter values, a GPUI runtime layer, UI-neutral
registry metadata, and prototyping/codegen helpers.

The project is organized around three priorities:

1. **Type safety** for generated columns, filters, delegates, and metadata.
1. **Ergonomics** for `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, and `#[gpui_table_impl]`.
1. **Developer experience** for built-in filters, inventory-backed table shapes, and example-driven workflows.

## Installation

```toml
[dependencies]
gpui-table = { version = "*", features = ["fluent", "inventory", "rust_decimal"] }
```

## Compatibility

| `gpui-table` | `gpui-component` | `gpui` |
| :----------- | :--------------- | :----- |
| **git** | | |
| `branch = "master"` | `branch = "main"` | `rev = "832c17e8192e2e1d472f0751e7cef2af84ded622"` |

`derive` and `chrono` are enabled by default. Add:

- `rust_decimal` for `filter(number_range(...))`
- `inventory` for `GpuiTableShape` registration and prototyping/codegen
- `fluent` for localized table titles and faceted labels through `es-fluent`
- `spacetimedb` for range filtering over supported SpacetimeDB temporal types

## Quick Start

`number_range(...)` requires the `rust_decimal` feature.

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum UserStatus {
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
    pub status: UserStatus,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        // fetch rows, append them to self.rows, then notify
        cx.notify();
    }
}
```

With `#[gpui_table(filters)]`, the derive also generates:

- `UserTableDelegate` and `UserTableColumn`
- `UserFilterEntities` for rendering built-in filters
- `UserFilterValues` for typed filter state
- `Matchable<UserFilterValues>` so client-side filtering stays strongly typed

If you enable `inventory`, the same derive registers a `GpuiTableShape` for
tooling and code generation.

### Table cells for value objects

Single-field wrappers still render by delegating to their inner value by
default. When a wrapper should render through its own display implementation or
a formatter, use `#[table_cell(display)]` or `#[table_cell(format = ...)]`.

```rs
use gpui_table::TableCell;
use std::fmt;

#[derive(TableCell)]
#[table_cell(display)]
pub struct AccountCode(String);

impl fmt::Display for AccountCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

fn format_percentage(value: &Percentage) -> String {
    format!("{}%", value.0)
}

#[derive(TableCell)]
#[table_cell(format = format_percentage)]
pub struct Percentage(i64);
```

### Localized labels

With the `fluent` feature, table titles and faceted labels are localized through
typed `es-fluent` messages and labels.

```rs
use es_fluent::{EsFluentLabel, EsFluentVariants};
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, EsFluentLabel, EsFluentVariants, GpuiTable)]
#[fluent_label(origin, variants)]
#[fluent_variants(keys = ["label"])]
#[gpui_table(fluent = "label", filters)]
pub struct User {
    #[gpui_table(filter(faceted()))]
    pub status: UserStatus,
}
```

The built-in table/filter widgets keep their embedded `es-fluent` localizer in
GPUI global state. The example app declares its languages with
`#[es_fluent_language]`, initializes `gpui_table_component::i18n` during GPUI
startup, and selects the active locale through the GPUI storybook locale APIs.
Generated Storybook table titles use the GPUI app context so they follow the
active Storybook locale; truly context-free metadata can still use fallback
label helpers.

## Examples

The canonical end-to-end examples live under [`examples/`](examples/README.md).

- `cargo run`
  Launches `examples/some-lib-tables`, the storybook app for the derived tables.
- `cargo run -p prototyping`
  Regenerates `examples/prototyping/output` from `GpuiTableShape` inventory registrations.

The main walkthrough files are:

- `examples/some-lib/src/structs/user.rs` for derived filters, localized titles, and context menus
- `examples/some-lib/src/structs/item.rs` for load-more and custom row rendering
- `examples/prototyping/src/main.rs` for inventory-driven code generation

## Feature Flags

- `derive` (default): re-exports `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`
- `chrono` (default): date cell rendering and `filter(date_range())`
- `fluent`: localized titles and faceted labels through `es-fluent`
- `inventory`: inventory-backed `GpuiTableShape` registration for tooling
- `rust_decimal`: numeric range filtering and decimal-backed helpers
- `spacetimedb`: SpacetimeDB temporal range filtering support
