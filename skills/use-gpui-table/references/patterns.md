# gpui-table application patterns

Use only the section needed for the current application task.

## Dependencies and features

```toml
[dependencies]
gpui-table = { version = "0.6", features = ["rust_decimal"] }
gpui-table-component = "0.6"
```

Keep `gpui` and `gpui-kit` as direct dependencies
using the application's existing source. Add `fluent`,
`inventory`, `mcp`, or `spacetimedb` only for
the corresponding workflow.

## Derive and render a table

```rust
use gpui_table::{Filterable, GpuiTable, TableCell};

#[derive(Clone, Eq, Filterable, Hash, PartialEq, TableCell)]
enum UserStatus {
    Active,
    Suspended,
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(
        sortable,
        width = 160.,
        filter(gpui_table_component::TextFilter)
    )]
    name: String,

    #[gpui_table(
        width = 120.,
        filter(gpui_table_component::FacetedFilter::<UserStatus>.searchable(true))
    )]
    status: UserStatus,
}
```

Construct state and filters in the owning GPUI view:

```rust
use gpui_kit::component::table::{DataTable, TableState};

let delegate = UserTableDelegate::new(rows);
let table = cx.new(|cx| TableState::new(delegate, window, cx));
let filters = UserFilterEntities::build_for_table(table.clone(), cx);

let table_element = DataTable::new(&table)
    .stripe(true)
    .scrollbar_visible(true, true);
```

Render `filters.filter_sidebar_data(cx)` by consuming its groups and
items. Each item supplies a stable ID, label, semantic type, active state, and
erased element.

## Configure built-in filters

```rust
#[gpui_table(filter(gpui_table_component::TextFilter.alphanumeric_only()))]
code: String,

#[gpui_table(filter(
    gpui_table_component::NumberRangeFilter.range(min, max).step(step)
))]
amount: u32,

#[gpui_table(
    filter(gpui_table_component::FacetedFilter::<Status>.searchable(true))
)]
status: Status,

#[gpui_table(filter(gpui_table_component::DateRangeFilter))]
created_at: chrono::DateTime<chrono::Utc>,
```

Text filters also support `alphabetic_only()`,
`numeric_only()`, and `matching_regex(...)`. Route domain
value types and custom shapes to
`use-gpui-table-component-shapes`.

## Control rows and filter state

Generated delegates keep source rows in `rows` and compose generated
filters with an optional application row scope:

```rust
delegate.set_filter_values(values);
delegate.clear_filter_values();
delegate.set_row_scope(|row| row.is_visible);
delegate.clear_row_scope();
```

Call `refresh_filtered_rows()` after mutating row values in place
when visibility may change without a row-count change.

Save and restore a complete generated filter snapshot:

```rust
let preset = filters.read_values(cx).to_preset_json();
let values = UserFilterValues::from_preset_json(&preset)?;
filters.apply_values(values, window, cx);
```

Use `active_filter_count(cx)` for badges and
`reset_filters(window, cx)` for one reset notification.

## Load additional rows

Add `load_more` to the row and implement the generated delegate:

```rust
use gpui_kit::{Context, Window};
use gpui_kit::component::table::TableState;
use gpui_table::runtime::TableLoader;

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        start_application_load(cx);
    }
}
```

The completion update must append rows, clear `loading`, set
`eof` when appropriate, and notify. Use
`FilterEntities::build_for_table_loader(...)` when filter changes
must clear rows and reload backend data.

## Customize cells and context menus

A field-level style function owns one cell element:

```rust
#[gpui_table(style = render_duration)]
duration_ms: u64,

fn render_duration(
    _row: &Event,
    value: &u64,
    _window: &mut gpui_kit::Window,
    _cx: &mut gpui_kit::App,
) -> impl gpui_kit::IntoElement {
    use gpui_kit::ParentElement as _;
    gpui_kit::div().child(format!("{value} ms"))
}
```

Use `#[derive(TableCell)]` for single-field wrappers. Add
`#[table_cell(display)]` for the wrapper's `Display` or
`#[table_cell(format = path)]` for a formatter.

Generated context-menu links use a field-level
`context_menu_id` (or struct-level `context_menu_row_id`)
with a route or route function. Add `custom_context_menu` only when
the application must compose generated and custom actions.

## Initialize localization

With the facade's `fluent` feature, derive the application's
`es-fluent` labels and initialize component localization after
`gpui_kit::component::init(cx)`:

```rust
gpui_kit::component::init(cx);
gpui_table_component::i18n::init(cx)?;
```

Use `#[filter(fluent)]` on faceted enums and
`#[gpui_table(fluent = "label")]` on localized row types. Apply later
locale changes with `gpui_table_component::i18n::set_locale`.

## Expose rows through MCP

```rust
#[derive(Clone, gpui_table::GpuiTable, serde::Serialize)]
#[gpui_table(mcp)]
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

Add `row_schema` and derive
`gpui_table::mcp::McpJsonSchema` when clients need precise row output
metadata. Use a `TableQuery<Row>` handler with
`query.result(rows, total)` for backend-owned execution, or
`query.filter_rows(rows)` for an in-memory collection.

Use `gpui_table::mcp::tool_registry()` to obtain the inventory-discovered
MCP definitions and handlers directly. MCP servers retain that registry across
calls until the transport or application host is explicitly stopped.
