# gpui-table-runtime

`gpui-table-runtime` is the GPUI-facing runtime layer for the workspace.
It owns row traits, load-more traits, default cell rendering, and the stable
filter-entity contracts that derive-generated filter code targets.

This crate is for deeper integration work. Most application code should use
`gpui-table`.

## Use This Crate When

- you need generic code over generated row metadata or rendering traits
- you are implementing manual load-more or loader-driven flows
- you need generic code over generated filter entities

## Example

```rs
use gpui_table_runtime::{TableRowMeta, default_render_cell};

fn render_default_cell<R: TableRowMeta>(
    row: &R,
    column_index: usize,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> impl gpui::IntoElement + '_ {
    default_render_cell(row, column_index, window, cx)
}
```

## What It Provides

- `TableCell` and the built-in cell renderers for common scalar/date/time values
- `DisplayCell` and `FormattedCell` wrappers for generic value-object rendering
- `TableId` and the `TableRowMeta::table_id()` helper for passing stable table
  identifiers as typed values instead of bare strings.
- `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`, and
  `TableRowGeneratedContextMenu`, which are the row contracts targeted by
  derive-generated code
- `TableLoader` and `TableDataLoader`
- `shape`, the table filter shape contract used by generated filter entities,
  plus facade re-exports for `ComponentShapeMetadata`, `DeclaredComponentShape`,
  `ComponentShapeFor`, and `McpInput` when implementing custom filters.
  `GpuiTableFilterShapeBuilder` and `build_filter_shape` support configured
  field-level filter expressions such as
  `FacetedFilter::<Status>.searchable(true)` and
  `NumberRangeFilter.range(min, max).step(step)`.
- `generated_filters`, which provides `FilterEntitiesExt` plus
  `FilterSidebarData`, `FilterSidebarGroup`, and `FilterSidebarItem` for stable
  semantic grouping, localized labels, active-filter counts, and erased GPUI
  elements. Generated collections expose `read_values(cx)`,
  `apply_values(values, window, cx)`, `filter_sidebar_data(cx)`,
  `active_filter_count(cx)`, and `reset_filters(window, cx)`.

Filter shapes that support applying typed presets implement `unwrap_value` and
`set_silent`; the default `unwrap_value` rejects preset application. Generated
filter-value structs encode and decode saved presets through the facade's
`FilterPresetValue` contract.

Built-in filter component types such as `gpui_table_component::TextFilter` are
their own shape types. Their `shape::GpuiTableFilterShape` implementations live
in `gpui-table-component`, not in this runtime crate.

## Feature Flags

- `chrono` (default): date cell rendering and date-like filter field support
- `rust_decimal`: decimal cell rendering and numeric range filter field support
- `spacetimedb`: supported SpacetimeDB temporal range-filter support through the core layer

Date/time cells use the active `gpui-component` locale. Invalid locale strings,
out-of-range conversions, and unavailable ICU formatters are hard errors
instead of silently degrading to raw `Display` output.

If you only want the normal derive-based workflow, depend on `gpui-table`
instead of this crate directly.

For crate boundaries and internal runtime contracts, read the crate rustdocs and
the focused row, load, cell, shape, and generated-filter modules.
