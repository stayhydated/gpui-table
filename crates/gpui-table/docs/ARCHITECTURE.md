# gpui-table Architecture

This document describes the internal architecture of the `gpui-table` crate.

## Overview

`gpui-table` is the main umbrella crate that re-exports functionality from the other crates in the workspace. It provides a unified public API for users who want all table functionality through a single dependency.

## Module Structure

```
src/
└── lib.rs    # Re-exports and feature-gated modules
```

## Re-export Strategy

The crate conditionally re-exports based on enabled features:

```rust
// Always available
pub use gpui_table_core::{
    filter,
    registry,
    TableCell,
    TableRowMeta,
    TableRowStyle,
    TableDataLoader,
    TableLoader,
    // ...
};

// When "derive" feature enabled
#[cfg(feature = "derive")]
pub use gpui_table_derive::{
    GpuiTable,
    gpui_table_impl,
    Filterable,
    TableCell,
};

// When "component" feature enabled
#[cfg(feature = "component")]
pub mod component {
    pub use gpui_table_component::{
        TextFilter, TextFilterExt,
        NumberRangeFilter, NumberRangeFilterExt,
        DateRangeFilter,
        FacetedFilter, FacetedFilterExt,
        TableStatusBar,
    };
}
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `derive` | Yes | Include procedural macros |
| `chrono` | Yes | DateTime type support |
| `component` | No | Include filter UI components |
| `fluent` | No | Internationalization support |
| `rust_decimal` | No | Decimal type support |
| `inventory` | No | Runtime registry metadata |

## Feature Interactions

```
gpui-table
├── derive (default)
│   └── Enables: GpuiTable, gpui_table_impl, Filterable, TableCell macros
├── chrono (default)
│   └── Enables: DateTime cell rendering, DateRangeFilter
├── component
│   └── Enables: All filter UI components
├── fluent
│   └── Enables: i18n for table titles and labels
├── rust_decimal
│   └── Enables: Decimal cell rendering, NumberRangeFilter precision
└── inventory
    └── Enables: GpuiTableShape registry collection
```

## Usage Patterns

### Minimal Usage

```toml
[dependencies]
gpui-table = "0.5"
```

Includes derive macros and chrono support.

### With Filter Components

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component"] }
```

### Full Features

```toml
[dependencies]
gpui-table = { version = "0.5", features = ["component", "fluent", "rust_decimal", "inventory"] }
```

## Dependency Graph

```
gpui-table
├── gpui-table-core (always)
├── gpui-table-derive (when "derive" feature)
└── gpui-table-component (when "component" feature)
```

## Design Rationale

The umbrella crate pattern provides:

1. **Simplified imports** - Users add one dependency instead of three
2. **Feature flexibility** - Components can be excluded to reduce compile time
3. **Version coherence** - All sub-crates stay synchronized
4. **API stability** - Internal reorganization doesn't affect users

## Public API Surface

The crate exposes:

- **Traits:** `TableCell`, `TableRowMeta`, `TableRowStyle`, `TableDataLoader`, `TableLoader`
- **Filter traits:** `Matchable`, `FilterValuesExt`, `FilterEntitiesExt`, `Filterable`
- **Filter types:** `FilterConfig`, `FilterType`, `FacetedFilterOption`
- **Filter wrappers:** `TextValue`, `RangeValue`, `FacetedValue`
- **Registry:** `GpuiTableShape`, `ColumnVariant`, `FilterVariant`
- **Macros:** `GpuiTable`, `gpui_table_impl`, `Filterable`, `TableCell`
- **Components:** `TextFilter`, `NumberRangeFilter`, `DateRangeFilter`, `FacetedFilter`, `TableStatusBar`
