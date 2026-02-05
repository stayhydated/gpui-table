# Project Overview

`gpui-table` is a table ecosystem written in **Rust**, built on top of `gpui`
and `gpui-component`. It focuses on:

1. **Type Safety**: Derive macros generate strongly-typed columns, filters, and metadata.
1. **Ergonomics**: `#[derive(GpuiTable)]` and `#[gpui_table_impl]` minimize boilerplate.
1. **Developer Experience**: Inventory-based shape registry (via the `inventory` feature)
   enables prototyping and codegen.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| --- | --- | --- |
| **Core** | | |
| `gpui-table` | [Architecture](crates/gpui-table/docs/ARCHITECTURE.md) | Facade crate; re-exports macros/core/components. |
| `gpui-table-core` | [Architecture](crates/gpui-table-core/docs/ARCHITECTURE.md) | Core traits, filter types, and registry metadata. |
| `gpui-table-derive` | [Architecture](crates/gpui-table-derive/docs/ARCHITECTURE.md) | Proc macros for table derivation and load-more wiring. |
| **Components & Runtime** | | |
| `gpui-table-component` | [Architecture](crates/gpui-table-component/docs/ARCHITECTURE.md) | GPUI filter components and status bar. |
| **Prototyping** | | |
| `gpui-table-prototyping-core` | [Architecture](crates/gpui-table-prototyping-core/docs/ARCHITECTURE.md) | Codegen from inventory shapes for prototyping. |

## Crate Descriptions

### Core Layers

- **`gpui-table`**: User-facing facade. Re-exports derive macros, core metadata, and
  optional UI components.
- **`gpui-table-core`**: Shared metadata and traits for table rows, filters, and registry.
- **`gpui-table-derive`**: Proc macros that expand row structs into columns, delegates,
  filters, and optional inventory registrations.

### Components & Runtime

- **`gpui-table-component`**: GPUI UI components for text, faceted, number-range, and
  date-range filters, plus `TableStatusBar`.

### Prototyping

- **`gpui-table-prototyping-core`**: Builds GPUI table scaffolding by consuming
  `GpuiTableShape` inventory data.

## Examples

### Derive a table with filters and load-more

```rs
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::{GpuiTable, TableLoader};

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, load_more)]
pub struct User {
    #[gpui_table(sortable, width = 160., filter(text()))]
    pub name: String,

    #[gpui_table(width = 80., filter(number_range(min = 0, max = 120)))]
    pub age: u8,

    #[gpui_table(width = 90., filter(faceted()))]
    pub active: bool,
}

#[gpui_table::gpui_table_impl]
impl TableLoader for UserTableDelegate {
    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        // fetch + append rows
        cx.notify();
    }
}
```

### Custom TableCell rendering

```rs
use gpui_table_core::TableCell;
use gpui::{AnyElement, App, Window};

pub struct Rating(pub u8);

impl TableCell for Rating {
    fn draw(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        format!("{}*", self.0).into()
    }
}
```

### Prototyping from the inventory registry

```rs
use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::{TableShape as _, TableShapeAdapter};

for shape in inventory::iter::<GpuiTableShape>() {
    let adapter = TableShapeAdapter::new(shape, true);
    let _delegate_tokens = adapter.delegate_creation();
    let _render_tokens = adapter.render_children();
}
```

## Repo layout

- `crates/gpui-table`: facade crate that re-exports core traits, derive macros, and optional components
- `crates/gpui-table-core`: traits, filter types, registry metadata
- `crates/gpui-table-derive`: proc-macros (`GpuiTable`, `TableCell`, `gpui_table_impl`)
- `crates/gpui-table-component`: GPUI filter components and status bar
- `crates/gpui-table-prototyping-core`: codegen helpers for prototyping
- `examples/`: sample apps and storybook output

## Agent Notes

- Ignore all folders matching `**/__crate_paths/**` (generated files).
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
- Prefer workspace dependencies and feature flags from the root `Cargo.toml`.
