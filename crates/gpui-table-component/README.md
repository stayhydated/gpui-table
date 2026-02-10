# gpui-table-component

GPUI filter UI components and a table status bar used by `gpui-table`.

## Components

- `TextFilter`
- `FacetedFilter`
- `NumberRangeFilter`
- `DateRangeFilter`
- `TableStatusBar`

## Traits

- `TableFilterComponent` for type-safe component construction in generated code
- `FilterValue` for query-string conversion of filter values (separate from
  `gpui_table_core::filter::FilterValue`)

## Example

```rs
use gpui_table_component::{TextFilter, TextFilterExt, TableStatusBar};
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

- Components are designed to be used via the generated `FilterEntities` in
  `gpui-table`, but can be instantiated directly.
- Filter components expose chainable style setters that accept
  `StyleRefinement` to customize trigger/input/popover subparts.
- `NumberRangeFilter` uses `rust_decimal` internally; `DateRangeFilter` uses
  `chrono`.
- This crate ships a small storybook binary (`main.rs`) for previewing filters.
- Story definitions live in `src/stories` and are auto-registered via
  `gpui-storybook` inventory macros.
