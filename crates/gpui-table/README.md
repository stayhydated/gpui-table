# gpui-table

[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` is the application-facing facade for strongly typed
GPUI tables. It re-exports the table derives, core filter semantics, runtime
traits, schema types, and optional MCP integration. Built-in filter widgets live
in `gpui-table-component`.

```toml
[dependencies]
gpui-table = "0.5"
gpui-table-component = "0.5"
```

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
struct User {
    #[gpui_table(sortable, width = 180.)]
    name: String,
}
```

The default features are `derive` and `chrono`. Enable
`rust_decimal` for numeric range filters, `fluent` for
localized labels, `inventory` for registered table metadata,
`mcp` for generated query tools, or `spacetimedb` for the
supported temporal conversions.

See the [user guide](https://stayhydated.github.io/gpui-table/book/) for setup,
filters, loading, localization, and MCP workflows. See the
[API documentation](https://docs.rs/gpui-table/) for the complete public
surface.
