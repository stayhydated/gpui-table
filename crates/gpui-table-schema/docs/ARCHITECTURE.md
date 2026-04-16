# Architecture

## Purpose

`gpui-table-schema` holds the static metadata shared across the workspace:
filter configuration, faceted-filter option metadata, and inventory-backed table
shape registry types.

It is intentionally UI-neutral so tooling such as prototyping/codegen can depend
on it without pulling `gpui` runtime dependencies.

## Module map

- `lib.rs`
  - Exposes the schema surface.
- `filter.rs`
  - `FilterConfig`, `FilterType`, `FacetedFilterOption`, `FacetedFilterIcon`
- `registry.rs`
  - `GpuiTableShape`, `ColumnVariant`, `FilterVariant`
  - `RegistryFilterType`, `ColumnFixed`
  - Inventory collection and re-export

## Data flow

1. `gpui-table-derive` emits `FilterConfig` values into generated row metadata.
1. `Filterable` implementations in `gpui-table-core` produce `FacetedFilterOption`
   values for faceted filters.
1. With the `inventory` feature enabled on `gpui-table`, derive-generated code
   submits `GpuiTableShape` items into the registry.
1. `gpui-table-prototyping-core` reads `GpuiTableShape` values to generate
   scaffolding without depending on GPUI runtime crates.
