# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` is a Rust workspace for building strongly typed GPUI tables.
It combines derive macros, typed filter values, a GPUI runtime layer, UI-neutral
registry metadata, and prototyping/codegen helpers.

The project is organized around three priorities:

1. **Type safety** for generated columns, filters, delegates, and metadata.
1. **Ergonomics** for `#[derive(GpuiTable)]`, `#[derive(Filterable)]`, and `#[gpui_table_impl]`.
1. **Developer experience** for built-in filters, inventory-backed table shapes, and example-driven workflows.

## Installation

```toml
[dependencies]
gpui-table = { version = "*", features = ["fluent", "inventory", "rust_decimal"] }
gpui-table-component = { version = "*" }
```

## Dependency Matrix

| `gpui-table` | `gpui-component` | `gpui` |
| :----------- | :--------------- | :----- |
| **git** | | |
| `branch = "master"` | workspace git dependency | `rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de"` |

`derive` and `chrono` are enabled by default. Add:

- `rust_decimal` for generated numeric range filter values
- `inventory` for `GpuiTableShape` registration and prototyping/codegen
  metadata, including `ComponentShapeUse` filter metadata
- `mcp` for experimental MCP query tools backed by generated typed filter
  values; this also enables inventory registration
- `fluent` for localized table titles and faceted labels through `es-fluent`
- `spacetimedb` for range filtering over supported SpacetimeDB temporal types

## Quick Start

Built-in filters are declared through the component crate, which is a direct
dependency of applications that render the widgets. For example,
`gpui_table_component::NumberRangeFilter` requires generated tables to enable
the `gpui-table` `rust_decimal` feature and the component crate's
`rust_decimal` feature, enabled by default.

```rs
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
        // fetch rows, append them to self.rows, then notify
        cx.notify();
    }
}
```

With `#[gpui_table(filters)]`, the derive also generates:

- `UserTableDelegate` and `UserTableColumn`
- `UserFilterEntities` for rendering built-in filters
- `UserFilterValues` for typed filter state
- `Matchable<UserFilterValues>` so client-side filtering stays strongly typed

The generated `TableRowMeta::TABLE_ID` defaults to the row type name converted to
snake_case, such as `user` for `User` and `purchase_order` for
`PurchaseOrder`. Use `#[gpui_table(id = "...")]` when a table needs a stable
external identifier that does not follow the snake_case Rust type name. Use
`TableRowMeta::table_id()` when callers need the typed `TableId` wrapper instead
of the raw string constant.

Field-level filters require an explicit shape path or configured shape
expression:
`filter(gpui_table_component::TextFilter)` for strings,
`filter(gpui_table_component::NumberRangeFilter)` for numeric ranges,
`filter(gpui_table_component::DateRangeFilter)` for date ranges, and
`filter(gpui_table_component::FacetedFilter::<T>)` for faceted values.
Use `TextFilter.numeric_only()`, `TextFilter.alphanumeric_only()`,
`TextFilter.matching_regex("[A-Z0-9-]*")`, or
`FacetedFilter::<T>.searchable(true)` when the generated filter entity should
construct a configured built-in filter. Use
`NumberRangeFilter.range(min, max).step(step)` to configure generated numeric
range widgets with explicit slider bounds or step size. Use the same forms for
custom shapes whose configured expression implements
`gpui_table::runtime::shape::GpuiTableFilterShapeBuilder<Shape>`.

Faceted filters work with `T`, `Option<T>`, or `Vec<T>` fields when `T`
implements `gpui_table::filter::Filterable`. Optional and vector faceted fields
store selected `T` values in the generated filter state; when a selection is
active, rows with `None` or no matching vector element do not match that facet.

If you enable `inventory`, the same derive registers a `GpuiTableShape` for
tooling and code generation. Filter registrations expose their field, field
type, and resolved base shape path through `ComponentShapeUse`; configured
filter expressions affect generated UI construction but do not change the
registered shape path.

### Custom cell rendering

Use field-level `style = path::to_fn` when a column needs custom GPUI
rendering. The function receives the row, the field value, the GPUI window, and
the app context. It owns the complete cell element for that field; other fields
continue to use the generated default renderer.

```rs
#[derive(Clone, gpui_table::GpuiTable)]
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
        .rounded_md()
        .bg(gpui::rgb(0xeab308))
        .text_color(gpui::white())
}
```

### MCP query tools

With the `mcp` feature, tables that opt in with `#[gpui_table(mcp)]` also get a
`gpui_table::mcp::McpTable` implementation and an MCP tool registration. The
tool accepts structured JSON arguments with generated filter fields plus optional
`limit` and `offset`; for tables without any generated filters, only
pagination arguments are accepted. It decodes filters into the generated
`FilterValues` type and lets an
application-owned handler return rows.

```rs
#[gpui_table::mcp_query]
fn rows() -> Vec<User> {
    vec![/* rows */]
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}
```

