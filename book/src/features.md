# Features and integration crates

Start with the `gpui-table` facade and enable only the contracts required by
the row types.

| Facade feature | Default | Use it for |
|---|---:|---|
| `derive` | Yes | `GpuiTable`, `Filterable`, `TableCell`, `GpuiTableFilterShape`, and `gpui_table_impl` |
| `chrono` | Yes | Chrono cells and date-range filter support |
| `rust_decimal` | No | Numeric range filters, including filters over integer fields |
| `fluent` | No | Typed localized table and facet labels |
| `spacetimedb` | No | Supported SpacetimeDB temporal range filtering |
| `inventory` | No | `GpuiTableShape` registration for tooling and code generation |
| `mcp` | No | Experimental generated MCP query tools; this also enables `inventory` |

Use `gpui-table-component` for rendered filters, `TableStatusBar`, reset
controls, and manual filter composition. Its `chrono` and `rust_decimal`
features are enabled by default. Enable its `mcp` feature when an MCP-enabled
row declares a built-in filter shape.

## Choose an integration crate

Most application code should not depend on the lower-level crates directly:

| Crate | Use it when |
|---|---|
| `gpui-table-core` | Typed filter semantics must run without GPUI |
| `gpui-table-runtime` | Generic code targets row, loader, cell, or generated-filter runtime traits |
| `gpui-table-schema` | UI-neutral table registry metadata is consumed by tooling |
| `gpui-table-derive` | Proc macros must be integrated without the facade |
| `gpui-table-mcp` | A custom MCP server or lower-level query registry is required |
| `gpui-table-prototyping-core` | Inventory metadata drives story or scaffold generation |

Filter-shape feature errors identify the field and missing facade capability.
Match the facade feature first, then match the corresponding component feature
when the shape is implemented by `gpui-table-component`.
