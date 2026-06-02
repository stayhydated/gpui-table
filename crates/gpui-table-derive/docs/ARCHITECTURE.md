# gpui-table-derive Architecture

## Purpose

`gpui-table-derive` owns the proc-macro expansion pipeline for the workspace.
It translates row structs, faceted enums, wrapper cells, and load-more impl
blocks into the generated types consumed by the facade/runtime/schema layers.

## Entry Points

- `#[derive(GpuiTable)]`
  - Generates row metadata, column enums, delegates, optional filters, optional
    context-menu helpers, and optional inventory registration.
- `#[derive(Filterable)]`
  - Generates `FilterValue`, `Filterable`, and `variant_name()` for faceted enums.
- `#[derive(TableCell)]`
  - Generates `TableCell` for wrapper structs and unit enums, with optional
    display or formatter overrides through `#[table_cell(...)]`.
- `#[gpui_table_impl]`
  - Attaches load-more behavior to the generated delegate.

## Module Map

- `src/lib.rs`
  - Proc-macro entry points.
- `src/components.rs`
  - Wraps shared filter shape path options and resolves `_` generics against
    field types.
- `src/filterable.rs`
  - Expansion for `#[derive(Filterable)]`.
- `src/table_cell.rs`
  - Expansion for `#[derive(TableCell)]`.
- `src/impl_attr.rs`
  - Parsing and validation for `#[gpui_table_impl]`.
- `src/gpui_table/meta.rs`
  - Shared parsed representation of struct-level and field-level `#[gpui_table(...)]` options.
- `src/gpui_table/expand.rs`
  - Main `GpuiTable` orchestration and high-level validation.
- `src/gpui_table/delegate.rs`
  - Delegate generation, row caches, loader wiring, and related helpers.
- `src/gpui_table/filter_entities.rs`
  - `XxxFilterEntities`, `XxxFilterValues`, and filter builder/render helpers.
- `src/gpui_table/filter_matching.rs`
  - Generated `Matchable<XxxFilterValues>` implementations.
- `src/gpui_table/filter_codegen/`
  - Shared filter token generation, chain helpers, and type validation.

## Expansion Pipeline

1. Parse the input syntax tree with `syn`.
1. Parse `#[gpui_table(...)]` or `#[filter(...)]` attributes with `darling`.
1. Validate structural rules such as:
   - `filter(...)` requires struct-level `#[gpui_table(filters)]`
   - only one context-menu id source is allowed
   - `context_menu_route` and `context_menu_route_fn` are mutually exclusive
   - selected filter shapes implement the declared-shape and field support
     contracts
   - required workspace features are enabled for the selected filter shapes
1. Generate the main row/delegate code.
1. Optionally generate filter entities, filter values, and matching logic.
1. Optionally generate inventory registration.
1. Optionally attach load-more wiring through `#[gpui_table_impl]`.

## Generated Type Contracts

`#[derive(GpuiTable)]` emits a predictable family of symbols:

- `XxxTableColumn`
- `XxxTableDelegate`
- `XxxFilterEntities` when filters are enabled
- `XxxFilterValues` when filters are enabled

Those generated types are consumed by example code, tests, and external users,
so name changes or path changes are semver-sensitive.

Additional generated contracts:

- Filter UI targets `gpui_table::runtime::generated_filters`.
- Fluent table labels, field titles, and faceted labels are emitted as typed
  `es-fluent` label/message calls through the runtime localization helpers.
- Feature-gated external types route through `gpui_table::__deps`.
- Load-more glue routes through `gpui_table::__private::LoadMoreDelegate`.
- Inventory metadata uses `gpui_table::registry::GpuiTableShape` and stores the
  original `file!()` path in `source_path`.

## Internal Contracts

- `filter(...)` accepts a shape type path. The generated code asserts
  `DeclaredGpuiTableFilterShape`, `GpuiTableFilterShape`, and
  `GpuiTableFilterShapeFor<Field>` at the field span.
- Filter shape path extraction uses `component-shape-codegen` helpers; table
  option grammar, duplicate checks, and diagnostics remain owned by this crate.
- Implementing `TableFilterComponent` alone does not make a widget selectable in
  `#[gpui_table(filter(...))]`; the table filter shape contract must also be
  implemented.
- `TableDataLoader` is generated for every delegate so downstream code can call
  a uniform loader surface.
- Filter builder helpers generate both client-side and loader-driven wiring.
  The loader-driven path resets delegate state before reloading unless callers
  opt into the `_with(...)` customization hook.
- `Filterable<bool>` is treated as a normal faceted filter path, not a special
  runtime exception.
- Optional and vector faceted fields are generated as `FacetedValue<T>` and
  `gpui_table_component::FacetedFilter<T>` for `Option<T>` and `Vec<T>` fields.
  Matching treats `None` or a vector without any selected value as a non-match
  only when the facet is active.

## Test And Generated Surfaces

- Compile-fail coverage lives under `crates/gpui-table/tests/ui`.
- Snapshot coverage for table rendering lives under `crates/gpui-table/tests`.
- `crates/gpui-table/wip` is scratch data for macro stderr work, not a public surface.

## Feature Gates

- `chrono`, `rust_decimal`, and `spacetimedb` let macro expansion validate
  supported filter/type combinations at compile time.
- `fluent` enables typed `es-fluent` titles and faceted labels in generated code.
- `inventory` enables `GpuiTableShape` registration.
