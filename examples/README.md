# Examples

Run these commands from the workspace root:

| Command | Purpose |
|---|---|
| `cargo run` | Open the derived-table Storybook application |
| `cargo run -p gpui-table-component --bin story --features story` | Open the built-in filter and status-bar stories |
| `cargo run -p mcp-query` | Serve an in-memory table as an MCP query tool |
| `cargo run -p prototyping` | Regenerate table stories from registered metadata |

Start with these sources:

- `some-lib/src/structs/user.rs` for filters, loading, localization,
  and row context menus
- `some-lib/src/structs/item.rs` for loading and custom cells
- `some-lib-tables/src/tables/user.rs` for composing filters,
  `TableStatusBar`, and `DataTable`
- `mcp-query/src/main.rs` for generated MCP query registration
- `prototyping/src/main.rs` for inventory-driven generation

`examples/prototyping/output` is generated. Regenerate it with the
command above rather than editing it.
