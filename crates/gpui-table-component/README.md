# gpui-table-component

GPUI filter UI components and a table status bar used by `gpui-table`.

`TextFilter`, `FacetedFilter`, `ResetFilters`, and `TableStatusBar` are
re-exported at the crate root. `DateRangeFilter` and `NumberRangeFilter` are
also re-exported when their features are enabled, along with the corresponding
extension traits (`TextFilterExt`, `FacetedFilterExt`, `DateRangeFilterExt`,
`NumberRangeFilterExt`).

## Interactive examples

```sh
cargo run -p gpui-table-component --bin story --features story
```

## Components

- `TextFilter`
- `FacetedFilter`
- `NumberRangeFilter` (`rust_decimal` feature, enabled by default)
- `DateRangeFilter` (`chrono` feature, enabled by default)
- `ResetFilters`
- `TableStatusBar`

## Traits

- `TableFilterComponent` for built-in filter component construction in generated code
- `QueryFilterValue` for query-string conversion of raw component values and
  generated wrapper values (distinct from `gpui_table_core::filter::FilterValue`)

## Example

```rs
use gpui_table_component::{TableStatusBar, TextFilter, TextFilterExt};
use gpui::{App, StyleRefinement, Window, px};

let filter = TextFilter::new(
    "Name",
    String::new(),
    move |_value, _window, _cx| {
        // handle filter change
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

## Notes

- The derive macros build these components for generated table filters, but you
  can also instantiate them directly in manual GPUI code.
- `#[derive(GpuiTable)]` currently supports the built-in filter syntaxes only;
  implementing `TableFilterComponent` does not automatically add new
  `#[gpui_table(filter(...))]` options.
- Custom filter components are a manual integration point today: instantiate
  them directly, or build your own filter-entity collection / reload wiring
  around `TableFilterComponent` and `QueryFilterValue`.
- `QueryFilterValue` supports the raw values used by the built-in components,
  the generated wrapper types (`TextValue`, `RangeValue`, `FacetedValue`), and
  manual `SingleValue` integrations, so generated
  `XxxFilterEntities::read_values()` can feed query serialization directly.
- Filter components expose chainable style setters that accept
  `StyleRefinement` to customize trigger/input/popover subparts.
- `NumberRangeFilter` uses `rust_decimal` internally; `DateRangeFilter` uses
  `chrono`.
- `chrono` and `rust_decimal` are default-enabled for direct crate users; opt
  out with `default-features = false` if you only need text/faceted filters.
- This crate ships a small storybook binary at `src/bin/story.rs` for previewing
  filters and `TableStatusBar`.
- Story definitions live in `src/stories` and are auto-registered via
  `gpui-storybook` inventory macros.
