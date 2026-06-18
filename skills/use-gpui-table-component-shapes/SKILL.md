---
name: use-gpui-table-component-shapes
description: "Use when Codex needs to wire component-shape filter shapes into gpui-table tables, including #[gpui_table(filter(...))], GpuiTableFilterShape, field support, generated filter entities/values, MCP filter decoding, Koruma newtype filters, or prototyping metadata."
---

# Use GPUI Table Component Shapes

## Scope Boundary

Use this skill for `gpui-table` integration work after a GPUI table filter
component shape exists or needs to be selected. It covers how filter shapes
participate in `#[derive(gpui_table::GpuiTable)]`, generated filter entities,
typed filter values, field matching, MCP query arguments, and inventory-backed
prototyping metadata.

Use `use-gpui-table` for ordinary application tables that only use the built-in
text, faceted, number range, or date range filters.

Use the component-shape repository skills for shape ownership and declaration:

- `use-component-shape` for framework-neutral shape metadata, suffixes,
  capabilities, and MCP input metadata.
- `use-component-shape-gpui` for `component_shape_gpui::GpuiComponentShape`,
  `component_shape_gpui::component_shape!`, GPUI render contracts, and
  value-change declarations.

This reusable skill does not cover proc-macro implementation internals,
repository maintenance, or contributor-only architecture work. Use the
workspace `AGENTS.md`, crate rustdocs, focused tests, and compile-fail fixtures
for those tasks.

## Integration Rule

First decide whether the table can use an existing filter shape:

- Existing built-in shape: prefer `gpui_table_component::TextFilter`,
  `FacetedFilter::<T>`, `NumberRangeFilter`, or `DateRangeFilter` when they
  already model the widget, raw value, field matching, and MCP input.
- Adapter shape: derive `gpui_table::GpuiTableFilterShape` when an existing
  base shape should keep its UI, reset behavior, and field matching but expose a
  custom raw value to table state or MCP decoding.
- New custom shape: create or select the shape with component-shape GPUI
  guidance first, then implement the table runtime shape contracts here.

Application crates that only derive tables normally depend on `gpui-table` and
`gpui-table-component`. Crates that define reusable custom filter shapes may
also depend on `gpui-table-runtime` directly when they implement lower-level
runtime traits.

## Field Shape Usage

Use the shape type in the field attribute:

```rust
#[derive(Clone, gpui_table::GpuiTable)]
#[gpui_table(filters)]
pub struct User {
    #[gpui_table(filter(gpui_table_component::TextFilter))]
    pub name: String,
}
```

The shape must implement `DeclaredGpuiTableFilterShape` and
`GpuiTableFilterShapeFor<Field>`. Compatibility determines whether the shape
can support the field type; `matches_field` determines client-side filtering
behavior and generated `Matchable<<Row>FilterValues>` semantics.

Built-in field support is intentionally narrow:

- `TextFilter`: `String` and `Option<String>`.
- `FacetedFilter<T>`: `T`, `Option<T>`, `Vec<T>`, and `Option<Vec<T>>` where
  `T: Filterable`.
- `NumberRangeFilter`: supported numeric types and `Option<T>` variants when
  the relevant feature flags are enabled.
- `DateRangeFilter`: supported date-like types and `Option<T>` variants when
  the relevant feature flags are enabled.

Use `#[gpui_table(filters)]` for UI-generated filter entities, or
`#[gpui_table(mcp)]` when the filters are MCP-only query arguments.

## Adapter Shapes

Prefer the derive when adapting an existing base shape:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrefixText(String);

impl gpui_table::mcp::McpToolValue for PrefixText {
    fn tool_value_schema() -> gpui_table::mcp::McpSchema {
        gpui_table::mcp::McpSchema::string()
    }

    fn from_tool_value(
        field: &str,
        value: gpui_table::mcp::serde_json::Value,
    ) -> Result<Self, gpui_table::mcp::McpToolError> {
        value
            .as_str()
            .map(|value| Self(value.to_string()))
            .ok_or_else(|| gpui_table::mcp::McpToolError::decode(field, "expected string"))
    }
}

