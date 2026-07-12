# gpui-table Application Patterns

Load this reference when implementing user-facing application tables, filters, load-more behavior, custom cell rendering, row context menus, localization, or direct filter widgets.

## Feature Flags

```toml
[dependencies]
gpui-table = { version = "*", features = ["fluent", "rust_decimal"] }
gpui-table-component = { version = "*" }
```

- `derive` and `chrono` are default features.
- `rust_decimal` is required for `gpui_table_component::NumberRangeFilter`.
- `fluent` localizes table titles and faceted labels with typed `es-fluent` resources.
- `spacetimedb` enables supported temporal range filtering helpers.
- `inventory` registers `GpuiTableShape` metadata for tooling; filter metadata is exposed through `ComponentShapeUse`.
- `mcp` exposes generated table filters as MCP query tool arguments and implies inventory.

## Basic Derived Table

```rust
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(gpui_table_component::TextFilter))]
    pub name: String,

    #[gpui_table(width = 80., filter(gpui_table_component::NumberRangeFilter))]
    pub age: u8,

    #[gpui_table(width = 120., filter(gpui_table_component::FacetedFilter::<UserStatus>.searchable(true)))]
    pub status: UserStatus,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

With `#[gpui_table(filters)]`, the derive generates:

- `<Row>TableDelegate`
- `<Row>TableColumn`
- `<Row>FilterEntities`
- `<Row>FilterValues`
- `Matchable<<Row>FilterValues>` for strongly typed client-side filtering
- `McpTable` query registration when `gpui-table/mcp` is enabled and the row
  opts in with `#[gpui_table(mcp)]`

## Built-In Filter Shapes

```rust
#[gpui_table(filter(gpui_table_component::TextFilter))]
name: String,

#[gpui_table(filter(gpui_table_component::NumberRangeFilter))]
age: u8,

#[gpui_table(filter(gpui_table_component::DateRangeFilter))]
created_at: chrono::DateTime<chrono::Utc>,

#[gpui_table(filter(gpui_table_component::FacetedFilter::<UserStatus>))]
status: UserStatus,

#[gpui_table(filter(gpui_table_component::FacetedFilter::<UserStatus>.searchable(true)))]
searchable_status: UserStatus,
```

Built-in filters are explicit shape paths or configured shape expressions:
`TextFilter` for strings,
`NumberRangeFilter` for numeric values, `DateRangeFilter` for date-like values,
and `FacetedFilter::<T>` for enum-like fields. Use
`TextFilter.alphabetic_only()`, `TextFilter.numeric_only()`,
`TextFilter.alphanumeric_only()`,
`TextFilter.matching_regex("[A-Z0-9-]*")`, or
`FacetedFilter::<T>.searchable(true)` when generated filter entities should
construct configured built-in filters. Use
`NumberRangeFilter.range(min, max).step(step)` for generated numeric range
filters with explicit slider bounds or step size. Use the same
`filter(path::ToShape)` or configured expression form for custom shapes.

Use `#[derive(Filterable)]` for faceted enums:

```rust
use gpui_component::IconName;
use gpui_table::Filterable;

#[derive(Clone, Eq, Hash, PartialEq, Filterable)]
pub enum Status {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(label = "Needs Review")]
    Pending,
}
```

## Delegate Visibility

Generated delegates keep source rows in `delegate.rows` and expose a filtered
view to `DataTable`. Use `set_filter_values(values)` or `clear_filter_values()`
to control generated client-side filters. Use `set_row_scope(predicate)` for an
additional application-owned predicate and `clear_row_scope()` to remove it;
the scope composes with generated filter values.

`visible_row_indices()` returns indices into the source `rows` vector. Call
`refresh_filtered_rows()` after mutating row values in place when the active
filters or row scope may produce a different result without changing the row
count.

## Fluent Labels

```rust
use es_fluent::{EsFluentLabel, EsFluentVariants};
use gpui_table::{Filterable, GpuiTable};

#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable)]
#[filter(fluent)]
pub enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, EsFluentLabel, EsFluentVariants, GpuiTable)]
#[fluent_variants(keys = ["label"])]
#[gpui_table(fluent = "label", filters)]
pub struct User {
    #[gpui_table(filter(gpui_table_component::FacetedFilter::<UserStatus>))]
    pub status: UserStatus,
}
```

