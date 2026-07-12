# gpui-table-prototyping-core

`gpui-table-prototyping-core` generates GPUI table stories or scaffolding from
inventory-registered `GpuiTableShape` metadata.

Use this crate when you are building tooling or prototypes around the registry.
Most application code should not depend on it directly.

## Use This Crate When

- you want to turn `GpuiTableShape` registrations into story files or scaffolding
- you want reusable adapters and token fragments instead of hard-coding codegen
- you want generation to stay coupled to schema metadata instead of GPUI runtime internals

## Example

```rs
# fn demo() -> Result<(), gpui_table_prototyping_core::TableCodegenError> {
use gpui_table::registry::{GpuiTableShape, inventory};
use gpui_table_prototyping_core::{TableLayout, TableParts, TableShapeAdapter};
use quote::quote;

struct StoryLayout;

impl TableLayout for StoryLayout {
    fn generate_file(&self, parts: &TableParts) -> syn::File {
        let TableParts {
            imports,
            story_struct_ident,
            struct_fields,
            render_children,
            ..
        } = parts;

        syn::parse2(quote! {
            #imports

            pub struct #story_struct_ident {
                #struct_fields
            }

            // splice #render_children into your layout
        })
        .expect("static layout should parse")
    }
}

for shape in inventory::iter::<GpuiTableShape>() {
    let file = TableShapeAdapter::new(shape, true).try_generate_file(&StoryLayout)?;
    let _formatted = prettyplease::unparse(&file);
}
# Ok(())
# }
```

## Main Types

- `TableShapeAdapter`
  Adapts a `GpuiTableShape` into validated identities, imports, and token fragments.
- `TableLayout`
  Lets a generator control the outer file shape.
- `TableParts`
  Exposes precomputed token fragments for custom layouts. Fluent-backed
  `title_expr` fragments are intended to be emitted inside
  `Story::title(cx: &gpui::App)` so generated Storybook titles follow the
  active locale.
- `TableCodegenError`
  Structured error type for invalid metadata or identifier generation failures.

The recommended entry points for tools are the `try_*` methods, which return
`TableCodegenError` instead of panicking on malformed metadata.

## Reference Generator

See `examples/prototyping/src/main.rs` for a complete generator and
`examples/prototyping/output` for the produced story modules.

For internal generator stages and adapter contracts, read the crate rustdocs and
the modules under `src/`.
