# Architecture

## Purpose

`gpui-table` is the facade crate for the workspace. It re-exports the core traits
and proc-macro derives so applications can depend on a single crate for table
metadata and macro generation.

## Structure

- `lib.rs`
  - Re-exports `gpui-table-core` as the public API surface.
  - Re-exports `gpui-table-derive` when the `derive` feature is enabled.

## How it fits

1. You derive `GpuiTable` on a row type.
1. The derive macro (from `gpui-table-derive`) generates a delegate and metadata
   based on traits from `gpui-table-core`.
1. If `#[gpui_table(filters)]` is enabled, generated filter entities integrate
   with components from the separate `gpui-table-component` crate.

## Feature flags

- `derive` (default): enables `GpuiTable` and `TableCell` derives.
- `chrono` (default): adds `TableCell` + filter support for date types.
- `inventory`: enables registry metadata for prototyping/codegen.
- `fluent`: integrates with `es-fluent` for localized titles/labels.
- `rust_decimal`: adds `TableCell` + filter support for decimal types.

## Extension points

- Implement `TableRowStyle` for custom rendering.
- Implement `TableLoader` or `TableDataLoader` for load-more behavior.
