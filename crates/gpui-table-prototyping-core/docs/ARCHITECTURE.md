# Architecture

## Purpose
`gpui-table-prototyping-core` provides codegen helpers that turn table registry
metadata into GPUI-ready table stories or prototypes.

## Module map
- `code_gen.rs`
  - `TableIdentities` and `TableShape` traits for generating identifiers and
    code fragments from a `GpuiTableShape`.
  - `TableShapeAdapter` that composes those fragments into a full story layout.
- `column.rs`
  - `ColumnCodeGenerator` trait + helpers for rendering or accessing columns.
  - Iterators and utilities for `ColumnVariant` slices.

## Data flow
1. `gpui-table-derive` (with the `inventory` feature) registers table shapes in
   the inventory.
2. Consumers iterate over `GpuiTableShape` entries and feed them into
   `TableShapeAdapter`.
3. Downstream tools format the generated token streams (e.g., with
   `prettyplease`) and write them out as story or prototype files
   (see `examples/prototyping`).

## Extension points
- Implement `ColumnCodeGenerator` to customize generated column rendering.
- Override `TableShape` implementations to alter story structure.

## Notes
- This crate is intended for tooling and prototyping, not runtime UI logic.
