# gpui-table-prototyping-core Architecture

## Purpose

`gpui-table-prototyping-core` generates gpui table scaffolding from the
`GpuiTableShape` inventory. It is intended for rapid prototyping and example
generation, and it depends only on schema metadata rather than the GPUI runtime
layer.

## Key modules

- `code_gen.rs`: adapts `GpuiTableShape` into a `TableShape` and orchestrates code generation.
  Key public API:
  - `TableShapeAdapter::parts() -> TableParts` — all pre-computed fragments + identifiers.
  - `TableShapeAdapter::try_parts() -> Result<TableParts, TableCodegenError>` — fallible version for user-facing tooling.
  - `TableShapeAdapter::generate_file(layout: &impl TableLayout) -> syn::File` — generate using a caller-supplied layout.
  - `TableShapeAdapter::try_generate_file(layout: &impl TableLayout) -> Result<syn::File, TableCodegenError>` — fallible version for user-facing tooling.
  - `TableLayout` trait — implement to control the entire generated file shape.
  - `TableParts` — all token-stream fragments exposed as named `pub` fields for use in custom layouts.
- `identities.rs`: `TableIdentities`, `TableIdentitiesExt`, `ShapeIdentities`
- `source_path.rs`: `source_path_to_use_path` — converts `file!()` paths to `use` import paths.
- `imports.rs`: `ImportItem`, `ImportSet` — per-item import tracking and grouped `use` statement rendering.
- `column.rs`: `ColumnCodeGenerator` trait, `ColumnInfo`, `ColumnIterator` — column-level utilities.

## Data flow

1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiTableShape>()`.
1. `TableShapeAdapter::new(shape, true).try_generate_file(&layout)` is the recommended entry point for user-facing tooling — it returns a ready-to-format `syn::File` or a structured `TableCodegenError`.
   Internally it:
   - Derives all identifiers from `GpuiTableShape` via `TableIdentities`.
   - Converts `shape.source_path` to a glob `use` path via `source_path_to_use_path`.
   - Calls `required_imports()` to build the minimal deduplicated import set.
   - Assembles all code fragments into `TableParts`.
   - For filter-enabled, non-load-more stories, emits
     `XxxFilterEntities::build_for_table(table.clone(), cx)` so client-side
     filtering stays interactive with `DataTable`. Generated stories call the
     inherent `filters.all_filters()` helper directly, so no filter-entity trait
     import is needed in the output.
   - For filter-enabled, load-more stories, emits
     `XxxFilterEntities::build_for_table_loader(table.clone(), window, cx)` so
     filter changes update delegate-owned filter state and trigger reloads.
   - Passes `TableParts` to the `TableLayout` implementation which produces the final `syn::File`.
1. The consumer formats with `prettyplease::unparse` and writes to disk.

The older `parts()` / `generate_file()` helpers remain as convenience wrappers
for trusted metadata, but they intentionally panic on malformed shapes; prefer
the `try_*` variants in generators and CLIs.

## Import design

Imports are declared at two levels:

- Framework items live in `code_gen::FRAMEWORK_IMPORTS` (always included).
- Filter items live in `code_gen::FILTER_IMPORTS` (conditionally included when filters are present).
- `ImportSet` deduplicates and groups items into compact `use parent::{a, b as c};` statements.

## Extension points

- Implement `TableLayout` to produce a completely custom file structure while reusing `TableParts` fragments.
- Implement `ColumnCodeGenerator` to customize column rendering in generated code.
- Override `TableShape` implementations to alter individual code generation fragments.
