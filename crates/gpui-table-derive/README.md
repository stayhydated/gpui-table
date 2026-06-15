# gpui-table-derive

`gpui-table-derive` contains the proc macros behind the `gpui-table`
derive-based workflow.

Most application code should depend on `gpui-table` and use the macro
re-exports from there. This crate is mainly for people reading the macro docs
or integrating with the proc-macro layer directly.

## Macros

### `#[derive(GpuiTable)]`

Generates the typed table delegate, column enum, row metadata, optional filter
entities/values, optional inventory registration, and optional MCP table query
registration for a row struct.

```rs
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(gpui_table::runtime::shape::TextFilter))]
    pub name: String,

    #[gpui_table(width = 80., filter(gpui_table::runtime::shape::NumberRangeFilter))]
    pub age: u8,

    #[gpui_table(width = 90., filter(gpui_table::runtime::shape::FacetedFilter::<bool>))]
    pub active: bool,
}
```

Field-level filters require explicit built-in or custom shape paths:

- strings use `filter(gpui_table::runtime::shape::TextFilter)`
- numbers use `filter(gpui_table::runtime::shape::NumberRangeFilter)`
- date-like values use `filter(gpui_table::runtime::shape::DateRangeFilter)`
- enum-like or `Filterable` values use `filter(gpui_table::runtime::shape::FacetedFilter::<T>)`

Faceted filters accept `T`, `Option<T>`, and `Vec<T>` fields. The generated
filter state uses `T` in all cases, so optional and vector fields can facet over
present values without requiring `Option<T>` or `Vec<T>` itself to implement
`Filterable`.

Feature requirements are validated during macro expansion:

- `gpui_table_component::NumberRangeFilter` requires `gpui-table/rust_decimal`
- `gpui_table_component::DateRangeFilter` requires `gpui-table/chrono`
- supported SpacetimeDB range usage requires `gpui-table/spacetimedb`

With the facade crate's `mcp` feature, a table that opts in with
`#[gpui_table(mcp)]` receives a `gpui_table::mcp::McpTable` implementation and
an inventory registration for `gpui-table-mcp`. MCP arguments use filter field
names directly, with `limit` and `offset` reserved for pagination, and decode
into the generated `XxxFilterValues` type before an application-owned query
handler runs. Tables without filters accept only pagination arguments.
Faceted filter schemas include valid `Filterable::to_filter_string()` values
and labels for MCP clients.
Field-level `#[koruma(...)]` validators on filtered fields validate the decoded
MCP filter argument before the query handler runs. Generated schemas attach the
rules in `x-gpuiTableValidation`; literal `LenValidation`, `RangeValidation`,
and `NonEmptyValidation` arguments are also reflected as JSON Schema hints when
the filter argument schema is unambiguous. The validators apply to the filter
shape's raw value, not directly to the row field type. Application crates using
these validators should depend on `koruma` and the validator crate that provides
the rule.
Custom filter shapes used by MCP-enabled tables must implement
`gpui_table::mcp::McpFilterShape`; the generated descriptor and decoder require
that trait, so missing decoders fail at the filter field.
For common custom filters, derive `gpui_table::McpFilterShape` on the shape and
let the generated impl decode `RawValue` through
`gpui_table::mcp::McpToolValue`. The blanket implementation covers
`Deserialize` raw values that implement or derive `McpJsonSchema`. Use
`gpui_table::mcp::McpRange<T>` for range-shaped raw values that should decode
from `{ "min": ..., "max": ... }`. App-owned named structs,
tuple or named transparent newtypes, and fieldless enums can derive
`McpJsonSchema`; fixed tuples with 1 to 4 elements publish exact array schemas,
field schemas record aliases in `x-mcpAliases`, and enum schemas include
aliases. `McpToolInput` also implements `McpJsonSchema`, so object inputs can
be reused as filter raw values. Implement
`gpui_table::mcp::McpFilterShape` manually for explicit schema or decode hooks.
Manual shapes that should support field-level Koruma filter validation must also
implement `gpui_table::mcp::McpFilterShapeValidation`.
Struct-level `#[gpui_table(mcp(...))]` supports `name`, `title`,
`description`, `row_schema`, `read_only`, `destructive`, `idempotent`, and
`open_world` options for generated MCP tool metadata, row output schemas, and
annotations. Use `row_schema` only when the row type implements
`gpui_table::mcp::McpJsonSchema`; opted-in tables publish that schema under the
standard query output `rows.items`. When `description` is omitted, the derive
uses the row type's Rust doc comment.
Use `#[gpui_table::mcp_query]` for custom query handlers and local row sources.
The macro infers the row type from `TableQuery<Row>` and zero-argument
`Result<Vec<Row>, E>` signatures. Local row sources are called for each MCP
query. The inferred row type must opt in with `#[gpui_table(mcp)]`. Custom query
handlers may be synchronous or async and must return matching
`Result<gpui_table::mcp::TableQueryResult<Row>, E>` with `Row: serde::Serialize`.
The handler
registration macros resolve the facade crate path, so renamed `gpui-table`
dependencies work for MCP handler
registration output.

