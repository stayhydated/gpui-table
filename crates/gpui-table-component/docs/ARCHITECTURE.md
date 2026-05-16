# gpui-table-component Architecture

## Purpose

`gpui-table-component` owns the concrete GPUI widgets for built-in table
filtering plus `TableStatusBar`. It is the only crate in the ecosystem that
should know about the specific UI composition of those built-in filters.

## Dependency Edges

- Depends on `gpui` and `gpui-component` for all rendering and interaction.
- Depends on `gpui-table-core` for typed filter wrappers and `FilterValue`.
- Depends on `gpui-table-schema` for UI-neutral metadata such as
  `RegistryFilterType` and `FacetedFilterIcon`.
- Is consumed indirectly by generated code through
  `gpui_table::runtime::generated_filters`, not by a direct path from the derive crate.

## Module Map

- `src/lib.rs`
  - Exports the component types.
  - Defines `TableFilterComponent`, the constructor trait used by generated filter entities.
  - Defines `QueryFilterValue`, the serialization trait shared by raw widget
    values and generated wrapper values.
- `src/text_filter.rs`
  - Debounced text input filter and its extension trait.
- `src/faceted_filter.rs`
  - Multi-select faceted filter with grouped options and optional search.
- `src/number_range_filter/`
  - Decimal-backed numeric range UI and styling helpers.
- `src/date_range_filter.rs`
  - Date range picker and formatted trigger text.
- `src/reset_filters.rs`
  - Localized reset button used by generated filter sets.
- `src/table_status_bar.rs`
  - Row-count and loading/eof summary UI.
- `src/stories/` and `src/bin/story.rs`
  - Storybook infrastructure for previewing the built-in components.
- `src/i18n.rs`, `i18n/`, `build.rs`, `i18n.toml`
  - Typed `es-fluent` messages, the GPUI-global component localizer, fallback
    helpers for context-free metadata, and asset tracking for component labels.

## Internal Contracts

- `TableFilterComponent::new(...)` is the constructor contract generated
  `XxxFilterEntities` rely on. Changes here must stay compatible with the
  derive/runtime layers.
- `QueryFilterValue` must serialize empty values as `None`. Generated loader
  flows depend on that to omit inactive filters.
- Faceted query serialization is sorted before joining with commas. That keeps
  logically equivalent selections deterministic across runs.
- Range serialization uses `>=x`, `<=x`, or `min-max`. Loader code should treat
  that as the canonical wire format for the built-in widgets.
- This crate can add new runtime widgets, but the derive crate only understands
  the hard-coded built-in filter syntaxes. Adding a widget here does not add a
  new `#[gpui_table(filter(...))]` form by itself.
- Component i18n stores its embedded localizer in GPUI global state, synchronizes
  the context-free core locale, and reads the active `gpui-component` locale
  before localizing built-in strings.
  Widget code should call this crate's localization helpers rather than
  accessing embedded resources directly.

## Data Flow

1. `gpui-table-derive` emits `XxxFilterEntities` that call
   `gpui_table::runtime::generated_filters::TableFilterComponent::new(...)`.
1. `gpui-table-runtime::generated_filters` re-exports this crate as the stable
   runtime target.
1. Each component owns its immediate UI state and invokes the supplied
   `on_change` callback.
1. Generated filter collections then snapshot the current widget state into
   typed wrappers from `gpui-table-core`.
1. Loader-oriented code can serialize either the raw component value or the
   wrapper value through `QueryFilterValue`.

## Feature Gates

- `chrono` enables `DateRangeFilter` and date query serialization.
- `rust_decimal` enables `NumberRangeFilter` and decimal query serialization.
- `story` enables the storybook binary and its supporting dependencies.
