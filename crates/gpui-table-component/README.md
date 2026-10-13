# gpui-table-component

[![Codecov: gpui-table-component][codecov-badge]][codecov]
[![crates.io: gpui-table-component][crate-badge]][crate]

`gpui-table-component` provides the built-in GPUI table filters,
adapter shapes, controlled `SortEditor`, reset control, and `TableStatusBar` for applications using
the [`gpui-table`][facade] derive workflow.

Requires Rust 1.99 or newer. Uses `gpui-kit` 0.7.

Use the same components directly when the application owns the filter layout.

Date filter labels use localized formatting for dates representable by Jiff.
Dates outside that range retain their original Chrono `Display` representation.
Initialize [`i18n::init`][i18n] after GPUI component initialization and before
rendering filters.

## Features

The default feature set is `chrono` and `rust_decimal`. Match the facade
features when deriving tables with date, numeric, SpacetimeDB, or MCP shapes.

| Feature | Default | Purpose |
| --- | --- | --- |
| `default` | Yes | Enables `chrono` and `rust_decimal`. |
| `chrono` | Yes | Provides date-range widgets and adapters, with Chrono, Jiff, ICU formatting, and matching runtime/core support. |
| `rust_decimal` | Yes | Provides numeric range widgets and adapters for decimal, integer, and float fields. |
| `mcp` | No | Implements MCP schemas and decoding for built-in filter shapes. |
| `spacetimedb` | No | Enables `chrono` and supported SpacetimeDB field conversions; numeric temporal filters also need `rust_decimal`. |
| `story` | No | Enables `chrono`, `rust_decimal`, and the built-in widget Storybook binary; intended for previews. |

## Example

With the facade's default `derive` feature and a direct `gpui-kit` dependency,
this complete row model uses the built-in `TextFilter` shape:

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

fn main() {
    let mut delegate = UserTableDelegate::new(vec![User { name: "Ada".into() }]);
    delegate.set_filter_values(UserFilterValues { name: "ada".into() });
    assert_eq!(delegate.visible_row_indices(), vec![0]);
}
```

The derive also creates `UserFilterEntities`; rendering them needs the
application's window and context. The [complete table view][table-view] builds
filter entities and renders their elements beside `DataTable`.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-component
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-component.svg?label=gpui-table-component
[crate]: https://crates.io/crates/gpui-table-component
[facade]: https://github.com/stayhydated/gpui-table/blob/master/crates/gpui-table/README.md
[table-view]: https://github.com/stayhydated/gpui-table/blob/master/examples/some-lib-tables/src/tables/user.rs
[i18n]: https://docs.rs/gpui-table-component/latest/gpui_table_component/i18n/fn.init.html
