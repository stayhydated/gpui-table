# gpui-table Application Patterns

Load this reference when implementing user-facing application tables, filters, load-more behavior, custom row rendering, row context menus, localization, or direct filter widgets.

## Feature Flags

```toml
[dependencies]
gpui-table = { version = "*", features = ["fluent", "rust_decimal"] }
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
    #[gpui_table(sortable, width = 160., filter(gpui_table::runtime::shape::TextFilter))]
    pub name: String,

    #[gpui_table(width = 80., filter(gpui_table::runtime::shape::NumberRangeFilter))]
    pub age: u8,

    #[gpui_table(width = 120., filter(gpui_table::runtime::shape::FacetedFilter::<UserStatus>))]
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
#[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
name: String,

#[gpui_table(filter(gpui_table::runtime::shape::NumberRangeFilter))]
age: u8,

#[gpui_table(filter(gpui_table::runtime::shape::DateRangeFilter))]
created_at: chrono::DateTime<chrono::Utc>,

#[gpui_table(filter(gpui_table::runtime::shape::FacetedFilter::<UserStatus>))]
status: UserStatus,
```

Built-in filters are explicit shape paths: `TextFilter` for strings,
`NumberRangeFilter` for numeric values, `DateRangeFilter` for date-like values,
and `FacetedFilter::<T>` for enum-like fields. Use the same
`filter(path::ToShape)` form for custom shapes.

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
    #[gpui_table(filter(gpui_table::runtime::shape::FacetedFilter::<UserStatus>))]
    pub status: UserStatus,
}
```

Use the application's existing Fluent resource layout and locale-selection pattern. Keep generated table labels and faceted enum labels in the same localization system as the rest of the app.

## Custom Rendering

Use `#[gpui_table(custom_style)]`, implement `TableRowStyle`, and fall back to `default_render_cell` for standard columns.

```rust
impl gpui_table::runtime::TableRowStyle for Item {
    type ColumnId = ItemTableColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        use gpui::IntoElement as _;

        match col {
            ItemTableColumn::Weight => {
                use gpui::{ParentElement, div};
                div().child(format!("{} kg", self.weight)).into_any_element()
            },
            _ => gpui_table::runtime::default_render_cell(self, col.into(), window, cx)
                .into_any_element(),
        }
    }
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
selects an application-owned backend, while a zero-argument `Result<Vec<Row>, E>`
return type selects a local row source. Local row sources are called for each MCP
query.

```rust
#[derive(Clone, gpui_table::GpuiTable, serde::Serialize)]
#[gpui_table(filters, mcp)]
struct User {
    #[gpui_table(filter(gpui_table::runtime::shape::TextFilter))]
    name: String,
}

#[gpui_table::mcp_query]
fn rows() -> Result<Vec<User>, String> {
    Ok(vec![/* rows */])
}

fn main() -> gpui_table::mcp::ServeStdioResult {
    gpui_table::mcp::serve_stdio_blocking()
}
```

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
Schema hints when the filter argument schema is unambiguous. Add `koruma` and
the validator crate that provides the rule to the application dependencies.
Custom query handlers can be synchronous or async and must return
`Result<gpui_table::mcp::TableQueryResult<Row>, E>`.
Use `query.result(rows, total)` to build the standard response from a decoded
query.
Use struct-level `#[gpui_table(mcp(...))]` with `name`, `title`,
`description`, `read_only`, `destructive`, `idempotent`, and `open_world` when
generated MCP tools need application-owned metadata or MCP tool annotation
hints. If `description` is omitted, the derive uses the row type's Rust doc
comment. Generated table query tools default to read-only, non-destructive, and
idempotent annotations.
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
`.row_source_async(source)?` for local rows.
Registration reports setup errors such as duplicate tool names.
MCP schemas and decoders use the same explicit filter shapes selected for
generated filter UI.
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
`gpui_table::mcp::McpFilterShapeValidation`.
