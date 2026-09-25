# gpui-table-schema

[![Codecov: gpui-table-schema][codecov-badge]][codecov]
[![crates.io: gpui-table-schema][crate-badge]][crate]

`gpui-table-schema` defines UI-neutral filter metadata and the
inventory-backed `GpuiTableShape` registry. Tooling can inspect table
IDs, columns, filter categories, Rust type syntax, and component-shape usage
without depending on GPUI in the [`gpui-table`][project] ecosystem.

Use this crate for metadata consumers and generators. Applications that derive
or render tables should normally depend on `gpui-table`.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-schema
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-schema.svg?label=gpui-table-schema
[crate]: https://crates.io/crates/gpui-table-schema
[project]: https://github.com/stayhydated/gpui-table
