# gpui-table-core Architecture

## Purpose

`gpui-table-core` owns the pure filtering model for the workspace. It contains
no GPUI code and should remain the place where filter matching semantics,
typed filter wrappers, and faceted value conversion live.

## Dependency Edges

- Depends only on `gpui-table-schema` plus optional conversion/i18n crates.
- Must stay free of GPUI runtime dependencies so it can be reused by tooling,
  tests, and server-side code.

## Module Map

- `src/lib.rs`
  - Exposes the filter module and optional i18n module.
- `src/filter/mod.rs`
  - Re-export hub for the pure filter surface.
- `src/filter/value.rs`
  - `FilterValue` and `Filterable`, including the built-in `bool` implementation.
- `src/filter/wrappers.rs`
  - `TextValue`, `RangeValue<T>`, `FacetedValue<T>`, and `SingleValue<T>`.
- `src/filter/traits.rs`
  - `Matchable<F>` and `FilterValuesExt`.
- `src/filter/convert.rs`
  - Feature-gated conversion helpers for number/date range filtering.
- `src/i18n.rs`, `i18n/`, `build.rs`, `i18n.toml`
  - Optional typed `es-fluent` localizer for built-in bool faceted labels.

## Internal Contracts

- Wrapper types default to an inactive state without requiring the inner type
  to implement `Default`.
- Wrapper equality preserves `Eq` whenever the wrapped value type supports it.
- `TextValue::matches(...)` is case-insensitive and that behavior is part of the
  core filter contract used by derive-generated client-side filtering.
- `RangeValue` treats missing bounds as open-ended. Codegen and runtime layers
  both assume that representation.
- `FilterValue` is the canonical string round-trip contract for faceted values.
  The schema layer stores strings; the core layer owns how typed values map to them.
- `bool` remains a built-in `Filterable` type so faceted boolean filters work
  without any derive or manual glue.
- The core i18n module owns a context-free embedded localizer for filter metadata
  and non-GPUI fallback paths. GPUI widget code should use
  `gpui-table-component` i18n helpers, which synchronize this core locale.

## Data Flow

1. `gpui-table-derive` validates field/filter combinations, then emits
   `XxxFilterValues` structs composed from wrapper types in this crate.
1. Generated or manual filtering code calls `Matchable<F>` with those wrapper values.
1. Faceted filters use `Filterable::options()` to convert typed variants into
   schema-level `FacetedFilterOption` values.
1. Loader-oriented code can later serialize the wrapper fields through
   `gpui-table-component::QueryFilterValue`.

## Feature Gates

- `chrono` enables `ToNaiveDate`.
- `rust_decimal` enables `ToDecimal`.
- `spacetimedb` layers supported SpacetimeDB temporal conversions on top of the range helpers.
- `fluent` enables localized bool filter labels via typed `es-fluent` messages.
