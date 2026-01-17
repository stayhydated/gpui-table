# gpui-table-component

Filter UI components for the gpui-table system.

## Overview

This crate provides ready-to-use filter components for table filtering:

- **TextFilter** - Text search with debouncing and validation
- **NumberRangeFilter** - Numeric range with slider
- **DateRangeFilter** - Date range with calendar picker
- **FacetedFilter** - Multi-select checkboxes for enums
- **TableStatusBar** - Row count and loading status display

## Installation

```toml
[dependencies]
gpui-table-component = "0.5"
```

Or use the umbrella crate with the `component` feature:

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component"] }
```

## Components

### TextFilter

A debounced text input for string filtering.

```rust
use gpui_table::component::{TextFilter, TextFilterExt};

let filter = TextFilter::new(cx)
    .alphabetic_only()  // Only allow letters
    .placeholder("Search...");
```

**Configuration:**
- `.alphabetic_only()` - Only allow alphabetic characters
- `.numeric_only()` - Only allow numeric characters
- `.alphanumeric_only()` - Allow letters and numbers
- `.validate(fn)` - Custom validation function

### NumberRangeFilter

A dual-input range filter with optional slider.

```rust
use gpui_table::component::{NumberRangeFilter, NumberRangeFilterExt};

let filter = NumberRangeFilter::new(cx)
    .range(0.into(), 1000.into())
    .step(10.into());
```

**Configuration:**
- `.range(min, max)` - Set the valid range
- `.step(step)` - Set the slider step size

### DateRangeFilter

A date range picker with calendar UI.

```rust
use gpui_table::component::DateRangeFilter;

let filter = DateRangeFilter::new(cx);
```

### FacetedFilter

A multi-select filter for enum values.

```rust
use gpui_table::component::{FacetedFilter, FacetedFilterExt};

let filter = FacetedFilter::<Category>::new(cx)
    .searchable();  // Enable search within options
```

**Configuration:**
- `.searchable()` - Add a search box to filter options

### TableStatusBar

A status display component.

```rust
use gpui_table::component::TableStatusBar;

TableStatusBar::new()
    .row_count(150)
    .loading(false)
    .eof(true)
```

## Integration with Generated Code

When using `#[derive(GpuiTable)]` with `filters = true`, filter entities are generated:

```rust
#[derive(Clone, GpuiTable)]
#[gpui_table(id = "products", title = "Products", filters)]
pub struct Product {
    #[gpui_table(filter(text))]
    pub name: String,
    
    #[gpui_table(filter(number_range(min = 0.0, max = 1000.0)))]
    pub price: f64,
}

// Generated: ProductFilterEntities, ProductFilterValues
```

Build and use filters:

```rust
// Create filter entities
let filters = ProductFilterEntities::build(cx);

// Read current values
let values = filters.read_values(cx);

// Filter data
let filtered: Vec<_> = products
    .iter()
    .filter(|p| p.matches_filters(&values))
    .collect();
```

## Callback Pattern

All filters support `on_change` callbacks:

```rust
let filter = TextFilter::new(cx);
filter.update(cx, |f, cx| {
    f.on_change = Some(Box::new(move |value, cx| {
        // Handle filter change
    }));
});
```

## Documentation

- [Architecture Documentation](docs/ARCHITECTURE.md)
