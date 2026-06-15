# MCP Query Example

This example exposes a derived `IssueRow` table as a stdio MCP server. The row
source is registered with `#[gpui_table::mcp_query]`, which infers `IssueRow`
from the zero-argument `Vec<IssueRow>` return type, so `main` only serves the
inventory-backed registry. The table also declares explicit MCP tool metadata
with `#[gpui_table(mcp(...))]` and explicit `#[gpui_table(filter(...))]`
fields for text, faceted, number-range, and date-range filter shapes. Because
this is an MCP-only table, it does not need to spell struct-level `filters`.
Filtered fields can use `#[koruma(...)]` to validate decoded MCP filter
arguments before the query handler runs. Custom filters that adapt built-in shapes can use
`#[derive(gpui_table::GpuiTableFilterShape)]`; see the `gpui-table-mcp`
README for that adapter pattern.

Run it from the workspace root:

```sh
cargo run -p mcp-query
```

List the generated tool:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | cargo run -q -p mcp-query
```

Call the tool with generated filters:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"mcp_query_issues","arguments":{"state":["Open"],"updated_on":{"min":"2026-06-01"},"limit":10,"offset":0}}}' \
  | cargo run -q -p mcp-query
```

The server does not instantiate GPUI windows or filter widgets. It decodes JSON
arguments into generated filter values, applies the generated
`Matchable` implementation to in-memory rows, and returns `rows`, `total`,
`offset`, and `limit`.