Built-in text, faceted, number range, and date range filters decode from the
same values used by generated filter widgets. Faceted MCP arguments use
`Filterable::to_filter_string()` values, for example
`{"status": ["Active"]}`.
Generated schemas publish faceted arguments as unique string sets, include
valid facet values under the filter's item `enum`, and preserve labels in
`x-gpuiTableFacetOptions`, so MCP clients can discover valid facet strings from
`tools/list`.
Field-level `#[koruma(...)]` validators on filtered fields validate the decoded
MCP filter argument before the query handler runs. Generated schemas attach the
rules in `x-gpuiTableValidation`; literal `LenValidation`, `RangeValidation`,
and `NonEmptyValidation` arguments are also reflected as JSON Schema hints when
the filter argument schema is unambiguous. Application crates using these
validators should depend on `koruma` and the validator crate that provides the
rule. For a field whose source type is a Koruma newtype but whose filter
argument should use the inner raw value, derive an adapter shape with
`#[gpui_table_filter_shape(..., field = MyNewtype, koruma_newtype)]`; generated
matching delegates through `NewtypeValue::as_inner`, and MCP validation checks
the decoded raw value with `MyNewtype::validate_inner`. Manual shapes that
support this pattern must implement
`gpui_table::mcp::McpKorumaNewtypeFilterValidation<Field>`. Koruma annotations
on non-filter columns are ignored by table MCP generation.

```rs
use koruma_collection::collection::LenValidation;

#[derive(Clone, Debug, gpui_table::GpuiTable, serde::Serialize)]
#[gpui_table(mcp)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    #[koruma(LenValidation::<_>.min(2).max(64))]
    name: String,
}
```

Use `#[gpui_table::mcp_query]` for both application-owned backend functions
that accept `gpui_table::mcp::TableQuery<User>` and local row sources that
return `Result<Vec<User>, E>` or `Vec<User>`. The row type must opt in with
`#[gpui_table(mcp)]`; for MCP-only tables, field-level filter attributes do not
also need struct-level `filters`. The handler signature chooses the mode, and
local sources are called for each MCP query. Custom query handlers can be
synchronous or async. Return
`Result<gpui_table::mcp::TableQueryResult<User>, E>` for explicit MCP errors,
where `User: serde::Serialize`. Use
`query.result(rows, total)` to build the standard response from a decoded query.
Use struct-level
`#[gpui_table(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. When
`description` is omitted, the derive uses the row type's Rust doc comment.
Add `row_schema` in the same list when the row type also implements
`gpui_table::mcp::McpJsonSchema`; the generated tool output schema then
publishes the exact row object under `rows.items` instead of the default
unconstrained item schema.
The same list accepts optional MCP tool annotation hints with
`read_only = ...`, `destructive = ...`, `idempotent = ...`, and
`open_world = ...`. Generated table query tools default to read-only,
non-destructive, and idempotent annotations. `read_only = true` and
`destructive = true` cannot be combined.
The lower-level `McpServer` API remains available when an
application wants to compose table tools with other `component-shape-mcp`
integrations. Use `gpui_table::mcp::server_named(name, version)?` when generated
table handlers should advertise application-owned metadata, then call
`.serve_stdio().await` or `.serve_stdio_blocking()`. Use
`gpui_table::mcp::builder()` or `builder_named(name, version)` when deferred
setup is useful. Use `gpui_table::mcp::serve_stdio_blocking()` for the default
stdio server. Generated registration also exposes JSON resources for each
`#[gpui_table(mcp)]` table at
`gpui-table://tables/{tool_name}/descriptor` and
`gpui-table://tables/{tool_name}/schema`. The descriptor resource includes
table metadata, filter field types, normalized component-shape MCP input
metadata such as scalar, set, or range, validation rules, per-filter schemas,
and the table query output schema. Use `gpui_table::mcp::register_inventory_table_resources(&mut server)?`
when a composed server should publish inventory-discovered table resources
without registering their query handlers.

For a composed server, such as a binary that also depends on `gpui-form`, use
the shared builder:

```rs
let server = gpui_table::mcp::McpServer::builder("my-app", env!("CARGO_PKG_VERSION"))
    .register(gpui_table::mcp::register)
    .register(gpui_form::mcp::register)
    .build()?;
```

