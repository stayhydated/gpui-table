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
- `src/stories/`
  - Storybook registrations showcasing filter and status-bar modes
- `faceted_filter.rs`
  - Multi-select filter with optional search and grouped option metadata
- `text_filter.rs`
  - Debounced text input with optional validation helpers
- `number_range_filter/`
  - Range slider + inputs for numeric filtering (decimal-backed)
  - Default bounds auto-adjust to current entered values (starting from `0..100`);
    explicit `.range(min, max)` keeps bounds fixed
- `date_range_filter.rs`
  - Calendar-based date range picker
  - Formats displayed dates by converting `chrono::NaiveDate` values through
    `jiff` and ICU4X so trigger text matches runtime cell rendering
  - Emits selection changes directly from the calendar instead of re-applying
    the same value when the popover closes
- `reset_filters.rs`
  - Localized reset button for clearing all generated filters
- `table_status_bar.rs`
  - Simple status summary for row count + load state

## Data flow

1. The derive macro generates `XxxFilterEntities` that instantiate these
   components through `gpui_table::runtime::generated_filters`, which re-exports
   this crate as the stable runtime target.
1. Each filter component owns its current state and calls the provided
   `on_change` callback when that state changes. `TextFilter` and
   `NumberRangeFilter` debounce freeform edits, while `FacetedFilter` and
   `DateRangeFilter` emit directly from user selections.
1. `ResetFilters` triggers generated reset bindings that clear all filters in one action.
1. Consumers read all filter values via generated `read_values()` /
   `all_filters()` helpers, or generically through `FilterEntitiesExt`.
   Those values can then be applied client-side or passed into loader requests.
1. `QueryFilterValue` supports both raw component values and the generated
   wrapper types from `gpui-table-core::filter`, so server-side loaders can
   serialize either representation directly.
   Faceted-value query strings are normalized into sorted comma-separated output
   so equivalent selections serialize deterministically.

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

- `chrono` (default): enables `DateRangeFilter`, date-based query serialization,
  and ICU4X-backed date display formatting for selected ranges.
- `rust_decimal` (default): enables `NumberRangeFilter` and decimal-based query serialization.
- `story`: enables the storybook binary and pulls in both range-filter features.
