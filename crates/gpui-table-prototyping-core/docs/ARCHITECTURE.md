# gpui-table-prototyping-core Architecture

## Purpose

`gpui-table-prototyping-core` turns `GpuiTableShape` registry metadata into
ready-to-format Rust syntax trees. It is intentionally a generator/helper layer,
not a runtime layer.

## Dependency Edges

- Depends on `gpui-table-schema`, not on `gpui-table-runtime`.
- Depends on `syn`, `quote`, and `proc-macro2` because its output is Rust code,
  not string templates.
- Is primarily consumed by `examples/prototyping`, but the API is designed for
  external generators and CLIs.

## Module Map

- `src/lib.rs`
  - Public exports and crate surface.
- `src/code_gen.rs`
  - `TableShapeAdapter`, `TableLayout`, `TableParts`, and `TableCodegenError`.
- `src/column.rs`
  - Column iteration and column-level generation helpers.
- `src/identities.rs`
  - Table/story identifier derivation and validation.
- `src/source_path.rs`
  - `file!()` path to `use` path normalization for generated imports.

## Internal Contracts

- `try_*` APIs are the safe external surface. They validate metadata and return
  `TableCodegenError` instead of panicking.
- Non-`try_*` helpers are convenience wrappers for trusted metadata and may panic.
- `TableParts` is the semantic boundary between metadata normalization and
  layout decisions. Layout implementations should consume it instead of
  re-deriving identifiers or imports.
- Import generation is intentionally centralized in
  `component_shape_codegen::imports::ImportSet` so generators do not drift into
  repeated or conflicting `use` statements.
- `source_path_to_use_path(...)` is part of the codegen contract because
  inventory registrations only preserve `file!()` paths, not ready-made import paths.

## Data Flow

1. A generator iterates `inventory::iter::<GpuiTableShape>()`.
1. `TableShapeAdapter::new(shape, use_fluent_titles)` normalizes registry data.
1. `try_parts()` or `try_generate_file()` validates identifiers, converts the
   registry source path into imports, and computes all reusable token fragments.
1. `TableLayout::generate_file(...)` decides the outer Rust file structure.
1. The caller formats the resulting `syn::File` and writes it to disk.

## Generated Output Expectations

- Filter-enabled, non-loader stories wire `XxxFilterEntities::build_for_table(...)`.
- Filter-enabled, loader-driven stories wire `XxxFilterEntities::build_for_table_loader(...)`.
- Generated stories call the inherent `all_filters()` helper on the filter
  entity set, which avoids forcing a trait import into every generated file.
- `examples/prototyping/output` is the canonical generated surface that
  validates these assumptions in the workspace.
