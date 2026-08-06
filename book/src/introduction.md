# Introduction

`gpui-table` turns a Rust row struct into the delegate and column
metadata required by `gpui-component`'s `DataTable`. Opt-in
attributes add typed filter controls, incremental loading, localized labels,
registry metadata, and MCP query tools while the application keeps ownership of
its rows and query execution.

## Who this guide is for

This guide is for Rust application developers who already have a GPUI
application, window, and view. It assumes familiarity with GPUI entities and
render methods.

Start with the facade and component crates:

```toml
[dependencies]
gpui-table = "0.6"
gpui-table-component = "0.6"
```

Keep `gpui` and `gpui-component` as direct dependencies
using the versions or source selected by your application. Generated table code
refers to both crates by name, and applications render
`gpui_component::table::DataTable` directly.

## Mental model

For a row type named `User`,
`#[derive(GpuiTable)]` generates
`UserTableDelegate` and `UserTableColumn`. Adding
`#[gpui_table(filters)]` also generates
`UserFilterEntities` and `UserFilterValues`.

The generated delegate owns table-facing sorting and visibility over the source
rows. The application still decides:

- where rows come from and when they change
- how `DataTable` and filter controls are laid out
- how loading and backend queries run
- whether table metadata or rows are exposed to external tooling

Continue with [Getting started](getting_started.md) to render a table. Use
[Typed filters](filters.md) for generated filter state,
[Filter components and custom shapes](custom_filters.md) for direct widgets or
domain value types, and [MCP query tools](mcp.md) when non-GPUI clients should
query rows.
