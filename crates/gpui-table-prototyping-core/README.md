# gpui-table-prototyping-core

Code generation utilities for gpui-table prototyping and storybook integration.

## Overview

This crate provides utilities for generating table UI code from registry metadata. It's primarily used by:

- Storybook generators
- Prototyping tools
- Documentation generators
- Testing infrastructure

## Installation

```toml
[dependencies]
gpui-table-prototyping-core = "0.5"
```

## Features

### Registry Iteration

Iterate over all registered table shapes:

```rust
use gpui_table_core::registry::GpuiTableShape;

for shape in inventory::iter::<GpuiTableShape> {
    println!("Table: {} with {} columns", 
        shape.struct_name, 
        shape.columns.len()
    );
}
```

### Column Utilities

Work with column metadata:

```rust
use gpui_table_prototyping_core::column::{ColumnSliceExt, ColumnInfo};

for col in shape.columns.column_iter() {
    let info = ColumnInfo::new(col);
    println!("Field: {} ({})", info.field_ident(), info.field_type_syn());
}
```

### Identity Generation

Generate consistent names from table metadata:

```rust
use gpui_table_prototyping_core::code_gen::{ShapeIdentities, TableIdentities};

let identities = ShapeIdentities::new(shape);
println!("Story struct: {}", identities.story_struct_ident());
println!("Delegate: {}", identities.delegate_struct_ident());
```

### Code Generation

Generate complete table UI code:

```rust
use gpui_table_prototyping_core::code_gen::{TableShapeAdapter, TableShape};

let adapter = TableShapeAdapter::new(shape);
let delegate_code = adapter.delegate_creation();
let fields_code = adapter.struct_fields();
let render_code = adapter.render_children();
```

## Key Types

### ColumnInfo

Wrapper providing utilities for column metadata:

```rust
impl<'a> ColumnInfo<'a> {
    fn field_ident(&self) -> Ident;
    fn pascal_case_name(&self) -> String;
    fn field_type_syn(&self) -> syn::Type;
    fn width(&self) -> f32;
    fn sortable(&self) -> bool;
    fn generate_value_accessor(&self) -> TokenStream;
    fn generate_display_child(&self) -> TokenStream;
}
```

### ShapeIdentities

Derives various identifiers from table metadata:

```rust
impl TableIdentities for ShapeIdentities<'_> {
    fn struct_name(&self) -> &str;
    fn story_struct_ident(&self) -> Ident;
    fn delegate_struct_ident(&self) -> Ident;
    fn snake_case_name(&self) -> String;
    fn has_filters(&self) -> bool;
}
```

### TableShapeAdapter

Generates code structures from table shape:

```rust
impl TableShape for TableShapeAdapter<'_> {
    fn delegate_creation(&self) -> TokenStream;
    fn table_state_creation(&self) -> TokenStream;
    fn field_initializers(&self) -> TokenStream;
    fn struct_fields(&self) -> TokenStream;
    fn render_children(&self) -> TokenStream;
    fn title_expr(&self) -> TokenStream;
}
```

## Example: Generating a Storybook

```rust
use gpui_table_prototyping_core::code_gen::{TableShapeAdapter, ShapeIdentities, TableIdentities, TableShape};
use gpui_table_core::registry::GpuiTableShape;
use quote::quote;

fn generate_stories() -> String {
    let mut stories = Vec::new();
    
    for shape in inventory::iter::<GpuiTableShape> {
        let adapter = TableShapeAdapter::new(shape);
        let identities = ShapeIdentities::new(shape);
        
        let story_name = identities.story_struct_ident();
        let fields = adapter.struct_fields();
        let init = adapter.field_initializers();
        let render = adapter.render_children();
        
        stories.push(quote! {
            pub struct #story_name {
                #fields
            }
            
            impl #story_name {
                pub fn new(cx: &mut WindowContext) -> Self {
                    Self { #init }
                }
                
                pub fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
                    #render
                }
            }
        });
    }
    
    let output = quote! { #(#stories)* };
    prettyplease::unparse(&syn::parse2(output).unwrap())
}
```

## Documentation

- [Architecture Documentation](docs/ARCHITECTURE.md)
