# gpui-table-runtime

[![Codecov: gpui-table-runtime][codecov-badge]][codecov]
[![crates.io: gpui-table-runtime][crate-badge]][crate]

`gpui-table-runtime` contains the GPUI-facing contracts targeted by
generated table code: row metadata and rendering, cell rendering, loading, filter
shapes, selection-preserving ordering, and generic helpers for generated filter collections in the
[`gpui-table`][project] ecosystem.

Requires Rust 1.99 or newer. Uses `gpui-kit` 0.9.0.

Depend on this crate directly when writing reusable integrations over
[`TableCell`][cell], `TableRowMeta`, `TableLoader`,
`GpuiTableFilterShape`, or `FilterEntitiesExt`. Application
tables should normally use the [`gpui-table`][facade] facade.

`TableRowSelection` captures row and cell identities. Inspect `row_ids()` to
retain a complete paged selection until its records become visible again.
Use an explicit `row_id` when source positions can change.

`ResolvedOrderedTableDelegate` retains typed executable keys. Apply replacements
with `set_table_resolved_sort_columns` to preserve row or cell selection.
`ResolvedTableColumn<Row>` supplies caller-rendered cells for these keys;
`set_table_resolved_columns` preserves selected row and column identities when
presentation columns move or disappear.

For multiple selection, construct the state with
`selection::new_multi_row_table`. Wrap its native `DataTable` with
`selection::selection_scope`; card/grid views use
`selection::selection_container`, which supplies their focus and `DataTable`
keyboard context. The host owns record commands and Tab traversal bindings.
Generated delegates own selected indices,
range anchors, cursors, and input revisions. Pointer modifiers, keyboard ranges,
Select All, Escape, and ordering restoration use the same state. Explicit input
uses `selection::select_row` or `selection::clear_selection`; source refreshes
restore a `TableRowSelection` snapshot without incrementing its input revision.

`text_cell` renders formatted text with a readable accessibility representation.
Built-in cells and derived text formatters use it automatically.

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
