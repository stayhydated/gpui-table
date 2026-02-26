# gpui-table-prototyping-core

Utilities for generating gpui table scaffolding from `GpuiTableShape` inventory data.

This crate is useful when you want to rapidly prototype tables from your struct
definitions without hand-writing the gpui widget wiring.

## Usage

Enable the `inventory` feature on `gpui-table` and iterate the registered shapes:

```rs
use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::{TableLayout, TableParts, TableShapeAdapter};
use quote::quote;

struct MyLayout;

impl TableLayout for MyLayout {
    fn generate_file(&self, parts: &TableParts) -> syn::File {
        let TableParts { imports, story_struct_ident, struct_fields, render_children, .. } = parts;
        syn::parse2(quote! {
            #imports
            pub struct #story_struct_ident { #struct_fields }
            // splice #render_children wherever you need it
        }).unwrap()
    }
}

for shape in inventory::iter::<GpuiTableShape>() {
    let syn_file = TableShapeAdapter::new(shape, true).generate_file(&MyLayout);
    let _formatted = prettyplease::unparse(&syn_file);
}
```

See `examples/prototyping` for a full generator that writes formatted files.
