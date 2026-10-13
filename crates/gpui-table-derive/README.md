# gpui-table-derive

[![Codecov: gpui-table-derive][codecov-badge]][codecov]
[![crates.io: gpui-table-derive][crate-badge]][crate]

`gpui-table-derive` provides the proc macros behind
the [`gpui-table`][facade] facade for developers integrating generated table,
cell, filter, and MCP query contracts.

Requires Rust 1.99 or newer. Generated UI code uses `gpui-kit` 0.7 directly.

## Overview

- `#[derive(GpuiTable)]`
- `#[derive(Filterable)]`
- `#[derive(TableCell)]`
- `#[derive(GpuiTableFilterShape)]`
- `#[derive(McpFilterShape)]` with the `mcp` feature
- `#[gpui_table_impl]`
- `#[mcp_query]` with the `mcp` feature

Application crates should use the facade's `derive` feature and macro
re-exports. A direct dependency is useful for explicit macro imports in
integration tooling; generated code still requires the `gpui-table` facade
paths and its matching integration features. The [macro API reference][api]
documents the available derives and attributes.

## Features

No features are enabled by default. Applications normally select these through
the facade, which keeps generated-code dependencies aligned.

| Feature | Default | Purpose |
| --- | --- | --- |
| `chrono` | No | Accepts built-in date-range shapes during macro expansion. |
| `fluent` | No | Generates typed Fluent labels for tables and facets. |
| `inventory` | No | Generates inventory-registered table metadata. |
| `mcp` | No | Enables `inventory`, MCP table generation, `McpFilterShape`, and `mcp_query`. |
| `rust_decimal` | No | Accepts built-in numeric range shapes during macro expansion. |
| `spacetimedb` | No | Accepts supported SpacetimeDB temporal range-filter fields. |

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-derive
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-derive.svg?label=gpui-table-derive
[crate]: https://crates.io/crates/gpui-table-derive
[facade]: https://github.com/stayhydated/gpui-table/blob/master/crates/gpui-table/README.md
[api]: https://docs.rs/gpui-table-derive/latest/gpui_table_derive/
