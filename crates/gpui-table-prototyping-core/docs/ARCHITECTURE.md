# gpui-table-prototyping-core Architecture

This document describes the internal architecture of the `gpui-table-prototyping-core` crate.

## Overview

`gpui-table-prototyping-core` provides code generation utilities for prototyping and storybook integration. It enables tools to generate complete table UI examples from registry metadata collected at compile time.

## Module Structure

```
src/
├── lib.rs        # Public exports
├── column.rs     # Column utilities and code generation
└── code_gen.rs   # Table identity and shape code generation
```

## Core Concepts

### Registry-Based Generation

This crate reads `GpuiTableShape` metadata (collected via the `inventory` crate) and generates code for:
- Storybook stories
- Example applications
- Prototyping tools

### Separation of Concerns

1. **Column utilities** (`column.rs`) - Working with individual columns
2. **Table identity** (`code_gen.rs`) - Deriving names and identifiers
3. **Table shape** (`code_gen.rs`) - Generating code structures

## column.rs

### ColumnInfo

A wrapper around `ColumnVariant` providing utilities:

```rust
pub struct ColumnInfo<'a> {
    variant: &'a ColumnVariant,
}

impl<'a> ColumnInfo<'a> {
    pub fn field_ident(&self) -> Ident;
    pub fn pascal_case_name(&self) -> String;
    pub fn field_type_syn(&self) -> syn::Type;
    pub fn width(&self) -> f32;
    pub fn sortable(&self) -> bool;
    pub fn generate_value_accessor(&self) -> TokenStream;
    pub fn generate_display_child(&self) -> TokenStream;
}
```

### ColumnIterator

Iterator over columns with index tracking:

```rust
pub struct ColumnIterator<'a> {
    columns: &'a [ColumnVariant],
    index: usize,
}
```

### ColumnSliceExt

Extension trait for column slices:

```rust
pub trait ColumnSliceExt {
    fn column_iter(&self) -> ColumnIterator;
    fn sortable_columns(&self) -> Vec<&ColumnVariant>;
}
```

### ColumnCodeGenerator

Trait for extensible column code generation:

```rust
pub trait ColumnCodeGenerator {
    fn generate_value_accessor(&self, column: &ColumnInfo) -> TokenStream;
    fn generate_display_child(&self, column: &ColumnInfo) -> TokenStream;
}

pub struct DefaultColumnGenerator;
impl ColumnCodeGenerator for DefaultColumnGenerator { ... }
```

## code_gen.rs

### TableIdentities

Trait for deriving various names from table metadata:

```rust
pub trait TableIdentities {
    fn struct_name(&self) -> &str;
    fn story_struct_ident(&self) -> Ident;
    fn delegate_struct_ident(&self) -> Ident;
    fn snake_case_name(&self) -> String;
    fn ftl_label_ident(&self) -> Ident;
    fn has_filters(&self) -> bool;
}
```

### ShapeIdentities

Wrapper implementing `TableIdentities` for `GpuiTableShape`:

```rust
pub struct ShapeIdentities<'a> {
    shape: &'a GpuiTableShape,
}
```

### TableShape

Trait for abstract code generation:

```rust
pub trait TableShape {
    fn delegate_creation(&self) -> TokenStream;
    fn table_state_creation(&self) -> TokenStream;
    fn field_initializers(&self) -> TokenStream;
    fn struct_fields(&self) -> TokenStream;
    fn render_children(&self) -> TokenStream;
    fn title_expr(&self) -> TokenStream;
}
```

### TableShapeAdapter

Implements `TableShape` for `GpuiTableShape`:

```rust
pub struct TableShapeAdapter<'a> {
    shape: &'a GpuiTableShape,
    identities: ShapeIdentities<'a>,
}

impl<'a> TableShapeAdapter<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self;
    pub fn filter_entities_ident(&self) -> Ident;
    pub fn filter_values_ident(&self) -> Ident;
}
```

## Code Generation Flow

```
1. Iterate over inventory::iter::<GpuiTableShape>
   ↓
2. Wrap each shape in ShapeIdentities / TableShapeAdapter
   ↓
3. Generate story struct definition
   ↓
4. Generate field initializers for delegate, filters, state
   ↓
5. Generate render method with table UI
   ↓
6. Format output with prettyplease
```

## Example Output

For a `Product` table, the generated code might look like:

```rust
pub struct ProductStory {
    delegate: Entity<ProductTableDelegate>,
    filters: ProductFilterEntities,
    // ...
}

impl ProductStory {
    pub fn new(cx: &mut WindowContext) -> Self {
        Self {
            delegate: ProductTableDelegate::new(cx),
            filters: ProductFilterEntities::build(cx),
        }
    }
    
    pub fn render(&self, cx: &mut WindowContext) -> impl IntoElement {
        // Generated table UI
    }
}
```

## Dependencies

- `gpui-table-core` - Registry types
- `inventory` - Runtime metadata iteration
- `proc-macro2` - Token manipulation
- `quote` - Code generation
- `syn` - Type parsing
- `heck` - Case conversion
- `prettyplease` - Code formatting

## Use Cases

1. **Storybook Generation** - Auto-generate stories for all registered tables
2. **Documentation** - Generate example code for documentation
3. **Prototyping** - Quickly scaffold table UIs during development
4. **Testing** - Generate test fixtures from metadata
