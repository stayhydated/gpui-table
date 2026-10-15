# gpui-table-core

[![Codecov: gpui-table-core][codecov-badge]][codecov]
[![crates.io: gpui-table-core][crate-badge]][crate]

`gpui-table-core` provides typed filter values, matching semantics and fallible ordering
without depending on GPUI for shared client/server filtering, non-UI query
logic, and generic code over `Matchable<F>` in the [`gpui-table`][project]
ecosystem.

Requires Rust 1.99 or newer.

The crate includes text, faceted, range, and single-value wrappers; faceted
filter traits; and feature-gated date, decimal, SpacetimeDB, and Fluent support.
Applications that derive or render tables should normally depend on
[`gpui-table`][facade] instead.

## Features

The default feature set is `chrono`.

| Feature | Default | Purpose |
| --- | --- | --- |
| `chrono` | Yes | Provides Chrono date conversions and the `DateRange` filter category. |
| `fluent` | No | Provides Fluent labels and localization initialization for filter categories. |
| `rust_decimal` | No | Provides decimal numeric conversions and the `NumberRange` filter category. |
| `spacetimedb` | No | Enables `chrono` and SpacetimeDB temporal conversions; decimal conversions additionally need `rust_decimal`. |

## Example

No integration features or GPUI context are needed for text matching. An active
filter matches case-insensitively; the empty filter leaves rows unrestricted:

```rust
use gpui_table_core::filter::TextValue;

fn main() {
    let filter = TextValue::from("ada");
    assert!(filter.matches("Ada Lovelace"));
    assert!(!filter.matches("Grace Hopper"));
    assert!(TextValue::default().matches("Grace Hopper"));
}
```

See the [API reference][api] for matching and fallible ordering semantics.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-core
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-core.svg?label=gpui-table-core
[crate]: https://crates.io/crates/gpui-table-core
[project]: https://github.com/stayhydated/gpui-table
[facade]: https://github.com/stayhydated/gpui-table/blob/master/crates/gpui-table/README.md
[api]: https://docs.rs/gpui-table-core/latest/gpui_table_core/
