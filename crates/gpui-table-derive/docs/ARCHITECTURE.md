# Architecture

## Purpose

`gpui-table-derive` contains the proc-macros that generate table delegates,
columns, filters, and optional registry metadata.

## Entry points

- `#[derive(GpuiTable)]`
  - Generates `TableRowMeta`, `TableRowStyle`, column enums, and a
    `TableDelegate` implementation.
  - Generates `TableRowGeneratedContextMenu` (default no-op or configured link).
  - Generates `TableRowContextMenu` unless `#[gpui_table(custom_context_menu)]`
    is set; generated impl forwards to `TableRowGeneratedContextMenu`.
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
  - Unit enums render via `EsFluent*` when derived, then `Display`, then the
    variant name as a fallback.
- `#[gpui_table_impl]`
  - Attribute macro that wires load-more behavior into a generated delegate.

## Module map

- `lib.rs`
  - Macro entry points and expansion logic
  - Validates configuration errors early (e.g. invalid `fixed`, invalid
    `number_range`, missing `chrono` / `rust_decimal` / `spacetimedb` feature
    requirements, unsupported built-in filter/type combinations, field
    `filter(...)` without struct `#[gpui_table(filters)]`)
- `components.rs`
  - Parses filter configuration attributes (text/number/date/faceted)
  - `number_range(...)` decimal options preserve source spans, accept numeric
    literals or quoted decimal strings, and feed compile-time validation/codegen
- `gpui_table/filter_codegen/`
  - Shared filter type-token generation, option-chain generation, and
    field/type validation helpers used during `GpuiTable` expansion
- `filter_entities.rs`
  - Generates `XxxFilterEntities`, `XxxFilterValues`, and filter builder/render helpers
- `filter_matching.rs`
  - Generates `Matchable<XxxFilterValues>` implementations
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
   `TableRowGeneratedContextMenu` appends a link entry resolved from the
   selected row-id field (template replacement or runtime function path).
   This remains composable when users implement `TableRowContextMenu` manually.
1. Filter metadata expands into `FilterEntities`, `FilterValues`, and
   `Matchable` implementations, plus grouped filter render helpers
   (text/number/faceted/date/all), a localized reset-button binding, and
   single-action filter reset wiring.
   Generated `FilterEntities` expose inherent `read_values(...)` /
   `all_filters(...)` methods so consumers do not need a trait import for the
   common render/read path.
1. When filters are enabled, generated delegates maintain a filtered row-index
   cache and expose `set_filter_values(...)` / `clear_filter_values(...)`.
   Generated helpers wire filter changes directly into `TableState`:
   - `FilterEntities::build_for_table(...)` for client-side interactive
     filtering with `DataTable`.
   - `FilterEntities::build_for_table_loader(...)` for `TableDataLoader`-driven
     server-side reloads.
   - `FilterEntities::build_for_table_loader_with(...)` when pre-reload delegate
     reset behavior needs customization.
     Generated `FilterValues` use typed wrappers from `gpui_table::core::filter`,
     and those fields can usually be serialized for server-side queries via
     `gpui_table::runtime::generated_filters::QueryFilterValue`.
1. If `inventory` is enabled, a `GpuiTableShape` is registered for tooling.
1. Generated filter code now targets `gpui_table::runtime::generated_filters`
   for built-in components and filter runtime traits.
1. Feature-gated external types such as `chrono::NaiveDate` and
   `rust_decimal::Decimal` still route through `gpui_table::__deps`, while
   missing `gpui-table` feature requirements are rejected earlier during macro
   expansion.
1. With `rust_decimal` enabled, `number_range(min/max/step)` values are parsed
   during macro expansion so invalid decimal literals, non-positive steps, and
   inverted ranges fail before code generation.
1. Built-in filter/type mismatches are rejected before code generation where
   the derive can prove the combination is impossible (for example `text()` on
   `bool`, `faceted()` on `Option<T>`, or `date_range()` on `String`), while
   still allowing local user types that may implement the required runtime
   traits.

## Feature flags

- `chrono`: forwarded from `gpui-table` so `date_range` support can be
  validated during macro expansion.
- `fluent`: generates localized titles via `es-fluent` helpers.
- `inventory`: registers table shapes for prototyping/codegen.
- `rust_decimal`: forwarded from `gpui-table` so `number_range` support can be
  validated during macro expansion.
- `spacetimedb`: forwarded from `gpui-table` so SpacetimeDB range-filter usage
  can be validated during macro expansion.

## Notes

- `__crate_paths` is generated via `just update_crate_paths` and should remain
  untouched.
