# Project Overview

gpui-table is a Rust workspace providing a table component system for GPUI (the GUI framework from Zed). It includes derive macros, filter components, and pagination support for building feature-rich tables.

## Crate Reference

| Crate | Link to Architecture Doc | Purpose |
|-------|-------------------------|---------|
| gpui-table | [ARCHITECTURE](crates/gpui-table/docs/ARCHITECTURE.md) | Main umbrella crate re-exporting all functionality |
| gpui-table-core | [ARCHITECTURE](crates/gpui-table-core/docs/ARCHITECTURE.md) | Core traits, types, filter system, and registry |
| gpui-table-derive | [ARCHITECTURE](crates/gpui-table-derive/docs/ARCHITECTURE.md) | Procedural macros for code generation |
| gpui-table-component | [ARCHITECTURE](crates/gpui-table-component/docs/ARCHITECTURE.md) | Filter UI components (text, range, date, faceted) |
| gpui-table-prototyping-core | [ARCHITECTURE](crates/gpui-table-prototyping-core/docs/ARCHITECTURE.md) | Code generation utilities for prototyping/storybook |

## Crate Descriptions

### gpui-table

The main umbrella crate that users depend on. It re-exports functionality from the other crates based on enabled feature flags. Users typically only need to add this single dependency to get full table functionality.

**Key exports:** `GpuiTable`, `gpui_table_impl`, `TableCell`, `TableRowMeta`, `TableRowStyle`, filter traits and types.

**Feature flags:**
- `derive` (default) - Include procedural macros
- `chrono` (default) - DateTime support
- `component` - Filter UI components
- `fluent` - Internationalization
- `rust_decimal` - Decimal type support
- `inventory` - Runtime metadata registry

### gpui-table-core

The foundation crate defining all core abstractions. Other crates depend on this for shared types and traits.

**Key types:**
- `TableCell` - Trait for rendering values in cells
- `TableRowMeta` - Trait defining table structure
- `TableRowStyle` - Trait for customizing rendering
- `TableLoader` / `TableDataLoader` - Traits for data loading
- `FilterConfig`, `FilterType` - Filter configuration
- `Matchable<F>` - Trait for filtering rows
- `GpuiTableShape` - Registry metadata type

### gpui-table-derive

Procedural macro crate that generates boilerplate code from annotated structs.

**Key macros:**
- `#[derive(GpuiTable)]` - Main derive generating table metadata, delegates, and filters
- `#[gpui_table_impl]` - Attribute macro for bridging user loading logic
- `#[derive(Filterable)]` - Makes enums usable as faceted filter options
- `#[derive(TableCell)]` - Enables custom types for cell rendering

**Generated types:** For a struct `Foo`:
- `FooTableColumn` - Column identifier enum
- `FooTableDelegate` - Table delegate struct
- `FooFilterEntities` - Filter UI entity handles
- `FooFilterValues` - Filter value struct

### gpui-table-component

UI components for table filtering, built on GPUI.

**Components:**
- `TextFilter` - Debounced text input with validation
- `NumberRangeFilter` - Dual input with slider for numeric ranges
- `DateRangeFilter` - Calendar-based date range picker
- `FacetedFilter<T>` - Multi-select checkboxes for enums
- `TableStatusBar` - Row count and status display

All filter components implement `TableFilterComponent` trait providing a uniform interface.

### gpui-table-prototyping-core

Utilities for generating table UI code from registry metadata. Used by storybook generators and prototyping tools.

**Key types:**
- `ColumnInfo` - Wrapper with column utilities
- `ShapeIdentities` - Derives identifiers from table metadata
- `TableShapeAdapter` - Generates code structures from shapes

## Build Commands

```bash
# Build all crates
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test

# Check specific crate
cargo check -p gpui-table-core
```

## Architecture Patterns

1. **Derive Macro Pattern** - `#[derive(GpuiTable)]` generates all table infrastructure
2. **Feature-Gated Re-exports** - Umbrella crate conditionally includes functionality
3. **Registry Pattern** - `inventory` crate collects metadata at compile time
4. **Extension Traits** - Configuration via traits like `TextFilterExt`, `NumberRangeFilterExt`
5. **Entity System** - GPUI entities wrap filter components for reactive updates
