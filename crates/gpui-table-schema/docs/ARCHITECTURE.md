# gpui-table-schema Architecture

## Purpose

`gpui-table-schema` holds the UI-neutral metadata shared across the workspace.
It should remain the lowest-level description of filters and inventory-backed
table shapes so tooling can consume it without inheriting GPUI dependencies.

## Dependency Edges

- Depends only on `component-shape`, `inventory`, and `strum`.
- Must not depend on `gpui`, `gpui-component`, or any runtime-only types.

## Module Map

- `src/lib.rs`
  - Exposes the filter and registry modules.
- `src/filter.rs`
  - `FilterConfig`, `FilterType`, `FacetedFilterOption`, and `FacetedFilterIcon`.
- `src/registry.rs`
  - `GpuiTableShape`, `ColumnVariant`, `FilterVariant`,
    `RegistryFilterType`, `ColumnFixed`, `RustPath`, `RustType`, and the `inventory` re-export.

## Internal Contracts

- Registry structs store only `'static` data so they can be submitted to
  `inventory` directly from macro expansion.
- `ColumnVariant::field_type` uses `component_shape::RustType` so tooling can
  recognize Rust syntax metadata without treating it as an untyped string.
- `FilterVariant::component_path` uses `component_shape::RustPath` to preserve
  the `gpui-table-component` filter component generated for a field.
- `GpuiTableShape::source_path` preserves the original `file!()` path because
  prototyping/codegen uses it to reconstruct import paths.
- `ColumnVariant` and `FilterVariant` are descriptive metadata only. They are
  not intended to carry runtime state.
- `RegistryFilterType` is derived from the generated component's
  `TableFilterComponent::FILTER_TYPE`; it describes registry shape, while
  `FilterVariant::component_path` preserves the concrete widget type.
- `ColumnFixed` uses `snake_case` string conversions through `strum`, which is
  consumed by docs, tooling, and diagnostics.

## Data Flow

1. `gpui-table-derive` emits `FilterConfig` metadata for generated tables.
1. `Filterable` implementations from `gpui-table-core` produce
   `FacetedFilterOption` values that flow through this schema layer.
1. When the `inventory` feature is enabled on `gpui-table`, macro output
   submits `GpuiTableShape` values to the registry.
1. `gpui-table-prototyping-core` then reads those shapes to generate stories or scaffolding.