#[derive(gpui_table::GpuiTableFilterShape)]
#[gpui_table_filter_shape(
    base = gpui_table_component::TextFilter,
    raw_value = PrefixText,
    field = String,
    into_base = |value: PrefixText| value.0,
    from_base = PrefixText
)]
pub struct PrefixTextFilter;
```

The derive emits `ComponentShapeMetadata`, declared-shape marker impls,
`GpuiTableFilterShape`, and `GpuiTableFilterShapeFor<Field>`. With the
`gpui-table/mcp` feature enabled, it also emits the default
`gpui_table::mcp::McpFilterShape` and validation bridge when the raw value
implements `McpToolValue`.

Use `fields(A, B, ...)` when the adapter supports multiple field types. Use
`koruma_newtype` when the source table field is a Koruma newtype but the filter
should operate on the validated inner value.

## Runtime Shape Contract

For fully custom filters, implement:

```rust
impl gpui_table::runtime::shape::GpuiTableFilterShape for MyTextFilterShape {
    type Component = MyTextFilterComponent;
    type RawValue = String;
    type FilterValue = gpui_table::core::filter::TextValue;

    const FILTER_TYPE: gpui_table::schema::registry::RegistryFilterType =
        gpui_table::schema::registry::RegistryFilterType::Text;

    // new_for, read_value, wrap_value, and reset_silent connect generated
    // filter entities to the GPUI component.
}

impl gpui_table::runtime::shape::DeclaredGpuiTableFilterShape for MyTextFilterShape {}

impl gpui_table::runtime::shape::GpuiTableFilterShapeFor<String> for MyTextFilterShape {
    fn filter_type() -> gpui_table::core::filter::FilterType {
        gpui_table::core::filter::FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        // Return true for inactive filters.
        value.matches(field.as_ref())
    }
}
```

`RawValue` is the component-facing state that generated filter entities read,
store, reset, serialize with `QueryFilterValue`, and decode from MCP when
enabled. `FilterValue` is the typed wrapper used for matching table fields.
Keep inactive filter values matching all rows.

Choose the registry and filter metadata category that matches the user-facing
filter semantics: `Text`, `Faceted`, `NumberRange`, or `DateRange`.

## MCP Filter Arguments

For MCP-exposed tables, every declared filter shape must be decodable:

- Built-in shapes already implement `gpui_table::mcp::McpFilterShape`.
- Adapter shapes get the default MCP implementation when `RawValue:
  McpToolValue`.
- Fully custom shapes can derive `gpui_table::McpFilterShape` when
  `RawValue: McpToolValue`, or implement `gpui_table::mcp::McpFilterShape`
  manually for custom schemas or decoding.

Use `gpui_table::mcp::McpAny` only when the typed raw value intentionally
accepts unconstrained JSON. Use `gpui_table::mcp::McpRange<T>` for custom
`{ "min": ..., "max": ... }` range arguments.

Manual shapes that should support field-level `#[koruma(...)]` validation must
also implement `gpui_table::mcp::McpFilterShapeValidation`. Manual shapes that
support Koruma newtype inner-value filters must also implement
`gpui_table::mcp::McpKorumaNewtypeFilterValidation<Field>`.

## Prototyping Alignment

When a filter shape is used by inventory-driven prototyping, ensure it exposes
stable component metadata. Shape paths are recorded in registry metadata through
`ComponentShapeUse`, and generated table stories use that metadata to produce
stable helper names and filter declarations.

If `GpuiTableShape`, `ColumnVariant`, `FilterVariant`, filter metadata, or
component shape metadata changes, update the relevant README files and
rustdocs, then regenerate `examples/prototyping/output` with
`cargo run -p prototyping`.

## Documentation Sync

When changing user-visible table/component-shape behavior in this repository,
keep the relevant public surfaces aligned:

- root `README.md`
- `crates/gpui-table/README.md`
- affected crate READMEs
- `examples/README.md` and showcased examples when behavior changes
- in-repository `skills/*` guidance
- matching rustdocs, source-adjacent comments, tests, and examples when
  boundaries or behavior change

Do not duplicate generic component-shape macro guidance here; update the
component-shape-owned skills instead.
