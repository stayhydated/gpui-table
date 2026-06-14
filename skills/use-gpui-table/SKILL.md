---
name: use-gpui-table
description: >-
  Build or extend user-facing Rust GPUI application tables with gpui-table. Use
  when application code needs #[derive(GpuiTable)], #[derive(Filterable)],
  #[derive(TableCell)], #[gpui_table_impl], generated table delegates, columns,
  filter entities, and filter values, built-in filters such as text, faceted,
  number_range, or date_range, TableStatusBar, load-more behavior, custom row
  rendering, row context menus, localized labels with fluent, MCP query tools,
  or feature flags such as mcp, rust_decimal, chrono, or spacetimedb.
---

# Use GPUI Table

## Scope Boundary

Use this skill for user-facing application workflows with `gpui-table`:
deriving typed tables, generated delegates and filters, built-in filter widgets,
`TableStatusBar`, load-more behavior, custom row rendering, row context menus,
localization, and feature selection.
Use `gpui-table/mcp` only when row data should be exposed to MCP clients as
query tools; generated filters become typed arguments when present, and
filterless tables expose pagination only.

This skill is for application code that consumes `gpui-table`. It avoids
repository maintenance, release, generated-output, and implementation-internal
guidance.

## Core Workflow

Start from the user-facing facade. Most application code uses `gpui-table` for
derives, generated types, and runtime helpers:

1. Enable the smallest feature set needed. `derive` and `chrono` are default
   features, `rust_decimal` supports numeric range filters,
   `fluent` supports localized labels through `es-fluent`, and `spacetimedb`
   supports SpacetimeDB temporal range filtering. Enable `mcp` only for MCP
   query tools backed by row data.
2. Define row structs with `#[derive(Clone, GpuiTable)]`.
3. Add field-level `#[gpui_table(...)]` attributes for widths, sorting,
   movement, resizing, filters, skipped fields, context menu ids, or generated
   context menu behavior. Prefer bare `#[gpui_table(filter)]` when a built-in
   filter can be inferred from the field type.
4. Use `#[derive(Filterable)]` for faceted enums. Include
   `Clone + Eq + Hash + PartialEq`; add `#[filter(fluent)]` only when labels
   come from `es-fluent`.
5. Use `#[derive(TableCell)]` for value objects. Add `#[table_cell(display)]`
   when the wrapper should render through its own `Display` implementation, or
   `#[table_cell(format = path::to::formatter)]` for a dedicated formatter.
6. Add `#[gpui_table(filters)]` when generated filter entities and typed filter
   state are needed.
7. Add `#[gpui_table(load_more)]` plus `#[gpui_table::gpui_table_impl] impl
   TableLoader for <Row>TableDelegate` for infinite-loading tables.
8. Add `#[gpui_table(custom_style)]` and implement `TableRowStyle` when a column
   needs custom rendering. Delegate unchanged columns to
   `gpui_table::runtime::default_render_cell`.
9. Compose generated tables with `gpui_component::table::DataTable` and
   generated filter helpers, or use `gpui-table-component` directly when manual
   filter UI composition is a better fit.
10. For MCP query integrations, register the generated row type with
    `#[gpui_table(mcp)]` on the row type and `#[gpui_table::mcp_query]` on the
    handler. A `TableQuery<Row>` first parameter selects a custom backend, while
    a zero-argument `Result<Vec<Row>, E>` return type selects a local row
    source. Keep query execution in application-owned code rather than GPUI
    widgets. Custom query handlers can be synchronous or async and must return
    `Result<gpui_table::mcp::TableQueryResult<Row>, E>`.

## Reference Selection

Load only the reference needed for the task:

- `references/patterns.md`: table derives, filters, load-more behavior, row rendering, row context menus, localization, feature flags, and direct filter widgets.

Prefer current public docs or source examples over memory when details matter.

## Implementation Rules

Use `gpui-table` for normal strongly typed GPUI tables. It re-exports the core
and runtime namespaces and, with the default `derive` feature, the proc macros.

Use `gpui-table-component` when the app needs direct GPUI filter widget
composition, `ResetFilters`, `TableStatusBar`, or `QueryFilterValue`.

