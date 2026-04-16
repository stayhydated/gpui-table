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
- `spacetimedb`: enables `gpui-table-core` conversions for SpacetimeDB `Timestamp` and `TimeDuration` in range filters.

`chrono`, `rust_decimal`, and `spacetimedb` are also forwarded into the derive
crate so unsupported range-filter configurations fail during macro expansion
with direct diagnostics instead of later missing-symbol errors.

## Extension points

- Implement `TableRowStyle` for custom rendering.
- Implement `TableRowContextMenu` for row context-menu composition.
- Use `TableRowGeneratedContextMenu` to compose derive-generated menu links
  with custom menu actions.
- Or derive a route link context-menu entry with
  `#[gpui_table(context_menu_row_id = \"...\", context_menu_route = \"...{id}...\")]`,
  or field `#[gpui_table(context_menu_id)]` plus runtime `context_menu_route_fn`.
- Implement `TableLoader` or `TableDataLoader` for load-more behavior.
