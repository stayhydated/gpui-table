# Architecture

## Purpose

`gpui-table` is the facade crate for the workspace. It re-exports the core traits
and proc-macro derives so applications can depend on a single crate for table
metadata and macro generation.

## Structure

- `lib.rs`
  - Re-exports `gpui-table-core` as the public API surface.
  - Re-exports `gpui-table-derive` when the `derive` feature is enabled.
  - Exposes hidden `__deps` re-exports used by macro-generated code
    (`gpui-table-component`, and feature-gated `chrono` / `rust_decimal`).

## How it fits

1. You derive `GpuiTable` on a row type.
1. The derive macro (from `gpui-table-derive`) generates a delegate and metadata
   based on traits from `gpui-table-core`.
1. If `#[gpui_table(filters)]` is enabled, generated filter entities integrate
   with components re-exported through `gpui_table::__deps`.

## Feature flags

- `derive` (default): enables `GpuiTable` and `TableCell` derives.
- `chrono` (default): adds `TableCell` + filter support for date types.
- `inventory`: enables registry metadata for prototyping/codegen.
- `fluent`: integrates with `es-fluent` for localized titles/labels.
- `rust_decimal`: adds `TableCell` + filter support for decimal types.

`chrono` and `rust_decimal` also drive hidden feature markers used by generated
code to emit clear compile errors when `date_range` / `number_range` filters
are used without enabling the corresponding `gpui-table` feature.

## Extension points

- Implement `TableRowStyle` for custom rendering.
- Implement `TableRowContextMenu` for row context-menu composition.
- Implement `TableLoader` or `TableDataLoader` for load-more behavior.
