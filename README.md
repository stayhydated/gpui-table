# gpui-table

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: gpui-table][gpui-table-badge]][gpui-table-crate]

`gpui-table` derives strongly typed
[`gpui-kit`](https://github.com/longbridge/gpui-kit) tables
from Rust row types for GPUI application developers. It generates column
metadata and delegates, with opt-in typed filters, incremental loading,
localization, registry metadata, and MCP query tools.

## Crates

| Crate | Purpose | Source |
| --- | --- | --- |
| `gpui-table` | Application-facing facade and derive re-exports | [README][gpui-table-readme] |
| `gpui-table-component` | Built-in filter widgets and table support components | [README][gpui-table-component-readme] |
| `gpui-table-core` | UI-neutral filter values and matching semantics | [README][gpui-table-core-readme] |
| `gpui-table-derive` | Proc macros for tables, cells, filters, and MCP queries | [README][gpui-table-derive-readme] |
| `gpui-table-mcp` | Typed MCP query contracts and server composition | [README][gpui-table-mcp-readme] |
| `gpui-table-prototyping-core` | Rust syntax generation from registered table metadata | [README][gpui-table-prototyping-core-readme] |
| `gpui-table-runtime` | GPUI row, cell, loading, and filter-shape contracts | [README][gpui-table-runtime-readme] |
| `gpui-table-schema` | UI-neutral table and filter registry metadata | [README][gpui-table-schema-readme] |

## Example

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(filters)]
struct User {
    #[gpui_table(
        sortable,
        filter(gpui_table_component::TextFilter)
    )]
    name: String,
}
```

The derive creates `UserTableDelegate`,
`UserTableColumn`, `UserFilterEntities`, and
`UserFilterValues`. The application owns the rows, the
`TableState`, and the layout around
`gpui_kit::component::table::DataTable`.

See the [runnable examples](examples/README.md) for complete table views,
filter controls, loading, and MCP query tools.

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
