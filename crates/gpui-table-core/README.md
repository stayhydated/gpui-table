# gpui-table-core

Pure filter semantics and typed filter values for the `gpui-table` ecosystem.

## What it provides

- Filter matching traits: `Matchable`, `FilterValuesExt`
- Typed filter value wrappers: `TextValue`, `RangeValue`, `FacetedValue`, `SingleValue`
- Faceted-filter traits: `FilterValue`, `Filterable`
- Feature-gated conversion helpers: `ToDecimal` (`rust_decimal`), `ToNaiveDate` (`chrono`)

## Feature flags

- `chrono`: date conversion helpers for range filtering
- `rust_decimal`: numeric conversion helpers for range filtering
- `spacetimedb`: SpacetimeDB temporal conversions for range filtering
- `fluent`: localized bool filter labels

## Notes

- GPUI-facing row traits, loaders, and default cell rendering now live in
  `gpui-table-runtime`.
- Static metadata types live in `gpui-table-schema`.
- All wrapper types default to their inactive state; `SingleValue<T>` and
  `RangeValue<T>` do not require the inner `T` to implement `Default`.
