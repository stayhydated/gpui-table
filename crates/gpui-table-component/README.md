# gpui-table-component

`gpui-table-component` provides the built-in GPUI filter widgets and
`TableStatusBar` used across the `gpui-table` ecosystem.

Use this crate when you want direct control over filter UI composition.
Most application code should still start with `gpui-table`.

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

All filter widgets expose chainable extension-trait setters for styling or
behavior tweaks.

## Interop With Generated Tables

The derive-generated filter code targets this crate through
`gpui_table::runtime::generated_filters`.

That means you can:

- let `#[derive(GpuiTable)]` build the standard filters for you
- use this crate directly when you want manual composition
- serialize either raw component values or generated wrapper values with `QueryFilterValue`

Custom `TableFilterComponent` implementations are a runtime integration point.
They are useful for manual filter collections, but they do not add a new
`#[gpui_table(filter(...))]` syntax on their own.

## Feature Flags

- `chrono` (default): enables `DateRangeFilter`
- `rust_decimal` (default): enables `NumberRangeFilter`
- `story`: enables the storybook binary and pulls in the built-in filter stories

## Storybook

```sh
cargo run -p gpui-table-component --bin story --features story
```

For internals, module boundaries, and serialization contracts, see
`docs/ARCHITECTURE.md`.
