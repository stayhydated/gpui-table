# gpui-table-prototyping-core

[![Codecov: gpui-table-prototyping-core][codecov-badge]][codecov]
[![crates.io: gpui-table-prototyping-core][crate-badge]][crate]

`gpui-table-prototyping-core` turns inventory-registered
`GpuiTableShape` metadata into Rust syntax trees for table stories and
scaffolding in the [`gpui-table`][project] ecosystem.

Use `TableShapeAdapter` with a custom `TableLayout` when a
tool needs validated identifiers, imports, field initializers, and render
fragments while retaining control of the generated file. Prefer the
`try_*` APIs so invalid metadata returns `TableCodegenError`.

The [reference generator][reference-generator]
creates Storybook views from the example row models.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-prototyping-core
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-prototyping-core.svg?label=gpui-table-prototyping-core
[crate]: https://crates.io/crates/gpui-table-prototyping-core
[project]: https://github.com/stayhydated/gpui-table
[reference-generator]: https://github.com/stayhydated/gpui-table/blob/master/examples/prototyping/src/main.rs
