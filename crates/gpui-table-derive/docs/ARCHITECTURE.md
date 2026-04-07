# Architecture

## Purpose

`gpui-table-derive` contains the proc-macros that generate table delegates,
columns, filters, and optional registry metadata.

## Entry points

- `#[derive(GpuiTable)]`
  - Generates `TableRowMeta`, `TableRowStyle`, column enums, and a
    `TableDelegate` implementation.
  - Generates `TableRowContextMenu` (default no-op) unless
    `#[gpui_table(custom_context_menu)]` is set.
  - Optionally generates a default row context-menu link item when
    context-menu attributes are configured:
    - row-id source: `context_menu_row_id = "..."` or field `#[gpui_table(context_menu_id)]`
    - route source: `context_menu_route = "...{id}..."` or `context_menu_route_fn = path::to_fn`
    - optional label source: `context_menu_label = "..."` or `context_menu_label_fn = path::to_fn`
  - Implements `TableDataLoader` for the generated delegate (load-more or no-op).
  - Optionally generates filter entities/values when `#[gpui_table(filters)]`
    is enabled.
  - Optionally registers a `GpuiTableShape` in the inventory when the
    `inventory` feature is enabled.
- `#[proc_macro_derive(TableCell)]`
  - Convenience derive for newtypes/enums that delegate to an inner `TableCell`.
- `#[gpui_table_impl]`
  - Attribute macro that wires load-more behavior into a generated delegate.

## Module map

- `lib.rs`
  - Macro entry points and expansion logic
  - Validates configuration errors early (e.g. invalid `fixed`, invalid
    `number_range`, field `filter(...)` without struct `#[gpui_table(filters)]`)
- `components.rs`
  - Parses filter configuration attributes (text/number/date/faceted)
- `impl_attr.rs`
  - Parses `#[gpui_table_impl]` blocks and validates load-more signatures
- `__crate_paths/` (generated)
  - Provides stable paths to external crates; do not edit by hand

## Data flow

1. Attributes on the row struct and its fields are parsed via `darling`.
1. The macro expands into column enums, `TableRowMeta`/`TableRowStyle`, and
   `TableDelegate` implementations.
1. Generated delegates route `TableDelegate::context_menu(...)` through the
   selected typed row via `TableRowContextMenu`.
1. When context-menu derive attributes are present, generated
   `TableRowContextMenu` appends a link entry resolved from the selected row-id
   field (template replacement or runtime function path).
1. Filter metadata expands into `FilterEntities`, `FilterValues`, and
   `Matchable` implementations, plus grouped filter render helpers
   (text/number/faceted/date/all), a localized reset-button binding, and
   single-action filter reset wiring.
1. When filters are enabled, generated delegates maintain a filtered row-index
   cache and expose `set_filter_values(...)` / `clear_filter_values(...)`.
   Generated helpers wire filter changes directly into `TableState`:
   - `FilterEntities::build_for_table(...)` for client-side interactive
     filtering with `DataTable`.
   - `FilterEntities::build_for_table_loader(...)` for `TableDataLoader`-driven
     server-side reloads.
   - `FilterEntities::build_for_table_loader_with(...)` when pre-reload delegate
     reset behavior needs customization.
1. If `inventory` is enabled, a `GpuiTableShape` is registered for tooling.
1. Generated filter code references runtime dependencies through
   `gpui_table::__deps` and emits marker-trait assertions so missing
   `gpui-table` features (`rust_decimal`, `chrono`) fail with direct diagnostics.

## Feature flags

- `fluent`: generates localized titles via `es-fluent` helpers.
- `inventory`: registers table shapes for prototyping/codegen.

## Notes

- `__crate_paths` is generated via `just update_crate_paths` and should remain
  untouched.
