# MCP query tools

The experimental `mcp` feature exposes application-owned rows through
typed MCP query tools. Generated code validates and decodes filter and
pagination arguments; the application still owns data access, authorization,
filter execution, and totals.

```toml
[dependencies]
gpui-table = { version = "0.6", features = ["mcp"] }
gpui-table-component = { version = "0.6", features = ["mcp"] }
serde = { version = "1", features = ["derive"] }
```

Enable `gpui-table-component/mcp` when a table uses the built-in
filter shapes. Match the facade and component `rust_decimal`,
`chrono`, or `spacetimedb` features required by those
shapes.

## Register a local row source

Opt the row into MCP metadata, register a row source, and serve the
inventory-backed tools over stdio:

```rust,ignore
#[derive(
    Clone,
    gpui_table::GpuiTable,
    gpui_table::mcp::McpJsonSchema,
    serde::Serialize,
)]
#[gpui_table(mcp(row_schema))]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[gpui_table::mcp_query]
fn rows() -> Vec<User> {
    load_users()
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}
```

`#[gpui_table(mcp)]` is sufficient for an MCP-only filtered table;
it does not also need struct-level `filters`. A zero-argument
`Vec<Row>` or `Result<Vec<Row>, E>` handler is called for
each query. The generated tool accepts the declared filter field names plus
optional `limit` and `offset`.

Rows must implement `serde::Serialize`. The `row_schema`
option additionally requires `McpJsonSchema` and publishes the
precise row object beneath `rows.items`. Without that option, row
items remain unconstrained in the output schema.

## Use an application backend

A handler whose first argument is `TableQuery<Row>` receives decoded
arguments for backend-owned execution:

```rust,ignore
#[gpui_table::mcp_query]
async fn rows(
    query: gpui_table::mcp::TableQuery<User>,
) -> Result<gpui_table::mcp::TableQueryResult<User>, QueryError> {
    let (rows, total) = query_users(&query).await?;
    Ok(query.result(rows, total))
}
```

The backend must apply the decoded query and return the total number of matching
rows with the requested page. Use `query.filter_rows(rows)` instead
when the application supplies an in-memory collection and wants generated
filtering, offset, and limit.

## Describe the tool

Set generated tool metadata on the row:

```rust,ignore
#[gpui_table(mcp(
    row_schema,
    name = "query_users",
    title = "Query users",
    description = "Query users visible to the current application.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false
))]
struct User {
    // fields
}
```

When `description` is omitted, the row type's Rust doc comment is
used. Generated query tools default to read-only, non-destructive, and
idempotent annotations. `read_only = true` and
`destructive = true` cannot be combined.

Faceted arguments are unique sets of
`Filterable::to_filter_string()` values. Their schemas include the
valid strings and display labels. Range arguments use
`{ "min": ..., "max": ... }` objects.

Field-level `#[koruma(...)]` validators on filtered fields run after
decoding and before the handler. Add `koruma` and the crate that owns
the selected validator to the application dependencies. For a filter over a
Koruma newtype's inner value, use a shape derived with
`koruma_newtype`; see
[Filter components and custom shapes](custom_filters.md#adapt-an-existing-shape).

## Compose a server

`gpui_table::mcp::server()?` registers inventory-discovered table
handlers with default package metadata. Use
`server_named(name, version)?` for application-owned metadata, or the
builder helpers when setup is deferred.

Use the shared `McpServer` builder to combine table query tools with
other component-shape integrations:

```rust,ignore
let server = gpui_table::mcp::McpServer::builder(
    "my-app",
    env!("CARGO_PKG_VERSION"),
)
.register(gpui_table::mcp::register)
.register(gpui_form::mcp::register)
.build()?;
```

Manual table registration supports backend handlers, fixed rows, synchronous row
sources, and asynchronous row sources through
`table::<Row>(&mut server)`.

Every registered table publishes descriptor and schema resources:

- `gpui-table://tables/{tool_name}/descriptor`
- `gpui-table://tables/{tool_name}/schema`

Use `register_inventory_table_resources` when a composed server
should publish inventory-discovered resources without registering their query
handlers. Query prompt templates are opt-in through
`register_prompt_templates` for inventory tables or
`register_table_prompt_templates::<Row>` for one table.

## Verify the tool

In a repository checkout, run:

```sh
cargo run -p mcp-query
```

The example exposes `mcp_query_issues` and returns `rows`,
`total`, `offset`, and `limit`. If registration
fails, check for duplicate tool names. If a built-in filter shape fails its MCP
trait bound, enable the component crate's `mcp` feature in addition
to the facade feature.
