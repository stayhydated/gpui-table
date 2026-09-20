# gpui-table-mcp

[![Codecov: gpui-table-mcp][codecov-badge]][codecov]
[![crates.io: gpui-table-mcp][crate-badge]][crate]

`gpui-table-mcp` is the experimental MCP integration for exposing
table rows through typed query tools. It provides query contracts, schemas,
inventory registration, server composition, table resources, and prompt
templates for the [`gpui-table`][project] ecosystem.

Most applications use the `mcp` feature on `gpui-table` and
`gpui-table-component`.

Use this crate directly when implementing a custom server or lower-level table
registration. Query execution remains application-owned.

`gpui_table::mcp::tool_registry()?` returns the inventory-discovered MCP
definitions and handlers for hosts that assemble the registry independently.
MCP servers retain that registry across calls until their transport or host is
explicitly stopped.

The [MCP example][mcp-example]
shows a complete stdio server with an in-memory row source.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-mcp
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-mcp.svg?label=gpui-table-mcp
[crate]: https://crates.io/crates/gpui-table-mcp
[project]: https://github.com/stayhydated/gpui-table
[mcp-example]: https://github.com/stayhydated/gpui-table/tree/master/examples/mcp-query