Use the application's existing Fluent resource layout and locale-selection pattern. Keep generated table labels and faceted enum labels in the same localization system as the rest of the app.

## Custom Cell Rendering

Use field-level `style = path::to_fn` when a column needs custom GPUI cell
rendering. The derive keeps generating the table renderer, and fields without a
style hook use the default cell renderer.

```rust
#[derive(gpui_table::GpuiTable)]
pub struct Item {
    pub name: String,

    #[gpui_table(width = 120., style = render_weight_cell)]
    pub weight: u8,
}

fn render_weight_cell(
    row: &Item,
    value: &u8,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> impl gpui::IntoElement {
    use gpui::{ParentElement as _, Styled as _};

    let _ = (row, window, cx);
    gpui::div()
        .child(format!("{value} kg"))
        .px_2()
        .py_0p5()
}
```

## Context Menus

Generated row-context-menu links use:

- `context_menu_row_id = "field_name"` or field-level `#[gpui_table(context_menu_id)]`
- `context_menu_route = "/users/{id}"` or `context_menu_route_fn = path::to_fn`
- `context_menu_label = "Open"` or `context_menu_label_fn = path::to_fn`
- `custom_context_menu` to compose generated links with custom menu items

Prefer route and label helper functions when the app already centralizes routing or translation outside the row type.

## Direct Component Use

Use `gpui-table-component` when composing filter UI manually.

```rust
use gpui::{StyleRefinement, px};
use gpui_table_component::{TableStatusBar, TextFilter, TextFilterExt};

let filter = TextFilter::new("Name", String::new(), move |_value, _window, _cx| {}, cx)
    .alphanumeric_only(cx)
    .container_style(StyleRefinement::default().w_full(), cx)
    .input_style(StyleRefinement::default().w(px(280.)), cx);

let status = TableStatusBar::new(rows.len(), loading, eof).row_label("Rows");
```

## MCP Query Tools

Enable `gpui-table/mcp` when an MCP client should control generated filters and
retrieve rows. Add `#[gpui_table(mcp)]` to each exposed row type and register a
handler with `#[gpui_table::mcp_query]`. A `TableQuery<Row>` first parameter
selects an application-owned backend, while a zero-argument
`Result<Vec<Row>, E>` or `Vec<Row>` return type selects a local row source.
Local row sources are called for each MCP query. For MCP-only filtered tables,
`#[gpui_table(mcp)]` is enough; field-level filter attributes do not also need
struct-level `filters`.

```rust
#[derive(Clone, gpui_table::GpuiTable, serde::Serialize)]
#[gpui_table(mcp)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}

#[gpui_table::mcp_query]
fn rows() -> Vec<User> {
    vec![/* rows */]
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}
```

Add `row_schema` to `mcp(...)` when the row type also implements
`gpui_table::mcp::McpJsonSchema` and MCP clients should discover precise
returned row fields:

```rust
#[derive(
    Clone,
    gpui_table::GpuiTable,
    gpui_table::mcp::McpJsonSchema,
    serde::Serialize,
)]
#[gpui_table(mcp(row_schema))]
struct User {
    name: String,
}
```

With that opt-in, generated query tools publish the row object schema under the
standard output schema's `rows.items`; otherwise row items remain unconstrained
for compatibility with existing serialized row handlers.

