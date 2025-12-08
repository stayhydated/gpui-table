# gpui-table-prototyping-core

opinionated core for prototyping table stories. inventory collects
`GpuiTableShape` metadata so we can generate table scaffolds without digging
into each row type by hand.

This crate exposes small adapters that mirror the gpui-form prototyping core:
identify shapes, generate delegate/table wiring, and optionally render column
debug helpers.

```rust
use gpui_table_core::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::{TableShape, TableShapeAdapter};

// import your library so inventory registrations run
use your_lib::*;

for shape in inventory::iter::<GpuiTableShape>() {
    let adapter = TableShapeAdapter::new(shape);

    let imports = adapter.additional_imports().unwrap_or_default();
    let table_setup = adapter.table_state_creation();
    let columns_debug = adapter.column_debug_children().unwrap_or_default();
    let story_children = adapter.render_children();

    // assemble a syn::File with the tokens above...
}
```

Best consumed from a small codegen binary (see `examples/prototyping`) rather
than directly in application code.
