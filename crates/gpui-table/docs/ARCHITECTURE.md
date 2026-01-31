# Architecture

## Purpose
`gpui-table` is the facade crate for the workspace. It re-exports the core traits,
proc-macro derives, and (optionally) the GPUI filter components so applications
can depend on a single crate.

## Structure
- `lib.rs`
  - Re-exports `gpui-table-core` as the public API surface.
  - Re-exports `gpui-table-derive` when the `derive` feature is enabled.
  - Exposes a `component` module (feature gated) that re-exports filter UI
    components (`TextFilter`, `FacetedFilter`, `NumberRangeFilter`,
    `DateRangeFilter`), their extension traits, and `TableFilterComponent`
    from `gpui-table-component`. (`TableStatusBar` is not re-exported.)

## How it fits
1. You derive `GpuiTable` on a row type.
2. The derive macro (from `gpui-table-derive`) generates a delegate and metadata
   based on traits from `gpui-table-core`.
3. If the `component` feature is enabled, filter components are available under
   `gpui_table::component` for the generated filter entities to use.

## Feature flags
- `derive` (default): enables `GpuiTable` and `TableCell` derives.
- `component`: exposes filter UI components under `gpui_table::component`.
- `inventory`: enables registry metadata for prototyping/codegen.
- `fluent`: integrates with `es-fluent` for localized titles/labels.
- `chrono`, `rust_decimal`: adds `TableCell` + filter support for those types.

## Extension points
- Implement `TableRowStyle` for custom rendering.
- Implement `TableLoader` or `TableDataLoader` for load-more behavior.
