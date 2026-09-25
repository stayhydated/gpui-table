# gpui-table-component

[![Codecov: gpui-table-component][codecov-badge]][codecov]
[![crates.io: gpui-table-component][crate-badge]][crate]

`gpui-table-component` provides the built-in GPUI table filters,
adapter shapes, reset control, and `TableStatusBar` for applications using
the [`gpui-table`][project] derive workflow.

Use the same components directly when the application owns the filter layout.
The default features enable date and numeric range filters; enable `mcp`
when those shapes are used by MCP query tables.

## Example

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    name: String,
}
```

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table-component
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table-component.svg?label=gpui-table-component
[crate]: https://crates.io/crates/gpui-table-component
[project]: https://github.com/stayhydated/gpui-table