Manual table tools can still be registered directly:
`gpui_table::mcp::table::<User>(&mut server).query(handler)?` for
`Result<gpui_table::mcp::TableQueryResult<User>, E>` handlers,
`.rows(rows)?` for fixed row vectors,
`.row_source(source)?` for per-query local rows, or
`.row_source_async(source)?` for async local row sources.
Manual table tool registration also publishes that table's descriptor and schema
resources. Manual `McpTable` implementations can call
`McpTableDescriptor::with_row_schema(...)` to publish precise row output
schemas. Registration reports setup errors such as duplicate tool names.
For transparent or domain-specific field types that should reuse a built-in
filter widget, prefer the adapter shapes:
`gpui_table_component::TextFilterAdapter`,
`NumberRangeFilterAdapter`, and `DateRangeFilterAdapter`. Implement the
matching field trait, such as `TextFilterField`, and the adapter supports both
`T` and `Option<T>` fields while reusing the built-in raw value and MCP schema.
For custom filters that adapt an existing built-in shape, derive
`gpui_table::GpuiTableFilterShape` and declare a base shape, raw value, field
type, and raw-value conversions. The derive generates the runtime filter shape,
declared-shape markers, field-support impl, and, with the `mcp` feature, the
default `McpFilterShape` decoder when the raw value implements
`gpui_table::mcp::McpToolValue`. Add `koruma_newtype` when the shape adapts a
base filter over a Koruma newtype field's inner value. For fully custom runtime filters, implement
the runtime shape traits directly, then derive `gpui_table::McpFilterShape`
when `RawValue: McpToolValue` or implement `gpui_table::mcp::McpFilterShape`
manually for custom schema/decoding. Use `gpui_table::mcp::McpAny` when a
typed raw value or manual tool input intentionally accepts unconstrained JSON,
and `gpui_table::mcp::McpRange<T>` for custom `{ "min": ..., "max": ... }`
range raw values. The `McpJsonSchema` derive follows
serde deserialize names, records field aliases in `x-mcpAliases`, includes enum aliases, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Manual shapes that should support
field-level Koruma filter validation must also implement
`gpui_table::mcp::McpFilterShapeValidation`. Fixed tuples with 1 to 4 elements
publish exact array schemas; app-owned named structs, transparent newtypes, and
fieldless enums can derive it. Custom top-level MCP tool argument structs can
also derive `gpui_table::mcp::McpToolInput` through the facade when composing
manual typed tools; that derive also implements `McpJsonSchema`, so object
inputs can be reused as field or filter values.

### Table cells for value objects

Single-field wrappers render by delegating to their inner value by default.
When a wrapper should render through its own display implementation or a
formatter, use `#[table_cell(display)]` or `#[table_cell(format = ...)]`.

```rs
use gpui_table::TableCell;
use std::fmt;

#[derive(TableCell)]
#[table_cell(display)]
pub struct AccountCode(String);

impl fmt::Display for AccountCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

fn format_percentage(value: &Percentage) -> String {
    format!("{}%", value.0)
}

#[derive(TableCell)]
#[table_cell(format = format_percentage)]
pub struct Percentage(i64);
```

### Localized labels

With the `fluent` feature, table titles and faceted labels are localized through
typed `es-fluent` messages and labels.

```rs
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
    #[gpui_table(filter(gpui_table_component::FacetedFilter::<UserStatus>.searchable(true)))]
    pub status: UserStatus,
}
```

The built-in table/filter widgets keep their embedded `es-fluent` localizer in
GPUI global state. The example app declares its languages in a library-reachable
`i18n` module with `#[es_fluent_language]`, initializes
`gpui_table_component::i18n` during GPUI startup, and selects the active locale
through the GPUI storybook locale APIs.
Generated Storybook table titles use the GPUI app context so they follow the
active Storybook locale; context-free metadata uses fallback label helpers.

## Examples

The canonical end-to-end examples live under [`examples/`](../../examples/README.md).

- `cargo run`
  Launches `examples/some-lib-tables`, the storybook app for the derived tables.
- `cargo run -p prototyping`
  Regenerates `examples/prototyping/output` from `GpuiTableShape` inventory registrations.
- `cargo run -p mcp-query`
  Starts a stdio MCP proof-of-concept that queries in-memory table rows.
- From the sibling `gpui-form` workspace, `cargo run -p mcp-form-table`
  runs a composed form-submit plus table-query MCP server with custom shapes.

The main walkthrough files are:

- `examples/some-lib/src/structs/user.rs` for derived filters, localized titles, and context menus
- `examples/some-lib/src/structs/item.rs` for load-more and custom cell rendering
- `examples/mcp-query/src/main.rs` for MCP tools that control generated filters and return rows
- `../gpui-form/examples/mcp-form-table/src/main.rs` for a composed
  `gpui-form` plus `gpui-table` MCP server with custom shapes
- `examples/prototyping/src/main.rs` for inventory-driven code generation

## Feature Flags

- `derive` (default): re-exports `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`
- `chrono` (default): date cell rendering and generated date-range filter values
- `fluent`: localized titles and faceted labels through `es-fluent`
- `inventory`: inventory-backed `GpuiTableShape` registration for tooling,
  including `ComponentShapeUse` filter metadata
- `mcp`: experimental stdio MCP query integration through generated
  `McpTable` implementations; implies `inventory`
- Built-in filter MCP decoding lives behind `gpui-table-component`'s `mcp`
  feature, keeping this facade contract-only.
- `rust_decimal`: numeric range filtering and decimal-backed helpers
- `spacetimedb`: SpacetimeDB temporal range filtering support
