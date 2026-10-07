# gpui-table-runtime

[![Codecov: gpui-table-runtime][codecov-badge]][codecov]
[![crates.io: gpui-table-runtime][crate-badge]][crate]

`gpui-table-runtime` contains the GPUI-facing contracts targeted by
generated table code: row metadata and rendering, cell rendering, loading, filter
shapes, selection-preserving ordering, and generic helpers for generated filter collections in the
[`gpui-table`][project] ecosystem.

Depend on this crate directly when writing reusable integrations over
`TableRowMeta`, `TableLoader`,
`GpuiTableFilterShape`, or `FilterEntitiesExt`. Application
tables should normally use the `gpui-table` facade.

`TableRowSelection` captures a row or cell selection by stable identity.
Paged callers can clone the snapshot, inspect `row_id()`, and retain it until
that record is visible again before restoring it through the table state.

`ResolvedOrderedTableDelegate` retains executable source-resolved columns.
`set_table_resolved_sort_columns` replaces that context atomically and preserves
row or cell selection by identity.

Chrono cells use localized formatting for values representable by Jiff. Dates
outside Jiff's range and leap seconds fall back to the original Chrono `Display`
value, including its original offset for timezone-aware values.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-runtime
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-runtime.svg?label=gpui-table-runtime
[crate]: https://crates.io/crates/gpui-table-runtime
[project]: https://github.com/stayhydated/gpui-table

`ResolvedTableColumn<Row>` appends caller-rendered presentation for an executable
resolved key. Generated delegates validate presentation keys and route both cells
and column-header sorting through that retained typed context. Use
`set_table_resolved_columns` to refresh presentation while retaining the selected
record. Renderers read each current row; formatting remains caller-owned.
