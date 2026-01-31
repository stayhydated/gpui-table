# Architecture

## Purpose
`gpui-table-core` holds the core traits and data structures that power table
metadata, filtering, and rendering. It has no proc-macro code and no UI
components; it is the foundation used by both the derive macros and the
runtime components.

## Module map
- `lib.rs`
  - Table traits: `TableRowMeta`, `TableRowStyle`, `TableCell`
  - Loading traits: `TableLoader`, `TableDataLoader`
  - Default rendering helpers: `default_render_cell`, `default_render_row`
  - Internal load-more bridge: `__private::LoadMoreDelegate`
- `filter/`
  - `config.rs`: `FilterConfig`, `FilterType`, `FacetedFilterOption`
  - `value.rs`: `FilterValue`, `Filterable`
  - `wrappers.rs`: `FacetedValue`, `RangeValue`, `TextValue`
  - `traits.rs`: `Matchable`, `FilterValuesExt`, `FilterEntitiesExt`
  - `convert.rs`: `ToDecimal`, `ToNaiveDate` (feature gated)
- `registry.rs`
  - `GpuiTableShape`, `ColumnVariant`, `FilterVariant`
  - `RegistryFilterType`, `ColumnFixed`
  - Inventory registration helpers

## Data flow
1. The derive macro generates `TableRowMeta`/`TableRowStyle` impls and optional
   filter metadata based on user attributes.
2. Filter wrappers and traits enable generated `FilterValues` to express
   "active" state and match rows (client-side filtering).
3. The registry module exposes static metadata for tools that want to inspect
   table shapes (e.g., prototyping or codegen).

## Extension points
- Implement `TableCell` for custom value types.
- Implement `FilterValue`/`Filterable` for faceted filter enums.
- Override `TableRowStyle` to customize row or cell rendering.

## Feature flags
- `chrono`: `TableCell` and date conversion helpers.
- `rust_decimal`: numeric conversion helpers for range filters.
- `fluent`: localized title helpers used by generated code.
