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
use gpui::{App, Window};

let filter = TextFilter::new(
    "Name",
    String::new(),
    move |_value, _window, _cx| {
        // handle filter change
    },
    cx,
)
.alphanumeric_only(cx);

let status = TableStatusBar::new(rows.len(), loading, eof)
    .row_label("Rows");
```

## Notes
- Components are designed to be used via the generated `FilterEntities` in
  `gpui-table`, but can be instantiated directly.
- `NumberRangeFilter` uses `rust_decimal` internally; `DateRangeFilter` uses
  `chrono`.
- This crate ships a small storybook binary (`main.rs`) for previewing filters.
