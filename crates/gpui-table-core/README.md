# gpui-table-core

`gpui-table-core` is the pure filter-semantics layer for the workspace.
It holds typed filter wrapper values, faceted-filter traits, and conversion
helpers without depending on GPUI.

This crate is for integration work. Most application code should depend on
`gpui-table` instead.

## Use This Crate When

- you want to match typed filter state outside the GPUI runtime
- you want shared filter DTOs or logic between client and server code
- you want to test filtering behavior without bringing in GPUI dependencies

## Example

```rs
use gpui_table_core::filter::{Matchable, RangeValue, TextValue};

#[derive(Default)]
pub struct UserFilters {
    pub name: TextValue,
    pub age: RangeValue<u8>,
}

pub struct User {
    pub name: String,
    pub age: u8,
}

impl Matchable<UserFilters> for User {
    fn matches_filters(&self, filters: &UserFilters) -> bool {
        filters.name.matches(&self.name) && filters.age.matches(&self.age)
    }
}
```

## What It Provides

- `TextValue`, `RangeValue<T>`, `FacetedValue<T>`, and `SingleValue<T>`
- `FilterValue` and `Filterable` for typed faceted filters
- `Matchable<F>` and `FilterValuesExt` for strongly typed filtering flows
- `ToDecimal` and `ToNaiveDate` when the corresponding features are enabled
- schema re-exports such as `FilterConfig`, `FilterType`, and `FacetedFilterOption`

`bool` already implements `FilterValue` and `Filterable`, so faceted boolean
filters work without extra glue code.

## Feature Flags

- `chrono` (default): enables `ToNaiveDate` conversions for date-range filters
- `fluent`: localizes built-in bool faceted labels through `es-fluent`
- `rust_decimal`: enables `ToDecimal` conversions for numeric-range filters
- `spacetimedb`: adds supported SpacetimeDB temporal conversions on top of the range helpers

If you also need generated tables, GPUI runtime traits, or built-in filter UI,
use `gpui-table` instead of depending on this crate directly.

For implementation details and internal contracts, see `docs/ARCHITECTURE.md`.
