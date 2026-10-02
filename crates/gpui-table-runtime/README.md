# gpui-table-runtime

[![Codecov: gpui-table-runtime][codecov-badge]][codecov]
[![crates.io: gpui-table-runtime][crate-badge]][crate]

`gpui-table-runtime` contains the GPUI-facing contracts targeted by
generated table code: row metadata and rendering, cell rendering, loading, filter
shapes, and generic helpers for generated filter collections in the
[`gpui-table`][project] ecosystem.

Depend on this crate directly when writing reusable integrations over
`TableRowMeta`, `TableLoader`,
`GpuiTableFilterShape`, or `FilterEntitiesExt`. Application
tables should normally use the `gpui-table` facade.

Chrono cells use localized formatting for values representable by Jiff. Dates
outside Jiff's range and leap seconds fall back to the original Chrono `Display`
value, including its original offset for timezone-aware values.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-runtime
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-runtime.svg?label=gpui-table-runtime
[crate]: https://crates.io/crates/gpui-table-runtime
[project]: https://github.com/stayhydated/gpui-table
