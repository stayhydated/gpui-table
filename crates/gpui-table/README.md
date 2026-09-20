# gpui-table

[![Codecov: gpui-table][codecov-badge]][codecov]
[![crates.io: gpui-table][crate-badge]][crate]

`gpui-table` is the application-facing facade for strongly typed
GPUI tables. It re-exports the table derives, core filter semantics, runtime
traits, schema types, and optional MCP integration. Built-in filter widgets live
in `gpui-table-component`.

## Overview

The default features are `derive` and `chrono`. Enable
`rust_decimal` for numeric range filters, `fluent` for
localized labels, `inventory` for registered table metadata,
`mcp` for generated query tools, or `spacetimedb` for the
supported temporal conversions.

## Example

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
struct User {
    #[gpui_table(sortable, width = 180.)]
    name: String,
}
```

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table.svg?label=gpui-table
[crate]: https://crates.io/crates/gpui-table
