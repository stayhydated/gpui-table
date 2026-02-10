# Architecture

## Purpose

`gpui-table-component` provides GPUI UI components for table filtering and a
status bar. These components are used by the generated filter entities when
`#[gpui_table(filters)]` is enabled.

## Module map

- `lib.rs`
  - `TableFilterComponent` trait for type-safe filter component construction
  - `FilterValue` trait for query-string conversion (distinct from
    `gpui_table_core::filter::FilterValue`)
  - Re-exports extension traits for filter configuration
- `main.rs`
  - Storybook gallery entrypoint for previewing filter components
- `stories/`
  - Storybook registrations showcasing filter and status-bar modes
- `faceted_filter.rs`
  - Multi-select filter with optional search and option providers
- `text_filter.rs`
  - Debounced text input with optional validation helpers
- `number_range_filter.rs`
  - Range slider + inputs for numeric filtering (decimal-backed)
- `date_range_filter.rs`
  - Calendar-based date range picker
- `table_status_bar.rs`
  - Simple status summary for row count + load state

## Data flow

1. The derive macro generates `XxxFilterEntities` that instantiate these
   components using the `TableFilterComponent` trait.
1. Each filter component calls the provided `on_change` callback with its value.
1. Consumers read all filter values via `FilterEntitiesExt::read_values` and
   apply them client-side or pass them into load-more requests.

## Extension points

- Add new filter component types by implementing `TableFilterComponent`.
- Extend filter components with chainable configuration methods (extension traits).
- Style existing filter components via chainable extension-trait setters that accept
  `gpui::StyleRefinement` (for example trigger/input/popover segment styles).

## Notes

- These components assume `gpui-component` primitives (inputs, popovers, sliders)
  and are intended for GPUI-based apps.
