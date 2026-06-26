# Examples

`examples/some-lib` and `examples/some-lib-tables` are the canonical end-to-end
examples for the workspace. They show the derive flow that most application
code should start with.

## Recommended Runs

### Launch the storybook app

```sh
cargo run
```

From the workspace root, this starts `examples/some-lib-tables`, which is also
the default workspace member.

Use it to see:

- derived columns rendered in `gpui_component::table::DataTable`
- generated built-in filters wired with `UserFilterEntities::build_for_table(...)`
- `TableStatusBar` and filter layout composition
- typed `es-fluent` titles, descriptions, and faceted labels
- field-level custom cell rendering and load-more behavior
- optional row-context-menu routing when `some-lib-tables` is run with `--features router`

### Regenerate prototyping output

```sh
cargo run -p prototyping
```

This iterates the inventory-registered `GpuiTableShape` values, including
`ComponentShapeUse` filter metadata, and rewrites
`examples/prototyping/output`. Generated Storybook table titles use the active
example app locale.

Do not hand-edit `examples/prototyping/output`; it is generated output.

### Query rows through MCP

```sh
cargo run -p mcp-query
```

This starts a stdio MCP server that exposes a derived table as a query tool.
The tool accepts JSON filters, `limit`, and `offset`, decodes them into the
generated typed filter values, and returns matching in-memory rows. The table
declares explicit MCP tool metadata with `#[gpui_table(mcp(...))]` and uses
inferred built-in filters. Filtered fields can use `#[koruma(...)]` to validate
decoded MCP filter arguments before the query handler runs. Custom filters that
adapt built-in shapes can use
`#[derive(gpui_table::GpuiTableFilterShape)]`; see the `gpui-table-mcp`
README for the adapter pattern.

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | cargo run -q -p mcp-query
```

## Workspace Layout

- `examples/some-lib`
  Domain types, `#[derive(GpuiTable)]` rows, `#[derive(Filterable)]` enums, and package-local i18n setup.
- `examples/some-lib-tables`
  Storybook-style GPUI app that renders the generated tables and filters.
- `examples/mcp-query`
  Stdio MCP query proof-of-concept backed by generated table filters.
- `../gpui-form/examples/mcp-form-table`
  Sibling-workspace composed MCP example with a custom form component shape and
  inferred table filter MCP arguments.
- `examples/prototyping`
  Inventory-driven generator that writes story modules into `examples/prototyping/output`.

## Fluent Setup

The examples use `es-fluent` typed messages directly: row structs derive
`EsFluentLabel`/`EsFluentVariants`, faceted enums derive `EsFluent`, and table
attributes opt into localized labels.

```rs
#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, gpui_table::Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    Active,
    Suspended,
}
```

`examples/some-lib/i18n.toml` lives next to that package's `Cargo.toml`, and
`examples/some-lib/src/i18n.rs` declares both the embedded resources and the
app language enum with `#[es_fluent_language]`. The storybook binary imports
`some_lib::i18n::Languages` and selects the storybook locale before rendering.

## Files To Read First

- `examples/some-lib/src/structs/user.rs`
  Generated filters, localized titles, faceted enums, and custom context-menu composition.
- `examples/some-lib/src/structs/item.rs`
  Load-more wiring via `#[gpui_table_impl]` plus field-level cell style functions.
- `examples/some-lib-tables/src/tables/user.rs`
  How generated filters are composed into a screen with `DataTable`.
- `examples/mcp-query/src/main.rs`
  How `#[gpui_table::mcp_query]` exposes generated filters as MCP tool arguments and row output schemas.
- `examples/prototyping/src/main.rs`
  A complete generator built on `TableShapeAdapter`, `TableLayout`, and `TableParts`.

## Notes

- If you change derive behavior, keep these examples aligned with the public README surfaces in the same change.
- If inventory or codegen behavior changes, rerun `cargo run -p prototyping` so `examples/prototyping/output` stays in sync.
