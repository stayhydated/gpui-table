# gpui-table-runtime

`gpui-table-runtime` is the GPUI-facing runtime layer for the workspace.
It owns row traits, load-more traits, default cell rendering, and the stable
runtime facade that derive-generated filter code targets.

This crate is for deeper integration work. Most application code should use
`gpui-table`.

## Use This Crate When

- you are customizing row rendering with `TableRowStyle`
- you are implementing manual load-more or loader-driven flows
- you need generic code over generated filter entities and built-in components

## Example

```rs
use gpui::{AnyElement, App, IntoElement, Window, div};
use gpui_table_runtime::{TableRowStyle, default_render_cell};

impl TableRowStyle for Item {
    type ColumnId = ItemTableColumn;

    fn render_table_cell(
        &self,
        column: Self::ColumnId,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        match column {
            ItemTableColumn::Weight => div()
                .child(format!("{} kg", self.weight))
                .into_any_element(),
            _ => default_render_cell(self, column.into(), window, cx).into_any_element(),
        }
    }
}
```

## What It Provides

- `TableCell` and the built-in cell renderers for common scalar/date/time values
- `DisplayCell` and `FormattedCell` wrappers for generic value-object rendering
- `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`, and `TableRowGeneratedContextMenu`
- `TableLoader` and `TableDataLoader`
- `shape`, the table filter shape contract used by generated filter entities,
  plus facade re-exports for `ComponentShapeMetadata`, `DeclaredComponentShape`,
  `ComponentShapeFor`, `McpInput`, and built-in filter shape types when
  implementing custom filters
- `generated_filters`, which re-exports the built-in filter UI, localization helpers, `FilterEntitiesExt`, `TableFilterComponent`, and `QueryFilterValue`

The `generated_filters` module is the stable runtime target for code emitted by
`#[derive(GpuiTable)]`. Use it when you want manual and generated filter flows
to share the same runtime surface.
Built-in filter component types such as `gpui_table_component::TextFilter` are
their own shape types; generated entities construct them through
`shape::GpuiTableFilterShape`.

## Feature Flags

- `chrono` (default): date cell/filter runtime support and date-range filter UI wiring
- `rust_decimal`: numeric range-filter runtime support
- `spacetimedb`: supported SpacetimeDB temporal range-filter support through the core layer

If you only want the normal derive-based workflow, depend on `gpui-table`
instead of this crate directly.

For crate boundaries and internal runtime contracts, see `docs/ARCHITECTURE.md`.
