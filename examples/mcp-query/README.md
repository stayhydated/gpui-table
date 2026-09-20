# mcp-query

This example exposes an in-memory `IssueRow` table as a stdio MCP
query tool. It demonstrates explicit text, faceted, numeric range, and date
range filter shapes, precise row output schemas, and a local row source
registered with `#[gpui_table::mcp_query]`.

## Example

Run it from the workspace root:

```sh
cargo run -p mcp-query
```

The generated `mcp_query_issues` tool accepts arguments such as:

```json
{
  "state": ["Open"],
  "updated_on": { "min": "2026-06-01" },
  "limit": 10,
  "offset": 0
}
```

Successful calls return `rows`, `total`, `offset`, and `limit`.
`total` counts all matching rows before pagination; `rows` contains the
requested page.
