# gpui-table-mcp

`gpui-table-mcp` is the experimental MCP integration for exposing
table rows through typed query tools. It provides query contracts, schemas,
inventory registration, server composition, table resources, and prompt
templates.

Most applications should enable the facade and component features:

```toml
[dependencies]
gpui-table = { version = "0.6", features = ["mcp"] }
gpui-table-component = { version = "0.6", features = ["mcp"] }
```

Use this crate directly when implementing a custom server or lower-level table
registration. Query execution remains application-owned.

`gpui_table::mcp::tool_registry()?` returns the inventory-discovered MCP
definitions and handlers for hosts that assemble the registry independently.
MCP servers retain that registry across calls until their transport or host is
explicitly stopped.

- [MCP query guide](https://stayhydated.github.io/gpui-table/book/mcp.html)
- [Runnable MCP example](https://github.com/stayhydated/gpui-table/tree/master/examples/mcp-query)
- [API documentation](https://docs.rs/gpui-table-mcp/)
