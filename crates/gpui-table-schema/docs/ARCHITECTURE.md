# gpui-table-schema Architecture

## Purpose

`gpui-table-schema` holds the UI-neutral metadata shared across the workspace.
It should remain the lowest-level description of filters and inventory-backed
table shapes so tooling can consume it without inheriting GPUI dependencies.

## Dependency Edges

- Depends only on `inventory` and `strum`.
- Must not depend on `gpui`, `gpui-component`, or any runtime-only types.

## Module Map

- `src/lib.rs`
  - Exposes the filter and registry modules.
- `src/filter.rs`
  - `FilterConfig`, `FilterType`, `FacetedFilterOption`, and `FacetedFilterIcon`.
- `src/registry.rs`
  - `GpuiTableShape`, `ColumnVariant`, `FilterVariant`,
    `RegistryFilterType`, `ColumnFixed`, and the `inventory` re-export.

## Internal Contracts

- Registry structs store only `'static` data so they can be submitted to
  `inventory` directly from macro expansion.
- `GpuiTableShape::source_path` preserves the original `file!()` path because
  prototyping/codegen uses it to reconstruct import paths.
- `ColumnVariant` and `FilterVariant` are descriptive metadata only. They are
  not intended to carry runtime state.
- `RegistryFilterType` is intentionally smaller than the full runtime filter UI;
  it describes registry shape, not widget configuration.
- `ColumnFixed` uses `snake_case` string conversions through `strum`, which is
  consumed by docs, tooling, and diagnostics.

## Data Flow

1. `gpui-table-derive` emits `FilterConfig` metadata for generated tables.
1. `Filterable` implementations from `gpui-table-core` produce
   `FacetedFilterOption` values that flow through this schema layer.
1. When the `inventory` feature is enabled on `gpui-table`, macro output
   submits `GpuiTableShape` values to the registry.
1. `gpui-table-prototyping-core` then reads those shapes to generate stories or scaffolding.
