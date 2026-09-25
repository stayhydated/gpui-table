# gpui-table-derive

[![Codecov: gpui-table-derive][codecov-badge]][codecov]
[![crates.io: gpui-table-derive][crate-badge]][crate]

`gpui-table-derive` provides the proc macros behind
the [`gpui-table`][project] facade for developers integrating generated table,
cell, filter, and MCP query contracts.

## Overview

- `#[derive(GpuiTable)]`
- `#[derive(Filterable)]`
- `#[derive(TableCell)]`
- `#[derive(GpuiTableFilterShape)]`
- `#[derive(McpFilterShape)]` with the `mcp` feature
- `#[gpui_table_impl]`
- `#[mcp_query]` with the `mcp` feature

Application crates should depend on `gpui-table` and use its macro
re-exports. Depend on this proc-macro crate directly only when integrating the
macros without the facade.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-derive
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-derive.svg?label=gpui-table-derive
[crate]: https://crates.io/crates/gpui-table-derive
[project]: https://github.com/stayhydated/gpui-table
