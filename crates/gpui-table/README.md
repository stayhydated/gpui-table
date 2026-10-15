# gpui-table

[![Codecov: gpui-table][codecov-badge]][codecov]
[![crates.io: gpui-table][crate-badge]][crate]

`gpui-table` is the application-facing facade for strongly typed
GPUI tables. It re-exports the table derives, core filter and ordering semantics, runtime
traits, schema types, and optional MCP integration. Built-in filter widgets live
in [`gpui-table-component`][components].

Requires Rust 1.99 or newer. Generated UI code uses `gpui-kit` 0.7 directly,
so the application must expose it as a direct dependency.

Generated delegates expose `set_allowed_sort_columns` to restrict both header
controls and direct ordering to source-supported keys. An empty list disables
ordering; configure restrictions before presenting a delegate.

`sort::ResolvedSortColumns` binds application-resolved keys to typed extraction.
`runtime::set_table_resolved_sort_columns` applies a replacement while preserving
selection. Attach `runtime::ResolvedTableColumn<Row>` for caller-rendered cells
and use `runtime::set_table_resolved_columns` when replacing their presentation.
The application owns calculation definitions, formatting, permissions, and
complete-result backend ordering.

## Features

The default feature set is `derive` and `chrono`. Macro re-exports require
`derive`; match integration features on `gpui-table-component` when using its
corresponding shapes.

| Feature | Default | Purpose |
| --- | --- | --- |
| `derive` | Yes | Re-exports the table, cell, filter, and supporting proc macros. |
| `chrono` | Yes | Enables Chrono cell rendering and date-filter support across core, derive, runtime, and an enabled MCP integration. |
| `fluent` | No | Enables typed Fluent labels in core and generated tables and facets. |
| `inventory` | No | Emits inventory-registered table metadata for tooling. |
| `mcp` | No | Enables the experimental MCP integration and `inventory`, plus MCP macro generation when derive support is present. |
| `rust_decimal` | No | Enables decimal cell rendering and numeric range-filter conversions, including integer and float fields. |
| `spacetimedb` | No | Enables supported SpacetimeDB temporal conversions in core, derive, runtime, and an enabled MCP integration. |

## Example

With the default `derive` feature, this complete row-model example generates
`UserTableDelegate` and `UserTableColumn` and retains one source row:

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
struct User {
    #[gpui_table(sortable, width = 180.)]
    name: String,
}

fn main() {
    let delegate = UserTableDelegate::new(vec![User { name: "Ada".into() }]);
    assert_eq!(delegate.rows.len(), 1);
}
```

Rendering requires the application's GPUI window, `TableState`, and
`DataTable` layout. See the [complete table view][table-view] for composition
and the [API reference][api] for contracts and generated-code support.

[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg?component=gpui-table
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[crate-badge]: https://img.shields.io/crates/v/gpui-table.svg?label=gpui-table
[crate]: https://crates.io/crates/gpui-table
[components]: https://github.com/stayhydated/gpui-table/blob/master/crates/gpui-table-component/README.md
[table-view]: https://github.com/stayhydated/gpui-table/blob/master/examples/some-lib-tables/src/tables/user.rs
[api]: https://docs.rs/gpui-table/latest/gpui_table/
