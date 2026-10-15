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

Use the lower-level crates for reusable integrations and tooling:

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

## Generate table stories from metadata

Use `gpui-table-schema` and `gpui-table-prototyping-core` when a tool needs
Rust scaffolding for registered row types. Enable the facade's `inventory`
feature on the row-model crate and link that crate into the generator so its
`GpuiTableShape` registrations are available.

The [reference generator](https://github.com/stayhydated/gpui-table/blob/master/examples/prototyping/src/main.rs)
implements `TableLayout` to assemble `TableParts` into a Storybook view. It
passes each registered shape to `TableShapeAdapter::new(shape, true)` and
calls `try_generate_file(&layout)`. Use `try_parts()` instead when the tool
needs individual fragments for its own template. Handle `TableCodegenError`
for invalid identifiers, source paths, or imports instead of using the
panicking convenience methods.

From a repository checkout with the example's dependencies available, run:

```sh
cargo run -p prototyping --locked
```

This replaces generated Rust stories in `examples/prototyping/output` and
writes their module index. Preserve any manual work before running it; make
lasting changes in the row metadata or `StorybookLayout`, then regenerate.
Successful output names each generated file and ends with
`Table story generation complete.` Compile the application that consumes the
generated files to check their imports and feature requirements; generating a
syntax tree alone does not prove that its referenced types are available.
