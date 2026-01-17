# gpui-table

A powerful table component system for GPUI with derive macros, filtering, and pagination.

## Overview

`gpui-table` provides everything you need to build feature-rich tables in GPUI applications:

- Derive macros for zero-boilerplate table definitions
- Built-in filter components (text, number range, date range, faceted)
- Paginated data loading support
- Customizable styling
- Type-safe column and filter management

## Installation

```toml
[dependencies]
gpui-table = "0.5"
```

### With Filter Components

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component"] }
```

### All Features

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component", "fluent", "rust_decimal", "inventory"] }
```

## Quick Start

### Define a Table

```rust
use gpui_table::GpuiTable;

#[derive(Clone, GpuiTable)]
#[gpui_table(id = "products", title = "Products", delegate, filters)]
pub struct Product {
    #[gpui_table(title = "Name", width = 200.0)]
    #[gpui_table(filter(text))]
    pub name: String,
    
    #[gpui_table(title = "Price", width = 100.0, sortable)]
    #[gpui_table(filter(number_range(min = 0.0, max = 1000.0)))]
    pub price: f64,
    
    #[gpui_table(title = "Category", width = 120.0)]
    #[gpui_table(filter(faceted))]
    pub category: Category,
    
    #[gpui_table(title = "Created", width = 150.0)]
    pub created_at: DateTime<Utc>,
}
```

### Add Data Loading

```rust
#[gpui_table_impl]
impl Product {
    #[threshold]
    const LOAD_THRESHOLD: usize = 50;
    
    #[load_more]
    async fn load_products(offset: usize, limit: usize) -> Result<Vec<Product>> {
        // Fetch from API, database, etc.
        api::fetch_products(offset, limit).await
    }
}
```

### Use in Your Application

```rust
fn render_table(cx: &mut WindowContext) -> impl IntoElement {
    let delegate = ProductTableDelegate::new(cx);
    let filters = ProductFilterEntities::build(cx);
    
    v_flex()
        .child(filters.render(cx))
        .child(Table::new(delegate))
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `derive` | Yes | Include derive macros |
| `chrono` | Yes | DateTime type support |
| `component` | No | Filter UI components |
| `fluent` | No | Internationalization |
| `rust_decimal` | No | Decimal type support |
| `inventory` | No | Runtime metadata registry |

## Crate Structure

This is the main umbrella crate that re-exports from:

- [`gpui-table-core`](../gpui-table-core/) - Core traits and types
- [`gpui-table-derive`](../gpui-table-derive/) - Procedural macros
- [`gpui-table-component`](../gpui-table-component/) - Filter UI components

## Filter Types

### Text Filter
```rust
#[gpui_table(filter(text))]
#[gpui_table(filter(text(validation = "numeric")))]
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

### Faceted Filter (for enums)
```rust
#[derive(Filterable)]
pub enum Category {
    Electronics,
    Clothing,
    Books,
}

#[gpui_table(filter(faceted))]
pub category: Category,
```

## Filtering Data

```rust
// Build filters
let filters = ProductFilterEntities::build(cx);

// Read current values
let filter_values = filters.read_values(cx);

// Filter rows
let visible_rows: Vec<_> = all_rows
    .iter()
    .filter(|row| row.matches_filters(&filter_values))
    .collect();
```

## Custom Cell Rendering

```rust
#[derive(TableCell)]
pub struct Currency(f64);

impl TableCell for Currency {
    fn render_cell(&self, cx: &WindowContext) -> impl IntoElement {
        div().child(format!("${:.2}", self.0))
    }
}
```

## Documentation

- [Architecture Documentation](docs/ARCHITECTURE.md)
