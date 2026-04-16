# gpui-table-core

Pure filter semantics and typed filter values for the `gpui-table` ecosystem.

## What it provides

- Filter matching traits: `Matchable`, `FilterValuesExt`
- Typed filter value wrappers: `TextValue`, `RangeValue`, `FacetedValue`, `SingleValue`
- Faceted-filter traits: `FilterValue`, `Filterable`
- Numeric/date conversion helpers: `ToDecimal`, `ToNaiveDate`

## Feature flags

- `chrono`: date conversion helpers for range filtering
- `rust_decimal`: numeric conversion helpers for range filtering
- `spacetimedb`: SpacetimeDB temporal conversions for range filtering
- `fluent`: localized bool filter labels

## Notes

- GPUI-facing row traits, loaders, and default cell rendering now live in
  `gpui-table-runtime`.
- Static metadata types live in `gpui-table-schema`.
- `SingleValue<T>` and `RangeValue<T>` provide manual `Default` impls, so they do
  not require the inner `T` to implement `Default`.
