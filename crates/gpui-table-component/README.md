# gpui-table-component

`gpui-table-component` provides the built-in GPUI filter widgets and
`TableStatusBar` used across the `gpui-table` ecosystem.

Use this crate when you want direct control over filter UI composition.
Most application code should start with `gpui-table`.

## Use This Crate When

- you want to instantiate the built-in table filters directly in GPUI code
- you want `TableStatusBar` without deriving an entire table
- you need `QueryFilterValue` to serialize raw or generated filter values for loader requests

## Example

```rs
use gpui::{StyleRefinement, px};
use gpui_table_component::{TableStatusBar, TextFilter, TextFilterExt};

let filter = TextFilter::new(
    "Name",
    String::new(),
    move |_value, _window, _cx| {
        // react to filter changes
    },
    cx,
)
.alphanumeric_only(cx)
.container_style(StyleRefinement::default().w_full(), cx)
.input_style(StyleRefinement::default().w(px(280.)), cx);

let status = TableStatusBar::new(rows.len(), loading, eof)
    .row_label("Rows")
    .activity_style(StyleRefinement::default().font_semibold());
```

## Built-In Components

- `TextFilter`
- `FacetedFilter`
- `NumberRangeFilter` when `rust_decimal` is enabled
- `DateRangeFilter` when `chrono` is enabled
- `ResetFilters`
- `TableStatusBar`

The filter component types are also the built-in `#[gpui_table(filter(...))]`
shape types:

```rs
#[gpui_table(filter(gpui_table_component::TextFilter))]
name: String,

#[gpui_table(filter(gpui_table_component::TextFilter.numeric_only()))]
numeric_code: String,

#[gpui_table(filter(gpui_table_component::TextFilter.matching_regex(r"[A-Z0-9-]*")))]
external_ref: String,

#[gpui_table(filter(
    gpui_table_component::NumberRangeFilter
        .range(rust_decimal::Decimal::new(0, 0), rust_decimal::Decimal::new(100, 0))
        .step(rust_decimal::Decimal::new(10, 0))
))]
score: rust_decimal::Decimal,

#[gpui_table(filter(gpui_table_component::FacetedFilter::<Status>.searchable(true)))]
status: Status,
```

Configured built-in filter expressions construct the same shape type while
applying its `GpuiTableFilterShapeBuilder`: `TextFilter.alphabetic_only()`,
`TextFilter.numeric_only()`, `TextFilter.alphanumeric_only()`, and
`TextFilter.matching_regex("...")`
enable text validators, `NumberRangeFilter.range(...).step(...)` sets numeric
slider bounds and step size, and
`FacetedFilter::<T>.searchable(true)` shows the faceted search input.

When a field is an application-owned value type but should reuse a built-in
filter widget, use an adapter shape and implement its field trait:

```rs
pub struct AccountCode(String);

impl gpui_table_component::TextFilterField for AccountCode {
    fn to_filter_text(&self) -> String {
        self.0.clone()
    }
}

#[gpui_table(filter(gpui_table_component::TextFilterAdapter))]
code: AccountCode,
```

`TextFilterAdapter`, `NumberRangeFilterAdapter`, and `DateRangeFilterAdapter`
reuse the built-in UI, raw value, reset behavior, and MCP schema while adding
support for both `T` and `Option<T>` fields that implement the matching field
trait.

All filter widgets expose chainable extension-trait setters for styling or
behavior tweaks.

## Localization

Built-in component text is localized through typed `es-fluent` messages. Manual
component composition should initialize the component localizer in GPUI startup
and can later select a locale through the public i18n helper:

```rs
gpui_table_component::i18n::init(cx)?;
gpui_table_component::i18n::set_locale(cx, "en")?;
```

Generated filter flows call the same localization helpers through
`gpui_table_component::i18n`. Runtime widget text reads the
`EmbeddedI18n` handle from GPUI global state. Locale parsing and selection
return typed errors. Ordinary message and label rendering requires initialized
localization state and a matching typed resource; missing state or resources
fail hard rather than rendering a key-derived string.
The status bar follows the same typed-resource path. Date-range display uses
the active `gpui-component` locale and treats invalid locales or unavailable
ICU formatters as hard errors instead of substituting raw or English dates.

## Interop With Generated Tables

The derive-generated filter code constructs filter widgets through
`gpui_table::runtime::shape::GpuiTableFilterShape`; this crate supplies those
impls for the built-in filter widgets.

That means you can:

- let `#[derive(GpuiTable)]` build the standard filters for you
- use this crate directly when you want manual composition
- serialize either raw component values or generated wrapper values with `QueryFilterValue`

Custom `TableFilterComponent` implementations are a runtime integration point.
They are useful for manual filter collections, but generated tables also require
a `GpuiTableFilterShape` implementation before a component can be used in
`#[gpui_table(filter(...))]`.

## Feature Flags

- `chrono` (default): enables `DateRangeFilter`
- `rust_decimal` (default): enables `NumberRangeFilter`
- `mcp`: implements `gpui_table::mcp::McpFilterShape` for the built-in filters
  and adapters
- `story`: enables the storybook binary and pulls in the built-in filter stories

## Storybook

```sh
cargo run -p gpui-table-component --bin story --features story
```

The launcher uses the stable `gpui-table-component-story` consumer ID and
passes `gpui_table_component::i18n::apply_locale` to `StorybookOptions`. The
adapter explicitly links this crate's generated Fluent module and keeps the
component and table-core contexts synchronized. Storybook loads the consumer's
local preferences before the first window is created; normal launches persist
saved intent in `.gpui-storybook/gpui-table-component-story.json` at the
workspace root.

For internals, module boundaries, and serialization contracts, read the crate
rustdocs and the focused tests in `src/lib.rs`.
