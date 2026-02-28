# gpui-table-prototyping-core Architecture

## Purpose

`gpui-table-prototyping-core` generates gpui table scaffolding from the `GpuiTableShape` inventory. It is intended for rapid prototyping and example generation.

## Key modules

- `code_gen.rs`: adapts `GpuiTableShape` into a `TableShape` and orchestrates code generation.
  Key public API:
  - `TableShapeAdapter::parts() -> TableParts` — all pre-computed fragments + identifiers.
  - `TableShapeAdapter::generate_file(layout: &impl TableLayout) -> syn::File` — generate using a caller-supplied layout.
  - `TableLayout` trait — implement to control the entire generated file shape.
  - `TableParts` — all token-stream fragments exposed as named `pub` fields for use in custom layouts.
  - `TableIdentities` trait — identifier derivation helpers (story struct name, delegate name, etc.).
  - `source_path_to_use_path` — converts `file!()` paths to `use` import paths.
- `imports.rs`: `ImportItem`, `ImportSet` — per-item import tracking and grouped `use` statement rendering.
- `column.rs`: `ColumnCodeGenerator` trait, `ColumnInfo`, `ColumnIterator` — column-level utilities.

## Data flow

1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiTableShape>()`.
1. `TableShapeAdapter::new(shape, true).generate_file(&layout)` is the single entry point — it returns a ready-to-format `syn::File`.
   Internally it:
   - Derives all identifiers from `GpuiTableShape` via `TableIdentities`.
   - Converts `shape.source_path` to a glob `use` path via `source_path_to_use_path`.
   - Calls `required_imports()` to build the minimal deduplicated import set.
   - Assembles all code fragments into `TableParts`.
   - For filter-enabled, non-load-more stories, emits
     `XxxFilterEntities::build_for_table(table.clone(), cx)` so client-side
     filtering stays interactive with `DataTable`.
   - Passes `TableParts` to the `TableLayout` implementation which produces the final `syn::File`.
1. The consumer formats with `prettyplease::unparse` and writes to disk.

## Import design

Imports are declared at two levels:

- Framework items live in `code_gen::FRAMEWORK_IMPORTS` (always included).
- Filter items live in `code_gen::FILTER_IMPORTS` (conditionally included when filters are present).
- `ImportSet` deduplicates and groups items into compact `use parent::{a, b as c};` statements.

## Extension points

- Implement `TableLayout` to produce a completely custom file structure while reusing `TableParts` fragments.
- Implement `ColumnCodeGenerator` to customize column rendering in generated code.
- Override `TableShape` implementations to alter individual code generation fragments.
