# gpui-table-prototyping-core

Core library for prototyping table views with gpui-table.

This crate provides utilities for generating table view scaffolds from structs
annotated with `#[derive(NamedTableRow)]`. It uses inventory to collect table
shape metadata at runtime and generates complete Rust source files for table
story components.

## Usage

This crate is intended to be used by a code generation binary (like the
`prototyping` example) rather than directly in application code.

```rust
use gpui_table_core::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::TableShapeAdapter;

// Import your library to trigger inventory registrations
use your_lib::*;

for shape in inventory::iter::<GpuiTableShape>() {
    let adapter = TableShapeAdapter::new(shape);
    let syn_file = generate_table_view(&adapter);
    // Write formatted code to file...
}
```

## Features

- Generates complete table story components
- Integrates with gpui-storybook for story registration
- Supports fluent localization for table titles
- Generates proper imports and struct definitions
