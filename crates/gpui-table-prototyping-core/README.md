# gpui-table-prototyping-core

Utilities for generating GPUI table stories or prototypes from registry data.

## Typical usage

This crate is designed to consume `GpuiTableShape` entries registered via
`inventory` and emit code (often for storybook or prototyping workflows).

```rs
use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::{TableShape as _, TableShapeAdapter};

for shape in inventory::iter::<GpuiTableShape>() {
    let adapter = TableShapeAdapter::new(shape, true);
    let _delegate_tokens = adapter.delegate_creation();
    let _render_tokens = adapter.render_children();
}
```

## Notes

- Intended for tooling and prototyping, not runtime UI logic.
- See `examples/prototyping` for a full generator.
