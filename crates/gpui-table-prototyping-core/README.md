# gpui-table-prototyping-core

`gpui-table-prototyping-core` turns inventory-registered
`GpuiTableShape` metadata into Rust syntax trees for table stories and
scaffolding.

Use `TableShapeAdapter` with a custom `TableLayout` when a
tool needs validated identifiers, imports, field initializers, and render
fragments while retaining control of the generated file. Prefer the
`try_*` APIs so invalid metadata returns `TableCodegenError`.

- [Feature and crate guide](https://stayhydated.github.io/gpui-table/book/features.html)
- [Reference generator](https://github.com/stayhydated/gpui-table/blob/master/examples/prototyping/src/main.rs)
- [API documentation](https://docs.rs/gpui-table-prototyping-core/)
