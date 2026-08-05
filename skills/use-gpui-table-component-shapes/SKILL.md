---
name: use-gpui-table-component-shapes
description: >-
  Select, adapt, implement, or debug gpui-table filter component shapes. Use
  whenever code involves #[gpui_table(filter(...))] compatibility,
  TextFilterAdapter, NumberRangeFilterAdapter, DateRangeFilterAdapter,
  GpuiTableFilterShape, GpuiTableFilterShapeFor, configured shape builders,
  saved-preset support, custom MCP filter schemas or decoding, Koruma newtype
  filters, or component-shape metadata consumed by table prototyping.
---

# Use gpui-table component shapes

## Workflow

1. Identify the table field type, desired widget, component-facing raw value,
   typed matching value, and whether UI, presets, MCP, or inventory metadata are
   required.
2. Choose the least custom option that satisfies those requirements.
3. Confirm the facade and component feature flags before writing trait
   implementations.
4. Declare the shape explicitly in `#[gpui_table(filter(...))]` and
   keep `#[gpui_table(filters)]` for UI filters or
   `#[gpui_table(mcp)]` for MCP-only filters.
5. Add preset and MCP contracts only when those flows are used.
6. Follow the application's existing component, validation, and metadata
   conventions.

## Choose a shape strategy

| Requirement | Strategy |
|---|---|
| Supported field and standard widget | Use the built-in shape |
| Domain wrapper with standard raw value | Use a built-in adapter and field trait |
| Existing widget with converted raw value | Derive `GpuiTableFilterShape` |
| New widget or matching semantics | Implement the runtime shape contracts |

Prefer `TextFilter`, `FacetedFilter<T>`,
`NumberRangeFilter`, or `DateRangeFilter` for supported
fields. Prefer `TextFilterAdapter`,
`NumberRangeFilterAdapter`, or `DateRangeFilterAdapter` for
application value objects.

Use `use-gpui-table` for ordinary table composition. Use
`use-component-shape` when the task changes framework-neutral
component metadata rather than table-specific entity lifecycle and matching.

## Preserve the contracts

A table filter shape must implement
`DeclaredGpuiTableFilterShape` and
`GpuiTableFilterShapeFor<Field>`. Manual shapes also need the
framework-neutral declared metadata and field marker expected by the registry.

Keep inactive values matching all rows. For saved presets, use a
`FilterValue` implementing `FilterPresetValue` and
implement `unwrap_value` plus `set_silent`. For MCP,
provide `McpFilterShape`; derive it when `RawValue:
McpToolValue` and implement it manually only for a custom schema or
decoder.

Use `koruma_newtype` on a derived adapter when filtering a Koruma
newtype through its inner value. Manual equivalents also need
`McpKorumaNewtypeFilterValidation<Field>` when MCP validation is
required.

## Load detailed patterns

Read [references/custom-shapes.md](references/custom-shapes.md) for adapter,
derive, full runtime, preset, MCP, and metadata patterns. Do not load it for a
table that only selects a supported built-in shape.
