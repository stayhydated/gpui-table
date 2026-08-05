# gpui-table-schema

`gpui-table-schema` defines UI-neutral filter metadata and the
inventory-backed `GpuiTableShape` registry. Tooling can inspect table
IDs, columns, filter categories, Rust type syntax, and component-shape usage
without depending on GPUI.

Use this crate for metadata consumers and generators. Applications that derive
or render tables should normally depend on `gpui-table`.

- [Feature and crate guide](https://stayhydated.github.io/gpui-table/book/features.html)
- [API documentation](https://docs.rs/gpui-table-schema/)
