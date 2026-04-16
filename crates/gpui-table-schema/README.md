# gpui-table-schema

Schema and registry metadata for the `gpui-table` ecosystem.

## What it provides

- Filter metadata types: `FilterConfig`, `FilterType`, `FacetedFilterOption`
- UI-neutral faceted icon metadata via `FacetedFilterIcon`
- Inventory-backed table shape metadata: `GpuiTableShape`, `ColumnVariant`,
  `FilterVariant`, `RegistryFilterType`, `ColumnFixed`

## Notes

- This crate intentionally does not depend on `gpui` or `gpui-component`.
- Runtime traits and rendering helpers live in `gpui-table-runtime`.
- Filter matching and typed filter values live in `gpui-table-core`.
