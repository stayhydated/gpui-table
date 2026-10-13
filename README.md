# gpui-table

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: gpui-table][gpui-table-badge]][gpui-table-crate]

`gpui-table` derives strongly typed
[`gpui-kit`](https://github.com/longbridge/gpui-kit) tables
from Rust row types for GPUI application developers. It generates column
metadata and delegates, with opt-in typed filters, incremental loading,
localization, registry metadata, ordered sorting, calculated keys, and MCP query tools.

Requires Rust 1.99 or newer for all public crates. Generated UI code uses
`gpui-kit` 0.7 directly, so applications must expose it as a direct dependency.

Start with [`gpui-table`][gpui-table-readme] for derives and table contracts. Add
[`gpui-table-component`][gpui-table-component-readme] for built-in filter widgets.

## Crates

| Crate | Purpose |
| --- | --- |
| [`gpui-table`][gpui-table-readme] | Application-facing facade and derive re-exports |
| [`gpui-table-component`][gpui-table-component-readme] | Built-in filter widgets and table support components |
| [`gpui-table-core`][gpui-table-core-readme] | UI-neutral filter values and matching semantics |
| [`gpui-table-derive`][gpui-table-derive-readme] | Proc macros for tables, cells, filters, and MCP queries |
| [`gpui-table-mcp`][gpui-table-mcp-readme] | Typed MCP query contracts and server composition |
| [`gpui-table-prototyping-core`][gpui-table-prototyping-core-readme] | Rust syntax generation from registered table metadata |
| [`gpui-table-runtime`][gpui-table-runtime-readme] | GPUI row, cell, loading, and filter-shape contracts |
| [`gpui-table-schema`][gpui-table-schema-readme] | UI-neutral table and filter registry metadata |

## Example

With the facade's default `derive` feature and a direct `gpui-kit` dependency,
this row declaration generates `UserTableDelegate` and `UserTableColumn`:

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

The application supplies its GPUI window, `TableState`, and layout around
`gpui_kit::component::table::DataTable`. The [complete table view][table-view]
composes those values with filters and loading; the [runnable examples][examples]
cover additional table and MCP workflows.

[ci-badge]: https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/gh/stayhydated/gpui-table/branch/master/graph/badge.svg
[codecov]: https://codecov.io/gh/stayhydated/gpui-table
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/gpui-table/book/
[gpui-table-badge]: https://img.shields.io/crates/v/gpui-table.svg?label=gpui-table
[gpui-table-crate]: https://crates.io/crates/gpui-table
[gpui-table-readme]: crates/gpui-table/README.md
[gpui-table-component-readme]: crates/gpui-table-component/README.md
[gpui-table-core-readme]: crates/gpui-table-core/README.md
[gpui-table-derive-readme]: crates/gpui-table-derive/README.md
[gpui-table-mcp-readme]: crates/gpui-table-mcp/README.md
[gpui-table-prototyping-core-readme]: crates/gpui-table-prototyping-core/README.md
[gpui-table-runtime-readme]: crates/gpui-table-runtime/README.md
[gpui-table-schema-readme]: crates/gpui-table-schema/README.md
[table-view]: https://github.com/stayhydated/gpui-table/blob/master/examples/some-lib-tables/src/tables/user.rs
[examples]: https://github.com/stayhydated/gpui-table/blob/master/examples/README.md
