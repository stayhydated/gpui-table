# gpui-table-runtime

`gpui-table-runtime` contains the GPUI-facing contracts targeted by
generated table code: row metadata and rendering, cell rendering, loading, filter
shapes, and generic helpers for generated filter collections.

Depend on this crate directly when writing reusable integrations over
`TableRowMeta`, `TableLoader`,
`GpuiTableFilterShape`, or `FilterEntitiesExt`. Application
tables should normally use the `gpui-table` facade.

- [Feature and crate guide](https://stayhydated.github.io/gpui-table/book/features.html)
- [Custom filter guide](https://stayhydated.github.io/gpui-table/book/custom_filters.html)
- [API documentation](https://docs.rs/gpui-table-runtime/)