### `#[derive(GpuiTableFilterShape)]`

Generates a custom filter shape by adapting an existing base shape. Use this
when a field should keep the UI, matching, and reset behavior of a built-in
filter but expose a different raw value type to table state or MCP decoding.

```rs
#[derive(Clone, Debug, Default, PartialEq)]
struct PrefixText(String);

#[derive(gpui_table::GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table::runtime::shape::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
struct PrefixTextFilter;
```

The derive emits `ComponentShapeMetadata`, declared-shape marker impls,
`GpuiTableFilterShape`, and `GpuiTableFilterShapeFor<Field>`. When the
`gpui-table/mcp` feature is enabled, it also emits the default
`gpui_table::mcp::McpFilterShape` impl if the raw value implements
`gpui_table::mcp::McpToolValue`.

### `#[derive(Filterable)]`

Generates `FilterValue`, `Filterable`, and `variant_name()` for faceted-filter
enums.

```rs
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

Use enum-level `#[filter(fluent)]` when labels should come from `es-fluent`.

```rs
use gpui_table::Filterable;

#[derive(Clone, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable)]
#[filter(fluent)]
pub enum Status {
    Active,
    Pending,
}
```

Use struct-level `#[gpui_table(fluent = "label")]` with
`EsFluentLabel`/`EsFluentVariants` when generated table titles and field labels
should use typed Fluent resources.

### `#[derive(TableCell)]`

Generates a `TableCell` impl for single-field wrapper types and unit enums.
This is useful when a column should render through an inner type and keep a
dedicated wrapper in your domain model.

Use `#[table_cell(display)]` when the wrapper's own `Display` implementation
should be used instead of delegating to the inner field, or
`#[table_cell(format = path::to::formatter)]` when a dedicated formatter should
own the label.

```rs
use gpui_table::TableCell;

#[derive(TableCell)]
#[table_cell(display)]
pub struct AccountCode(String);

fn render_percent(value: &Percent) -> String {
    format!("{}%", value.0)
}

#[derive(TableCell)]
#[table_cell(format = render_percent)]
pub struct Percent(u8);
```

### `#[gpui_table_impl]`

Attaches load-more behavior to the generated delegate.

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::runtime::TableLoader;

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        cx.notify();
    }
}
```

## Context Menu Helpers

`#[derive(GpuiTable)]` also supports generated row-context-menu links through:

- `context_menu_row_id = "field_name"` or field-level `#[gpui_table(context_menu_id)]`
- `context_menu_route = "/users/{id}"` or `context_menu_route_fn = path::to_fn`
- `context_menu_label = "Open"` or `context_menu_label_fn = path::to_fn`
- `custom_context_menu` when you want to compose the generated link with your own menu items

## When Not To Depend On This Crate Directly

- Use `gpui-table` for the normal application-facing workflow.
- Use `gpui-table-component` if you only need the built-in filter widgets.
- Use `gpui-table-mcp` when you need the experimental MCP query registry,
  stdio server, or generated table query contracts.
- Use `gpui-table-prototyping-core` if you are consuming inventory metadata for generation.

For expansion details and generated type contracts, read the crate rustdocs, the
macro modules under `src/gpui_table`, and the compile-fail fixtures under
`crates/gpui-table/tests/ui`.
