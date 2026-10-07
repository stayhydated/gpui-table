# gpui-table examples

These runnable workspace packages demonstrate derived GPUI tables, built-in
components, MCP queries, and inventory-driven table generation.

Run these commands from the workspace root:

| Command | Purpose |
|---|---|
| `cargo run` | Open the derived-table Storybook application |
| `cargo run -p gpui-table-component --bin story --features story` | Open the built-in filter and status-bar stories |
| `cargo run -p gpui-table --example ordered_sort --features spacetimedb,mcp,rust_decimal` | Verify temporal sorting, calculated keys and MCP pagination |
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

`crates/gpui-table/examples/ordered_sort.rs` shares one calculated key between
ordering and cell presentation using real SpacetimeDB temporal values.
