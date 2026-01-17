# gpui-table-derive

Procedural macros for the gpui-table system.

## Overview

This crate provides derive macros that generate table infrastructure from annotated structs:

- `#[derive(GpuiTable)]` - Generate table metadata, delegates, and filters
- `#[gpui_table_impl]` - Bridge user-defined loading logic to delegates
- `#[derive(Filterable)]` - Make enums usable as faceted filter options
- `#[derive(TableCell)]` - Enable custom types for cell rendering

## Installation

```toml
[dependencies]
gpui-table-derive = "0.5"
```

Or use the umbrella crate:

```toml
[dependencies]
gpui-table = "0.5"
```

## Usage

### Basic Table

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(id = "products", title = "Products")]
pub struct Product {
    #[gpui_table(title = "Name", width = 200.0)]
    pub name: String,
    
    #[gpui_table(title = "Price", width = 100.0, sortable)]
    pub price: f64,
    
    #[gpui_table(title = "In Stock", width = 80.0)]
    pub in_stock: bool,
}
```

### With Delegate and Filters

```rust
#[derive(Clone, GpuiTable)]
#[gpui_table(id = "products", title = "Products", delegate, filters)]
pub struct Product {
    #[gpui_table(title = "Name", width = 200.0)]
    #[gpui_table(filter(text))]
    pub name: String,
    
    #[gpui_table(title = "Price", width = 100.0)]
    #[gpui_table(filter(number_range(min = 0.0, max = 1000.0)))]
    pub price: f64,
    
    #[gpui_table(title = "Category", width = 120.0)]
    #[gpui_table(filter(faceted))]
    pub category: Category,
}
```

### Data Loading

```rust
#[gpui_table_impl]
impl Product {
    #[threshold]
    const LOAD_THRESHOLD: usize = 50;
    
    #[load_more]
    async fn load_products(offset: usize, limit: usize) -> Result<Vec<Product>> {
        // Load data from API, database, etc.
    }
}
```

## Struct Attributes

| Attribute | Description |
|-----------|-------------|
| `id` | Unique table identifier |
| `title` | Display title |
| `delegate` | Generate TableDelegate struct |
| `custom_style` | Skip default TableRowStyle generation |
| `fluent` | Use fluent for i18n |
| `loading` | Include loading state in delegate |
| `filters` | Generate filter infrastructure |

## Field Attributes

| Attribute | Description |
|-----------|-------------|
| `title` | Column header title |
| `width` | Column width in pixels |
| `sortable` | Enable sorting |
| `fixed = "left"/"right"` | Fix column position |
| `filter(type)` | Add filter (text, number_range, date_range, faceted) |

## Filter Types

### Text Filter
```rust
#[gpui_table(filter(text))]
#[gpui_table(filter(text(validation = "alphabetic")))]
```

### Number Range Filter
```rust
#[gpui_table(filter(number_range))]
#[gpui_table(filter(number_range(min = 0.0, max = 100.0, step = 1.0)))]
```

### Date Range Filter
```rust
#[gpui_table(filter(date_range))]
```

### Faceted Filter
```rust
#[gpui_table(filter(faceted))]
#[gpui_table(filter(faceted(searchable)))]
```

## Generated Types

For a struct `Product`, the macro generates:

- `ProductTableColumn` - Column identifier enum
- `ProductTableDelegate` - Table delegate (when `delegate` enabled)
- `ProductFilterEntities` - Filter UI entities (when `filters` enabled)
- `ProductFilterValues` - Filter value struct (when `filters` enabled)

## Documentation

- [Architecture Documentation](docs/ARCHITECTURE.md)
