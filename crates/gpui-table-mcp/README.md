# gpui-table-mcp

`gpui-table-mcp` is the experimental MCP integration crate for exposing
generated `gpui-table` filters as query tools.

Most GPUI application code should start with `gpui-table`. Enable this crate
through the facade when you want a stdio MCP server or custom MCP tool server:

```toml
[dependencies]
gpui-table = { version = "*", features = ["mcp"] }
```

The derive macro emits a `gpui_table::mcp::McpTable` implementation for tables
that opt in with `#[gpui_table(mcp)]`. The MCP tool accepts generated filter
fields plus pagination arguments:

```json
{
  "name": "ann",
  "status": ["Active"],
  "created_on": { "min": "2026-01-01", "max": "2026-12-31" },
  "limit": 25,
  "offset": 0
}
```

Applications own execution. Use `#[gpui_table::mcp_query]` for both custom
backends and local data sources. The inferred row type must opt in with
`#[gpui_table(mcp)]`. The macro infers the row type from common handler
signatures:

```rs
#[gpui_table::mcp_query]
fn rows() -> Vec<UserRow> {
    vec![/* rows */]
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}
```

Query tools keep `Row: serde::Serialize` as the baseline. Add `row_schema`
when the row type also implements `McpJsonSchema` and clients should discover
the exact shape of returned rows from `tools/list` and descriptor resources:

```rs
#[derive(gpui_table::GpuiTable, gpui_table::mcp::McpJsonSchema, serde::Serialize)]
#[gpui_table(mcp(row_schema))]
struct UserRow {
    name: String,
}
```

With that opt-in, the standard table query output schema remains an object and
publishes the row schema under `properties.rows.items`; without it,
`rows.items` stays `{}` for compatibility with existing serialized row
handlers.

Built-in text, faceted, number range, and date range filters are supported.
Declare the same explicit `#[gpui_table(filter(path::ToShape))]` shape paths
used by generated filter UI before the MCP descriptor is generated. Faceted
filter schemas publish unique string sets, include valid
`Filterable::to_filter_string()` values in the item `enum`, and labels in
`x-gpuiTableFacetOptions`.
Field-level `#[koruma(...)]` validators on filtered fields validate the decoded
MCP filter argument before the query handler runs. Generated schemas attach
rule metadata in `x-gpuiTableValidation`; literal `LenValidation`,
`RangeValidation`, and `NonEmptyValidation` arguments are reflected as JSON
Schema hints when the filter argument schema is unambiguous. Application crates
using these validators should depend on `koruma` and the validator crate that
provides the rule.
For custom filters that adapt an existing built-in shape, derive
`gpui_table::GpuiTableFilterShape` and declare the base shape, raw value, field
type, and raw-value conversions:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
struct PrefixText(String);

#[derive(gpui_table::GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;
```

The derive generates the runtime `GpuiTableFilterShape` implementation,
declared-shape markers, `GpuiTableFilterShapeFor<String>`, and, with the
`mcp` feature, the default `McpFilterShape` decoder when `PrefixText`
implements `McpToolValue`.
For fully custom runtime filters, implement the runtime shape traits directly,
then derive `gpui_table::McpFilterShape` when the raw filter value implements
`McpToolValue`; the blanket implementation covers `Deserialize` raw values
that implement or derive `McpJsonSchema`. Use `McpAny` when a typed raw value
or manual tool input intentionally accepts unconstrained JSON. Use
`McpRange<T>` as the raw value for custom `{ "min": ..., "max": ... }` range
arguments. Implement `McpFilterShape` manually when a custom shape needs richer
schema or decoding than the blanket `McpToolValue` contract. Manual shapes that
should support field-level Koruma filter validation must also implement
`gpui_table::mcp::McpFilterShapeValidation`. The
`McpJsonSchema` derive follows serde deserialize names, includes enum
aliases, records field aliases in `x-mcpAliases`, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Fixed tuples with 1 to 4 elements
publish exact array schemas; app-owned named structs, tuple or named
transparent newtypes, and fieldless enums can derive it. Custom top-level MCP
tool argument structs can also derive `gpui_table::mcp::McpToolInput` through
the facade when composing manual typed tools; that derive also implements
`McpJsonSchema`, so object inputs can be reused as field or filter values.
The lower-level `McpServer` API remains available for custom server
composition. `McpServer` is the shared `component-shape-mcp` server, so table
query tools can be served beside form submit tools in a binary that also
depends on `gpui-form`:

```rust
let server = gpui_table::mcp::McpServer::builder("my-app", env!("CARGO_PKG_VERSION"))
    .register(gpui_table::mcp::register)
    .register(gpui_form::mcp::register)
    .build()?;
```

Generated registration also exposes JSON resources for every
`#[gpui_table(mcp)]` table:

- `gpui-table://tables/{tool_name}/descriptor`
- `gpui-table://tables/{tool_name}/schema`

The descriptor resource includes table metadata, filter field types, normalized
component-shape MCP input metadata such as scalar, set, or range, validation
rules, per-filter schemas, the table query output schema, and the row schema
when one is configured. Use
`register_inventory_table_resources(&mut server)?` when a custom server should
expose resources for all inventory-discovered tables without registering query
handlers, or `register_table_resources::<Row>(&mut server)?` for one table.
Prompt templates are opt-in. Call
`gpui_table::mcp::register_prompt_templates(&mut server)?` after inventory
registration, or `register_table_prompt_templates::<Row>(&mut server)?` when
manually exposing one table. The generated prompt name is
`query_{tool_name}_table`; it directs clients to the descriptor and schema
resources before drafting filter and pagination arguments for the query tool.

Attribute handlers infer the row type from a `TableQuery<Row>` first parameter,
a zero-argument `Result<Vec<Row>, E>` or `Vec<Row>` source. Local sources are
called for each MCP query. `#[gpui_table(mcp)]` is enough for MCP-only filtered
tables; field-level filter attributes do not also need struct-level `filters`.
Custom query handlers can be synchronous or async and return
`Result<gpui_table::mcp::TableQueryResult<Row>, E>` where `Row: serde::Serialize`.
Use
`query.result(rows, total)` for backend-owned totals or
`query.filter_rows(rows)` for local filtering and paging. Manual `McpServer`
composition uses
`table::<Row>(&mut server).query(handler)?` for
`Result<gpui_table::mcp::TableQueryResult<Row>, E>` handlers, plus
`.rows(rows)?`, `.row_source(source)?`, or `.row_source_async(source)?` for local
rows. Manual table tool registration also publishes that table's descriptor and
schema resources. Manual `McpTable` implementations can call
`McpTableDescriptor::with_row_schema(...)` to publish precise row output
schemas.
Use struct-level
`#[gpui_table(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. When
`description` is omitted, the derive uses the row type's Rust doc comment.
The same list accepts optional MCP tool annotation hints with
`read_only = ...`, `destructive = ...`, `idempotent = ...`, and
`open_world = ...`. Generated table query tools default to read-only,
non-destructive, and idempotent annotations. `read_only = true` and
`destructive = true` cannot be combined.
Registration reports setup errors such as duplicate tool names. Use
`gpui_table::mcp::server()?` for the default generated server or
`gpui_table::mcp::server_named(name, version)?` when generated table handlers
should advertise application-owned metadata. Use `gpui_table::mcp::builder()`
or `builder_named(name, version)` when callers need the shared builder for
deferred setup. Use `gpui_table::mcp::serve_stdio_blocking()` for the default
stdio server.