Use `gpui_table::mcp` when an MCP client should control generated filters and
retrieve rows. Add `#[gpui_table(mcp)]` to each exposed row type. The generated
tool accepts filter field names directly, with `limit` and `offset` reserved for
pagination; filter arguments decode into the generated `<Row>FilterValues`
type. Tables without generated filters accept only pagination arguments. Faceted
filter schemas publish valid facet strings and labels for MCP clients. Custom
query handlers can be synchronous or async and must return
`Result<gpui_table::mcp::TableQueryResult<Row>, E>`.
Use `query.result(rows, total)` to build the standard response from a decoded
query. Use `McpServer` directly only when a custom server composition is
needed. Use `gpui_table::mcp::server()?` for the default generated server and
`gpui_table::mcp::server_named(name, version)?` when application-owned server
metadata is needed. Use `gpui_table::mcp::builder()` or
`builder_named(name, version)` when deferred builder setup is needed. Use
`gpui_table::mcp::serve_stdio_blocking()` for the default stdio server. Use
`McpServer::builder(name, version)` when composing tables with forms or other
MCP integrations, and add generated handlers with
`.register(gpui_table::mcp::register)`. Register manual handlers
with `gpui_table::mcp::table::<Row>(&mut server).query(handler)?` for
`Result<gpui_table::mcp::TableQueryResult<Row>, E>` handlers, `.rows(rows)?`,
`.row_source(source)?`, or
`.row_source_async(source)?` for local rows. Use struct-level
`#[gpui_table(mcp(...))]` with `name`, `title`, `description`, `read_only`,
`destructive`, `idempotent`, and `open_world` when generated MCP tools need
application-owned metadata or MCP tool annotation hints. If `description` is
omitted, the derive uses the row type's Rust doc comment. Generated table query
tools default to read-only, non-destructive, and idempotent annotations.
`read_only = true` and `destructive = true` cannot be combined.
Registration reports setup errors such as duplicate tool names. Bare `#[gpui_table(filter)]`
infers the MCP filter schema and decoder through the same shape selected for
normal generated filters. For custom filters that adapt an existing built-in
shape, derive `gpui_table::GpuiTableFilterShape` and declare the base shape,
raw value, field type, and raw-value conversions; with the `mcp` feature, the
derive also emits the default `McpFilterShape` decoder when the raw value
implements `gpui_table::mcp::McpToolValue`. For fully custom runtime filters,
implement the runtime shape traits directly, then derive
`gpui_table::McpFilterShape` when `RawValue: McpToolValue` or write a manual
`McpFilterShape` impl when the blanket `McpToolValue` contract is not the
right MCP contract. Use `gpui_table::mcp::McpAny` when a typed raw value or
manual tool input intentionally accepts unconstrained JSON. Use
`gpui_table::mcp::McpRange<T>` for `{ "min": ..., "max": ... }` range raw
values.
Fixed tuples with 1 to 4 elements publish exact array schemas.
App-owned named structs, tuple or named transparent newtypes, and fieldless
enums can derive `McpJsonSchema`; the derive follows serde deserialize names,
records field aliases in `x-mcpAliases`, includes enum aliases, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Custom top-level MCP tool inputs can
derive `gpui_table::mcp::McpToolInput`; that derive also implements
`McpJsonSchema`, so object inputs can be reused as field or filter values.

Generated names follow the row type:

- `<Row>TableDelegate`
- `<Row>TableColumn`
- `<Row>FilterEntities`
- `<Row>FilterValues`

Use built-in filters through bare field attributes when they match the
application workflow:

- `filter` on strings infers text search.
- `filter` on enum-like `T`, `Option<T>`, or `Vec<T>` fields infers
  `FacetedFilter::<T>` when `T` derives or implements `Filterable`.
- `filter` on numeric values infers numeric ranges.
- `filter` on date-like values infers temporal ranges.

Use `filter(path::ToShape)` when a field needs a custom shape or when inference
would choose the wrong built-in shape.

Keep localized labels explicit. Use `#[filter(fluent)]` or the matching table
label attributes only when the application owns an `es-fluent` localizer and the
labels are rendered through that context.
