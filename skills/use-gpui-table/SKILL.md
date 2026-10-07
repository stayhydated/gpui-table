---
name: use-gpui-table
description: >-
  Build, debug, or explain application tables using gpui-table derives,
  generated filter state, DataTable composition, loading, localization, or MCP
  queries. For custom filter shape implementations, use
  use-gpui-table-component-shapes.
---

# Use gpui-table

## Workflow

1. Inspect the application's manifest and an existing table before choosing
   features or syntax. Generated code uses `gpui_kit` directly; preserve the
   application's existing `gpui-kit` dependency source.
2. Use `gpui-table` as the facade. Add
   `gpui-table-component` when the table renders built-in filters or
   `TableStatusBar`.
3. Derive `GpuiTable` on a named row struct. Configure columns and
   filters at the field, then construct the generated delegate and
   `TableState`.
4. Render `gpui_kit::component::table::DataTable` and place generated
   filter elements in the application's existing layout.
5. Add loading, localization, saved presets, or MCP only when the requested
   behavior needs them.
6. Follow existing application architecture for data access, async work,
   routing, localization, and errors. Keep those responsibilities outside the
   generated row model where the repository already does so.

## Choose the public surface

| Task | Public surface |
|---|---|
| Derive a table | `gpui_table::GpuiTable` |
| Render built-in filters | `gpui-table-component` shapes |
| Define facet values | `Filterable` |
| Render a value object | `TableCell` |
| Load more rows | `TableLoader` plus `#[gpui_table_impl]` |
| Render a custom cell | field-level `style = path` |
| Control visible rows | generated delegate filter and row-scope methods |
| Order loaded rows | `runtime::set_table_ordering`, `sort::SortOrder` and `SortEditor` |
| Sort calculated values | `sort_key = path` paired with the same calculation in `style` |
| Retain record selection | `row_id = "id"` and `runtime::TableRowSelection` |
| Save filter state | generated `FilterValues` JSON methods |
| Expose rows over MCP | `#[gpui_table(mcp)]` and `#[mcp_query]` |
| Adapt or implement a filter shape | `use-gpui-table-component-shapes` |

## Select features deliberately

- `derive` and `chrono` are enabled by default.
- Enable `rust_decimal` for numeric range filters, including integer
  fields.
- Enable `fluent` for typed localized table and facet labels.
- Enable `inventory` for registered `GpuiTableShape`
  metadata.
- Enable `mcp` for generated query tools; it also enables
  `inventory`.
- Enable `spacetimedb` for supported SpacetimeDB temporal filters.
- Match the corresponding `gpui-table-component` feature when a
  built-in component owns the shape.

## Preserve generated contracts

For a row named `User`, use the generated
`UserTableDelegate`, `UserTableColumn`, and, with
`filters`, `UserFilterEntities` and
`UserFilterValues`.

Declare every filter with an explicit shape or configured shape expression.
Use `#[gpui_table(filters)]` for rendered filters, or
`#[gpui_table(mcp)]` when the filters exist only as MCP arguments.

Keep loader flags coherent: set `loading` before starting work, clear
it on completion, and set `eof` only when no later page exists.
Initialize `gpui_table_component::i18n` before rendering localized
components.

For MCP, keep query execution in application code. Local row sources return
`Vec<Row>` or `Result<Vec<Row>, E>`; backend handlers
accept `TableQuery<Row>` and return
`Result<TableQueryResult<Row>, E>`.

Use `gpui_table::mcp::tool_registry()` when the host needs the
inventory-discovered MCP definitions and handlers directly. Keep the server
running for the host's intended session lifetime.

## Ordering contracts

Use stable public column keys, including `col` overrides, in `SortOrder`.
Declare `row_id = "id"` for stable tie-breaking and record selection. Apply
controlled orders through `runtime::set_table_ordering`; delegate sorting
covers loaded rows. Remote sources must order the complete result before
pagination. Local MCP `filter_rows` returns a `Result` after filtering, ordering
and pagination. Backend handlers receive `query.ordering` and own execution.

A calculated `sort_key` returns `Option<Key>` or `Result<Option<Key>, SortError>`.
Reuse its calculation in the cell `style` renderer. Refresh the ordered view
with `refresh_filtered_rows()` after in-place changes and inspect `sort_error()`
when inputs can fail. Capture/restore `TableRowSelection` around source changes.
Use existing UI composition for `SortEditor`; it emits controlled changes and
owns no persistence, backend or formula language.

## Load focused patterns

Read [references/patterns.md](references/patterns.md) when implementation needs
copyable patterns for table setup, generated filters, loading, cells,
localization, or MCP. Use `use-gpui-table-component-shapes` for
adapter shapes, custom shape contracts, or custom MCP filter decoding.
