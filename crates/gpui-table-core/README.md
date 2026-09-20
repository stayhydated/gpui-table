# gpui-table-core

[![Codecov: gpui-table-core][codecov-badge]][codecov]
[![crates.io: gpui-table-core][crate-badge]][crate]

`gpui-table-core` provides typed filter values and matching semantics
without depending on GPUI for shared client/server filtering, non-UI query
logic, and generic code over `Matchable<F>` in the [`gpui-table`][project]
ecosystem.

The crate includes text, faceted, range, and single-value wrappers; faceted
filter traits; and feature-gated date, decimal, SpacetimeDB, and Fluent support.
Applications that derive or render tables should normally depend on
`gpui-table` instead.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-core
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-core.svg?label=gpui-table-core
[crate]: https://crates.io/crates/gpui-table-core
[project]: https://github.com/stayhydated/gpui-table
