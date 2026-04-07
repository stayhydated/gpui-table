# gpui-table-core

Core traits and types for building tables with `gpui-component`. This crate is
shared by the derive macros, runtime components, and tooling.

## What it provides

- `TableRowMeta`, `TableRowStyle`, `TableRowContextMenu`, `TableRowGeneratedContextMenu`, and `TableCell` traits
- Load-more traits (`TableLoader`, `TableDataLoader`)
- Filter types and helper wrappers (`TextValue`, `RangeValue`, `FacetedValue`)
- Filter matching traits (`Matchable`, `FilterValuesExt`, `FilterEntitiesExt`)
- Registry metadata types for tooling (`GpuiTableShape`)

## Feature flags

- `chrono`: `TableCell` + date filter helpers
- `rust_decimal`: numeric range helpers
- `fluent`: localized title helpers used by generated code

## Example: custom TableCell

```rs
use gpui_table_core::TableCell;
use gpui::{AnyElement, App, Window};

pub struct Rating(pub u8);

impl TableCell for Rating {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        format!("{}★", self.0).into()
    }
}
```

## Example: custom row styling

```rs
use gpui_table_core::{TableRowMeta, TableRowStyle};
use gpui::{AnyElement, App, Window, div, IntoElement};

impl TableRowStyle for MyRow {
    type ColumnId = MyRowTableColumn;

    fn render_table_cell(
        &self,
        col: Self::ColumnId,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        // custom rendering per column
        self.cell_value(col.into()).draw(window, cx)
    }
}
```

## Example: custom row context menu

```rs
use gpui::{App, Window};
use gpui_component::menu::PopupMenu;
use gpui_table_core::{TableRowContextMenu, TableRowGeneratedContextMenu};

impl TableRowContextMenu for MyRow {
    fn render_table_context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> PopupMenu {
        self.render_generated_table_context_menu(row_ix, menu, window, cx)
            .link("Share", "https://example.com")
    }
}
```
