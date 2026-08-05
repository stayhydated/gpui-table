# gpui-table-core

`gpui-table-core` provides typed filter values and matching semantics
without depending on GPUI. It is useful for shared client/server filtering,
non-UI query logic, and generic code over `Matchable<F>`.

The crate includes text, faceted, range, and single-value wrappers; faceted
filter traits; and feature-gated date, decimal, SpacetimeDB, and Fluent support.
Applications that derive or render tables should normally depend on
`gpui-table` instead.

- [Feature and crate guide](https://stayhydated.github.io/gpui-table/book/features.html)
- [API documentation](https://docs.rs/gpui-table-core/)
