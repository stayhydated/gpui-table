# gpui-table-schema

`gpui-table-schema` contains the UI-neutral metadata shared across the
`gpui-table` workspace: filter configuration, faceted option metadata, and the
inventory-backed table-shape registry.

This crate is for tooling and integration work. Most application code should
use `gpui-table`.

## Use This Crate When

- you are building tooling around table metadata
- you want to inspect or transform `GpuiTableShape` inventory registrations
- you want schema types without depending on GPUI runtime crates

## Example

```rs
use gpui_table_schema::registry::{GpuiTableShape, inventory};

for shape in inventory::iter::<GpuiTableShape>() {
    println!(
        "{} -> {} columns, {} filters",
        shape.struct_name,
        shape.columns.len(),
        shape.filters.len()
    );
}
```

In normal application code, those registrations are produced by
`#[derive(GpuiTable)]` with the `inventory` feature enabled on `gpui-table`.

## What It Provides

- `FilterConfig`, `FilterType`, `FacetedFilterOption`, and `FacetedFilterIcon`
- `GpuiTableShape`, `ColumnVariant`, `FilterVariant`, `RegistryFilterType`, and `ColumnFixed`
- `ComponentShapeUse`, `RustPath`, and `RustType` for field-to-filter-shape and Rust syntax metadata captured from macro-generated schema registrations
- the `inventory` re-export for collecting and iterating registered shapes

This crate intentionally does not depend on `gpui` or `gpui-component`.

If you need derives, loaders, rendering traits, or built-in filter UI, use
`gpui-table` instead.

For internal metadata contracts and dependency boundaries, read the crate
rustdocs and the registry module.
