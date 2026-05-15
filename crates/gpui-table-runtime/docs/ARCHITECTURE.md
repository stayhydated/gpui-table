# gpui-table-runtime Architecture

## Purpose

`gpui-table-runtime` owns the GPUI-facing runtime contracts for the workspace:
row rendering traits, load-more traits, default cell rendering, and the stable
runtime facade that generated filter code compiles against.

## Dependency Edges

- Depends on `gpui` and `gpui-component` for table integration and rendering.
- Depends on `gpui-table-core` for filter semantics.
- Depends on `gpui-table-component` for the built-in filter UI.
- Depends on `gpui-table-schema` for filter metadata types used by the runtime.

## Module Map

- `src/lib.rs`
  - Re-export hub for the runtime surface.
  - Defines hidden `__private::LoadMoreDelegate` bridge for macro output.
- `src/cell.rs`
  - `TableCell`, value-object wrappers, and the built-in cell renderers for
    common value types.
- `src/row.rs`
  - `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`,
    `TableRowGeneratedContextMenu`, `default_render_cell`, and `default_render_row`.
- `src/load.rs`
  - `TableLoader`, `TableDataLoader`, and the hidden `LoadMoreDelegate` bridge.
- `src/generated_filters.rs`
  - Stable runtime target for generated filter code.
  - Re-exports built-in filter components, localization helpers, and generic
    filter traits.

## Internal Contracts

- `generated_filters` is a compatibility surface, not just a convenience
  module. The derive crate should keep targeting it instead of reaching into
  `gpui-table-component` directly.
- Generated Fluent labels and messages route through `generated_filters`
  localization helpers so derive output does not depend on component i18n paths.
  Runtime render paths pass GPUI context to those helpers; context-free table
  metadata uses explicit fallback helpers.
- `__private::LoadMoreDelegate` is hidden from user docs but is part of the
  generated-code contract for load-more tables.
- `default_render_cell` and `default_render_row` are the baseline rendering
  behavior that custom `TableRowStyle` impls compose around.
- `TableDataLoader` is implemented by generated delegates even when
  `load_more` is not enabled. In that case the generated implementation is a
  no-op, which keeps generic loader code uniform.

## Data Flow

1. `gpui-table-derive` emits delegates, row traits, and filter code against the
   types exported from this crate.
1. Built-in filters are instantiated through `generated_filters`, which
   forwards to `gpui-table-component`.
1. Client-side filtering stores typed wrapper values on the generated delegate
   and then evaluates `Matchable<F>` against in-memory rows.
1. Loader-driven tables also store those wrapper values on the delegate, then
   call `load_data(...)` so application code can translate them into requests.

## Feature Gates

- `chrono` enables date cell rendering, date filter runtime support, and the
  supporting date-formatting stack.
- `rust_decimal` enables numeric range runtime support.
- `spacetimedb` forwards supported temporal conversions through the core layer.
