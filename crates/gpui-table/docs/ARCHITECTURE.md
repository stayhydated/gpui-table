# Architecture

## Purpose

`gpui-table` is the facade crate for the workspace. Applications can depend on
it alone and get:

- pure filter semantics from `gpui-table-core`
- GPUI runtime traits/helpers from `gpui-table-runtime`
- schema/registry metadata from `gpui-table-schema`
- proc macros from `gpui-table-derive`

## Structure

- `lib.rs`
  - Re-exports the core crate as `gpui_table::core`
  - Re-exports the runtime crate as `gpui_table::runtime`
  - Re-exports the schema crate as `gpui_table::schema`
  - Re-exports derive macros when the `derive` feature is enabled
  - Exposes hidden `__deps` for feature-gated external types (`chrono`, `rust_decimal`)
  - Exposes hidden `__private` load-more bridge for macro-generated code

## How it fits

1. You derive `GpuiTable` on a row type.
1. The derive macro generates row/delegate/filter code against traits exported by
   the facade's explicit `core` / `runtime` / `schema` namespaces.
1. Generated filter code targets `gpui_table::runtime::generated_filters`
   instead of directly hard-coding the component crate path.
1. Tooling such as prototyping/codegen consumes schema metadata from
   `gpui_table::schema::registry`.

## Feature flags

- `derive` (default): enables `GpuiTable` and `TableCell` derives.
- `chrono` (default): forwards chrono/date support into `core`, `runtime`, and `derive`.
- `inventory`: enables registry metadata for prototyping/codegen.
- `fluent`: integrates with `es-fluent` for localized titles/labels.
- `rust_decimal`: forwards numeric-range support into `core`, `runtime`, and `derive`.
- `spacetimedb`: forwards SpacetimeDB temporal support into `core`, `runtime`, and `derive`.

## Extension points

- Implement `gpui_table::runtime::TableRowStyle` for custom rendering.
- Implement `gpui_table::runtime::TableRowContextMenu` for row context-menu composition.
- Use `gpui_table::runtime::TableRowGeneratedContextMenu` to compose derive-generated menu links with
  custom menu actions.
- Implement `gpui_table::runtime::TableLoader` or
  `gpui_table::runtime::TableDataLoader` for load-more behavior.
- Use `gpui_table::runtime::generated_filters` as the stable runtime target when
  integrating custom generated-filter flows.
