# Architecture

## Purpose

`gpui-table-runtime` holds the GPUI-facing runtime layer for the workspace:
table rendering traits, load-more wiring, default cell rendering, and the
stable runtime facade that generated filter code targets.

## Module map

- `lib.rs`
  - Re-exports the runtime surface
  - Exposes hidden `__private::LoadMoreDelegate` bridge for macro internals
- `cell.rs`
  - `TableCell` trait
  - Built-in rendering implementations for primitive, decimal, and date/time values
- `row.rs`
  - `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`, `TableRowGeneratedContextMenu`
  - `default_render_cell`, `default_render_row`
- `load.rs`
  - `TableLoader`, `TableDataLoader`, `LoadMoreDelegate`
- `generated_filters.rs`
  - Stable runtime target for derive-generated filter code
  - Re-exports built-in filter components from `gpui-table-component`
  - Forwards `chrono` / `rust_decimal` gating to the date/number filter exports
  - `FilterEntitiesExt` trait for generated filter entity collections
  - `QueryFilterValue` for serializing generated wrapper fields in loader flows

## Data flow

1. `gpui-table-derive` generates row/delegate code against traits from this crate.
1. Generated filter code targets `generated_filters` instead of directly coupling
   itself to `gpui-table-component`.
1. Loader-oriented code can read generated filter wrappers and serialize
   supported fields through `generated_filters::QueryFilterValue`.
1. Consumers normally access this layer through the `gpui-table` facade crate.
