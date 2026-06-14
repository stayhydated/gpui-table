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
```

## Dependency Matrix

| `gpui-table` | `gpui-component` | `gpui` |
| :----------- | :--------------- | :----- |
| **git** | | |
| `branch = "master"` | workspace git dependency | `rev = "b077f41a9f26ae5ed7fadfea55a501d34afb25de"` |

`derive` and `chrono` are enabled by default. Add:

- `rust_decimal` for numeric range filters
- `inventory` for `GpuiTableShape` registration and prototyping/codegen
  metadata, including `ComponentShapeUse` filter metadata
- `mcp` for experimental MCP query tools backed by generated typed filter
  values; this also enables inventory registration
- `fluent` for localized table titles and faceted labels through `es-fluent`
- `spacetimedb` for range filtering over supported SpacetimeDB temporal types

## Quick Start

`gpui_table_component::NumberRangeFilter` requires the `rust_decimal` feature.

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
    #[gpui_table(sortable, width = 160., filter)]
    pub name: String,

    #[gpui_table(width = 80., filter)]
    pub age: u8,

    #[gpui_table(width = 120., filter)]
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

Bare `#[gpui_table(filter)]` infers common filter shapes from the field type:
strings use `TextFilter`, numbers use `NumberRangeFilter`, date-like values use
`DateRangeFilter`, and other enum-like values use `FacetedFilter::<T>`.
Use `filter(path::ToShape)` when a field needs a custom or non-inferred shape.

Faceted filters work with `T`, `Option<T>`, or `Vec<T>` fields when `T`
implements `gpui_table::filter::Filterable`. Optional and vector faceted fields
store selected `T` values in the generated filter state; when a selection is
active, rows with `None` or no matching vector element do not match that facet.

If you enable `inventory`, the same derive registers a `GpuiTableShape` for
tooling and code generation. Filter registrations expose their field, field
type, and shape path through `ComponentShapeUse`.

### MCP query tools

With the `mcp` feature, tables that opt in with `#[gpui_table(mcp)]` get a
`gpui_table::mcp::McpTable` implementation and an MCP tool registration. The
tool accepts structured JSON arguments with generated filter fields plus
optional `limit` and `offset`; for tables without filters, only pagination
arguments are accepted. It decodes filters into the generated `FilterValues`
type and lets an application-owned handler return rows.

```rs
#[gpui_table::mcp_query]
fn rows() -> Result<Vec<User>, String> {
    Ok(vec![/* rows */])
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

Use `#[gpui_table::mcp_query]` for both application-owned backend functions
that accept `gpui_table::mcp::TableQuery<User>` and local row sources that
return `Result<Vec<User>, E>`. The row type must opt in with
`#[gpui_table(mcp)]`. The handler signature chooses the mode, and local
sources are called for each MCP query. The custom query parameter and return
value must use the same row type, so custom backends must return
`Result<gpui_table::mcp::TableQueryResult<User>, E>`, and `User` must implement
`serde::Serialize`. Custom query handlers can be synchronous or async.
Use
`query.result(rows, total)` to build the standard response from a decoded
query. Use struct-level
`#[gpui_table(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. When
`description` is omitted, the derive uses the row type's Rust doc comment.
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
stdio server.

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
`Result<gpui_table::mcp::TableQueryResult<User>, E>` handlers with
`User: serde::Serialize`, `.rows(rows)?` for fixed row vectors,
`.row_source(source)?` or `.row_source_async(source)?` for per-query local rows.
Registration reports setup errors such as duplicate tool names.
Custom filter shapes can derive `gpui_table::McpFilterShape` when their
`RawValue` implements `gpui_table::mcp::McpToolValue`; the blanket
implementation covers `Deserialize` raw values that implement or derive
`gpui_table::mcp::McpJsonSchema`; use `gpui_table::mcp::McpAny` when a typed
raw value or manual tool input intentionally accepts unconstrained JSON. The
derive decodes the raw value and wraps it with the table filter shape contract.
Use `gpui_table::mcp::McpRange<T>` for custom `{ "min": ..., "max": ... }`
range raw values. Implement
`gpui_table::mcp::McpFilterShape` manually when a custom filter needs richer
schema or decoding than the blanket `McpToolValue` contract. The `McpJsonSchema` derive follows
serde deserialize names, records field aliases in `x-mcpAliases`, includes enum aliases, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Fixed tuples with 1 to 4 elements
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
    #[gpui_table(filter)]
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

The canonical end-to-end examples live under [`examples/`](examples/README.md).

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
- `examples/some-lib/src/structs/item.rs` for load-more and custom row rendering
- `examples/mcp-query/src/main.rs` for MCP tools that control generated filters and return rows
- `../gpui-form/examples/mcp-form-table/src/main.rs` for a composed
  `gpui-form` plus `gpui-table` MCP server with custom shapes
- `examples/prototyping/src/main.rs` for inventory-driven code generation

## Feature Flags

- `derive` (default): re-exports `GpuiTable`, `Filterable`, `TableCell`, and `gpui_table_impl`
- `chrono` (default): date cell rendering and `gpui_table_component::DateRangeFilter`
- `fluent`: localized titles and faceted labels through `es-fluent`
- `inventory`: inventory-backed `GpuiTableShape` registration for tooling,
  including `ComponentShapeUse` filter metadata
- `mcp`: experimental stdio MCP query integration through generated
  `McpTable` implementations; implies `inventory`
- `rust_decimal`: numeric range filtering and decimal-backed helpers
- `spacetimedb`: SpacetimeDB temporal range filtering support
