# Architecture

## Purpose

`gpui-table-core` is the pure filter-semantics layer for the workspace. It owns
typed filter values, filter matching traits, and conversion helpers used by
derive-generated filtering logic.

It intentionally does not depend on `gpui` or `gpui-component`.

## Module map

- `lib.rs`
  - Exposes the pure filter surface
- `filter/`
  - Schema re-exports: `FilterConfig`, `FilterType`, `FacetedFilterOption`, `FacetedFilterIcon`
  - `value.rs`: `FilterValue`, `Filterable`
  - `wrappers.rs`: `FacetedValue`, `RangeValue`, `SingleValue`, `TextValue`
    with manual defaults that do not require inner value types to implement `Default`
  - `traits.rs`: `Matchable`, `FilterValuesExt`
  - `convert.rs`: `ToDecimal`, `ToNaiveDate` (feature gated)

## Data flow

1. `gpui-table-derive` generates `XxxFilterValues` structs using wrappers from this crate.
1. Generated `Matchable` impls call into `TextValue` / `RangeValue` / `FacetedValue`
   helpers plus `ToDecimal` / `ToNaiveDate` conversion traits from this crate.
1. Faceted filters use `Filterable::options()` to obtain schema-level
   `FacetedFilterOption` metadata.

## Extension points

- Implement `FilterValue` / `Filterable` for typed faceted-filter enums.
- Use `TextValue`, `RangeValue`, `FacetedValue`, and `SingleValue` in your own
  filtering code.
- Implement `Matchable<F>` for non-derived filtering flows.

## Feature flags

- `chrono`: enables `ToNaiveDate` conversions for date-range filtering.
- `rust_decimal`: enables `ToDecimal` conversions for numeric-range filtering.
- `spacetimedb`: adds SpacetimeDB temporal conversions on top of `chrono` /
  `rust_decimal`.
- `fluent`: localized bool filter labels used by generated `Filterable<bool>`.
