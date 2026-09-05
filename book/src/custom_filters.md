# Filter components and custom shapes

Choose the least custom filter path that models the field. Built-in shapes cover
ordinary strings, facets, numeric ranges, and date ranges. Adapter shapes let
domain value types reuse those widgets. Implement a new runtime shape only when
the component or matching semantics are genuinely different.

| Need | Use |
|---|---|
| A standard filter on a supported field | `TextFilter`, `FacetedFilter<T>`, `NumberRangeFilter`, or `DateRangeFilter` |
| A standard widget over an application value object | `TextFilterAdapter`, `NumberRangeFilterAdapter`, or `DateRangeFilterAdapter` |
| A different raw value over an existing shape | `#[derive(GpuiTableFilterShape)]` |
| A new widget or matching model | Implement the runtime shape contracts |

## Use a filter component directly

Construct components directly when the application owns a manual filter
collection rather than generated `FilterEntities`. Initialize table
component localization before rendering them.

```rust,ignore
use gpui_kit::{StyleRefinement, px};
use gpui_table_component::{TextFilter, TextFilterExt};

let filter = TextFilter::new(
    "Name",
    String::new(),
    move |value, _window, _cx| {
        update_query(value);
    },
    cx,
)
.alphanumeric_only(cx)
.container_style(StyleRefinement::default().w_full(), cx)
.input_style(StyleRefinement::default().w(px(280.)), cx);
```

`TableStatusBar::new(row_count, loading, eof)` provides the matching
row-count and loading indicator. `ResetFilters` is available for
manual collections; generated filter collections provide their own
`reset_filters(window, cx)` method.

## Adapt an application value type

Use a built-in adapter when a domain wrapper should keep the built-in widget,
raw value, preset behavior, and MCP schema. Implement the matching field trait:

```rust,ignore
#[derive(Clone, gpui_table::TableCell)]
struct AccountCode(String);

impl gpui_table_component::TextFilterField for AccountCode {
    fn to_filter_text(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(filters)]
struct Account {
    #[gpui_table(filter(gpui_table_component::TextFilterAdapter))]
    code: AccountCode,
}
```

`TextFilterAdapter` supports `T` and
`Option<T>` when `T: TextFilterField`. The number and date
adapters follow the same pattern with `NumberRangeFilterField` and
`DateRangeFilterField`, subject to their feature flags.

## Adapt an existing shape

Derive `GpuiTableFilterShape` when the base widget and matching
behavior are correct but its component-facing value needs conversion:

```rust,ignore
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

The generated shape delegates UI construction, matching, reset, and preset
application to the base shape. Use `fields(A, B, ...)` instead of
`field = A` when the adapter supports several field types. Use
`koruma_newtype` when a Koruma newtype should filter and validate its
inner value.

For MCP tables, the adapted raw value must implement
`gpui_table::mcp::McpToolValue` for the derive to provide the default
MCP decoder and schema.

## Implement a new shape

A fully custom filter connects a component to generated table state through:

- `ComponentShapeMetadata` and
  `DeclaredComponentShape`
- `GpuiTableFilterShape` and
  `DeclaredGpuiTableFilterShape`
- `ComponentShapeFor<Field>` and
  `GpuiTableFilterShapeFor<Field>` for each supported field type

The runtime shape defines its component, raw value, typed filter value, semantic
filter category, construction, value reading, wrapping, reset, and field
matching. Inactive filter values must match every row.

Generated saved presets require the typed filter value to implement
`gpui_table::FilterPresetValue`. Override `unwrap_value` and
`set_silent` so applying a preset restores a non-default value without
firing each component callback.

For MCP tables, derive `gpui_table::McpFilterShape` when the raw value
implements `McpToolValue`, or implement `McpFilterShape`
directly for a custom schema or decoder. Add
`McpFilterShapeValidation` when field-level Koruma validation must be
supported.
