# Architecture

## Purpose

`gpui-table-component` provides GPUI UI components for table filtering and a
status bar. These components are used by the generated filter entities when
`#[gpui_table(filters)]` is enabled.

## Module map

- `lib.rs`
  - `TableFilterComponent` trait used by the built-in generated filter entities
  - `QueryFilterValue` trait for query-string conversion (distinct from
    `gpui_table_core::filter::FilterValue`)
  - Re-exports the concrete component types and extension traits at the crate root
  - Gates `date_range_filter` / `number_range_filter` behind `chrono` /
    `rust_decimal`
- `src/bin/story.rs`
  - Storybook gallery entrypoint for previewing filter components
- `stories/`
  - Storybook registrations showcasing filter and status-bar modes
- `faceted_filter.rs`
  - Multi-select filter with optional search and option providers
- `text_filter.rs`
  - Debounced text input with optional validation helpers
- `number_range_filter.rs`
  - Range slider + inputs for numeric filtering (decimal-backed)
  - Default bounds auto-adjust to current entered values (starting from `0..100`);
    explicit `.range(min, max)` keeps bounds fixed
- `date_range_filter.rs`
  - Calendar-based date range picker
- `reset_filters.rs`
  - Localized reset button for clearing all generated filters
- `table_status_bar.rs`
  - Simple status summary for row count + load state

## Data flow

1. The derive macro generates `XxxFilterEntities` that instantiate these
   components through `gpui_table::runtime::generated_filters`, which re-exports
   this crate as the stable runtime target.
1. Each filter component calls the provided `on_change` callback with its value.
1. `ResetFilters` triggers generated reset bindings that clear all filters in one action.
1. Consumers read all filter values via `FilterEntitiesExt::read_values` and
   apply them client-side or pass them into load-more requests.
1. `QueryFilterValue` supports both raw component values and the generated
   wrapper types from `gpui-table-core::filter`, so server-side loaders can
   serialize either representation directly.

## Extension points

- Add standalone filter component types by implementing `TableFilterComponent`.
- `#[derive(GpuiTable)]` currently wires only the built-in filter syntaxes from
  `gpui-table-derive`; custom `TableFilterComponent` implementations are not yet
  selectable through `#[gpui_table(filter(...))]`.
- Custom components therefore integrate at runtime today: instantiate them
  directly or wrap them in a manual filter-entity collection that owns state,
  query serialization, and reload callbacks.
- Extend filter components with chainable configuration methods (extension traits).
- Style existing filter components via chainable extension-trait setters that accept
  `gpui::StyleRefinement` (for example trigger/input/popover segment styles).

## Notes

- These components assume `gpui-component` primitives (inputs, popovers, sliders)
  and are intended for GPUI-based apps.

## Feature flags

- `chrono` (default): enables `DateRangeFilter` and date-based query serialization.
- `rust_decimal` (default): enables `NumberRangeFilter` and decimal-based query serialization.
- `story`: enables the storybook binary and pulls in both range-filter features.
