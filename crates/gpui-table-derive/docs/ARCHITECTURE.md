# Architecture

## Purpose
`gpui-table-derive` contains the proc-macros that generate table delegates,
columns, filters, and optional registry metadata.

## Entry points
- `#[derive(GpuiTable)]`
  - Generates `TableRowMeta`, `TableRowStyle`, column enums, and a
    `TableDelegate` implementation.
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
- `components.rs`
  - Parses filter configuration attributes (text/number/date/faceted)
- `impl_attr.rs`
  - Parses `#[gpui_table_impl]` blocks and validates load-more signatures
- `__crate_paths/` (generated)
  - Provides stable paths to external crates; do not edit by hand

## Data flow
1. Attributes on the row struct and its fields are parsed via `darling`.
2. The macro expands into column enums, `TableRowMeta`/`TableRowStyle`, and
   `TableDelegate` implementations.
3. Filter metadata expands into `FilterEntities`, `FilterValues`, and
   `Matchable` implementations, plus grouped filter render helpers
   (text/number/faceted/date/all).
4. If `inventory` is enabled, a `GpuiTableShape` is registered for tooling.

## Feature flags
- `fluent`: generates localized titles via `es-fluent` helpers.
- `inventory`: registers table shapes for prototyping/codegen.

## Notes
- `__crate_paths` is generated via `just update_crate_paths` and should remain
  untouched.
