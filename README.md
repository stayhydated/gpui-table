# gpui-table

[![Build Status](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-table/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/gpui-table/graph/badge.svg)](https://codecov.io/github/stayhydated/gpui-table)
[![Docs](https://docs.rs/gpui-table/badge.svg)](https://docs.rs/gpui-table/)
[![Crates.io](https://img.shields.io/crates/v/gpui-table.svg)](https://crates.io/crates/gpui-table)

`gpui-table` derives strongly typed
[`gpui-kit`](https://github.com/longbridge/gpui-kit) tables
from Rust row types. It generates column metadata and delegates, with opt-in
typed filters, incremental loading, localization, registry metadata, and MCP
query tools.

## Add it to an application

```toml
[dependencies]
gpui-table = "0.6"
gpui-table-component = "0.6"
```

Keep `gpui` and `gpui-kit` as direct dependencies
using the source or versions selected by your application.

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

## Learn and explore

- [User guide](https://stayhydated.github.io/gpui-table/book/)
- [API documentation](https://docs.rs/gpui-table/)
- [Runnable examples](examples/README.md)
- [Project site](https://stayhydated.github.io/gpui-table/)

Licensed under MIT or Apache-2.0.
