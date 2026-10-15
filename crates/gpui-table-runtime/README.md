# gpui-table-runtime

[![Codecov: gpui-table-runtime][codecov-badge]][codecov]
[![crates.io: gpui-table-runtime][crate-badge]][crate]

`gpui-table-runtime` contains the GPUI-facing contracts targeted by
generated table code: row metadata and rendering, cell rendering, loading, filter
shapes, selection-preserving ordering, and generic helpers for generated filter collections in the
[`gpui-table`][project] ecosystem.

Requires Rust 1.99 or newer. Uses `gpui-kit` 0.7.

Depend on this crate directly when writing reusable integrations over
[`TableCell`][cell], `TableRowMeta`, `TableLoader`,
`GpuiTableFilterShape`, or `FilterEntitiesExt`. Application
tables should normally use the [`gpui-table`][facade] facade.

`TableRowSelection` captures row or cell identity. Clone a snapshot and inspect
`row_id()` to retain a paged selection until its record becomes visible again.
Use an explicit `row_id` when source positions can change.

`ResolvedOrderedTableDelegate` retains typed executable keys. Apply replacements
with `set_table_resolved_sort_columns` to preserve row or cell selection.
`ResolvedTableColumn<Row>` supplies caller-rendered cells for these keys;
`set_table_resolved_columns` preserves selected row and column identities when
presentation columns move or disappear.

## Features

The default feature set is `chrono`, which also enables `jiff`.

| Feature | Default | Purpose |
| --- | --- | --- |
| `chrono` | Yes | Enables Chrono cells, core date conversions, and `jiff` formatting. |
| `jiff` | Yes | Enables Jiff cells and ICU-based localized date/time formatting. |
| `rust_decimal` | No | Enables decimal cells and core numeric conversions. |
| `spacetimedb` | No | Enables `chrono`, SpacetimeDB cells, and core temporal conversions. |

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-runtime
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-runtime.svg?label=gpui-table-runtime
[crate]: https://crates.io/crates/gpui-table-runtime
[project]: https://github.com/stayhydated/gpui-table
[facade]: https://github.com/stayhydated/gpui-table/blob/master/crates/gpui-table/README.md
[cell]: https://docs.rs/gpui-table-runtime/latest/gpui_table_runtime/trait.TableCell.html