Tool arguments use filter field names directly, with `limit` and `offset`
reserved for pagination. Text filters decode from a string, faceted filters
decode from unique `Filterable::to_filter_string()` string sets, and range
filters decode from `{ "min": ..., "max": ... }` objects.
Generated faceted filter schemas include `uniqueItems: true`, valid facet
strings in the item `enum`, and labels in `x-gpuiTableFacetOptions`.
Field-level `#[koruma(...)]` validators on filtered fields validate the decoded
MCP filter argument before the query handler runs. Generated schemas attach rule
metadata in `x-gpuiTableValidation`; literal `LenValidation`,
`RangeValidation`, and `NonEmptyValidation` arguments are also reflected as JSON
Schema hints when the filter argument schema is unambiguous.
For Koruma newtype fields filtered by their inner raw value, derive the adapter
shape with `#[gpui_table_filter_shape(..., koruma_newtype)]`; generated matching
delegates through `NewtypeValue::as_inner`, and MCP validation checks the
decoded raw value with `NewtypeValue::validate_inner`. Manual shapes must
implement `gpui_table::mcp::McpKorumaNewtypeFilterValidation<Field>`. Koruma
annotations on non-filter columns are ignored by table MCP generation. Add
`koruma` and the validator crate that provides the rule to the application
dependencies.
Custom query handlers can be synchronous or async and must return
`Result<gpui_table::mcp::TableQueryResult<Row>, E>`.
Use `query.result(rows, total)` to build the standard response from a decoded
query when the backend owns filtering or totals. Use `query.filter_rows(rows)`
for generated filtering, offset, and limit over an in-memory row source.
Use struct-level `#[gpui_table(mcp(...))]` with `name`, `title`,
`description`, `row_schema`, `read_only`, `destructive`, `idempotent`, and
`open_world` when generated MCP tools need application-owned metadata, precise
row output schemas, or MCP tool annotation hints. If `description` is omitted,
the derive uses the row type's Rust doc comment. Generated table query tools
default to read-only, non-destructive, and idempotent annotations.
`read_only = true` and `destructive = true` cannot be combined.
Use `gpui_table::mcp::server()?` for the default generated server and
`gpui_table::mcp::server_named(name, version)?` when application-owned server
metadata is needed. Use `gpui_table::mcp::builder()` or
`builder_named(name, version)` when deferred builder setup is needed. Use
`gpui_table::mcp::serve_stdio_blocking()` for the default stdio server.
Use `McpServer::builder(name, version)` when composing tables with forms or
other MCP integrations, and add generated handlers with
`.register(gpui_table::mcp::register)`.
Register manual handlers with
`gpui_table::mcp::table::<Row>(&mut server).query(handler)?` for
`Result<gpui_table::mcp::TableQueryResult<Row>, E>` handlers, `.rows(rows)?`,
`.row_source(source)?`, or
`.row_source_async(source)?` for local rows. Manual table tool registration
also publishes that table's descriptor and schema resources. Manual `McpTable`
implementations can call `McpTableDescriptor::with_row_schema(...)` to publish
precise row output schemas. Use
`gpui_table::mcp::register_inventory_table_resources(&mut server)?` when a
composed server should publish inventory-discovered table resources without
their query handlers.
Prompt templates are opt-in: use
`gpui_table::mcp::register_prompt_templates(&mut server)?` for inventory tables
or `register_table_prompt_templates::<Row>(&mut server)?` for one table. The
generated prompt directs clients to the table descriptor and schema resources.
Registration reports setup errors such as duplicate tool names.
MCP schemas and decoders use the same explicit filter shapes selected for
generated filter UI.
For transparent or domain-specific field types that should reuse a built-in raw
value and MCP schema, prefer `TextFilterAdapter`, `NumberRangeFilterAdapter`,
or `DateRangeFilterAdapter` and implement the matching field trait.
For custom filters that adapt an existing built-in shape, derive
`gpui_table::GpuiTableFilterShape` and declare the base shape, raw value, field
type, and raw-value conversions; with the `mcp` feature, the derive also emits
the default `McpFilterShape` decoder when the raw value implements
`gpui_table::mcp::McpToolValue`.
For fully custom runtime filters, implement the runtime shape traits directly,
then derive `gpui_table::McpFilterShape` when `RawValue: McpToolValue` or write
a manual `McpFilterShape` impl when the blanket `McpToolValue` contract is not
the right MCP contract. Use `gpui_table::mcp::McpAny` when a typed raw value or
manual tool input intentionally accepts unconstrained JSON. Use
`gpui_table::mcp::McpRange<T>` for `{ "min": ..., "max": ... }` range raw
values. The `McpJsonSchema` derive supports
named structs, tuple or named transparent newtypes, fieldless enums, and fixed
tuples with 1 to 4 elements; it follows serde deserialize names, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Manual shapes that should support
field-level Koruma filter validation must also implement
`gpui_table::mcp::McpFilterShapeValidation`. Manual shapes that support Koruma
newtype inner-value filters must also implement
`gpui_table::mcp::McpKorumaNewtypeFilterValidation<Field>`.
