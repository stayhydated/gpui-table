# gpui-table-derive Architecture

This document describes the internal architecture of the `gpui-table-derive` crate.

## Overview

`gpui-table-derive` is a procedural macro crate that generates table infrastructure code from annotated structs. It transforms simple struct definitions into fully-functional table implementations with optional filtering, styling, and data loading capabilities.

## Module Structure

```
src/
├── lib.rs              # Main macro definitions and code generation
├── impl_attr.rs        # #[gpui_table_impl] attribute macro
├── components.rs       # Filter component configuration types
└── __crate_paths/      # Auto-generated crate path resolution
    └── mod.rs
```

## Macros

### `#[derive(GpuiTable)]`

The primary derive macro. Transforms a struct into a table row type.

**Supported Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `id` | String | Unique table identifier |
| `title` | String | Display title for the table |
| `delegate` | Flag | Generate a TableDelegate struct |
| `custom_style` | Flag | Skip default TableRowStyle generation |
| `fluent` | Flag | Use fluent for i18n |
| `loading` | Flag | Include loading state in delegate |
| `filters` | Flag | Generate filter entities and values |

**Field Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `title` | String | Column header title |
| `width` | f32 | Column width in pixels |
| `sortable` | Flag | Enable sorting on this column |
| `fixed` | "left"/"right" | Fix column position |
| `filter` | FilterConfig | Add filter for this field |

### `#[gpui_table_impl]`

Attribute macro for impl blocks that bridges user-defined loading logic to generated delegates.

**Method Attributes:**

- `#[load_more]` - Mark async method as the data loader
- `#[threshold]` - Mark const as the loading threshold

### `#[derive(Filterable)]`

Derive for enum types to make them usable as faceted filter options.

### `#[derive(TableCell)]`

Derive for custom types to enable cell rendering.

## Code Generation

### Generated Types

When `#[derive(GpuiTable)]` is applied to a struct `Foo`:

```rust
// Column enum
pub enum FooTableColumn {
    Column0,
    Column1,
    // ...
}

// Delegate (when delegate = true)
pub struct FooTableDelegate {
    rows: Vec<Foo>,
    loading: bool,  // when loading = true
    eof: bool,
    // ...
}

// Filter entities (when filters = true)
pub struct FooFilterEntities {
    pub field_name: Entity<FilterComponent>,
    // ...
}

// Filter values (when filters = true)
pub struct FooFilterValues {
    pub field_name: FilterValueType,
    // ...
}
```

### Generated Trait Implementations

```rust
impl TableRowMeta for Foo { ... }
impl TableRowStyle for Foo { ... }  // unless custom_style = true
impl TableDelegate for FooTableDelegate { ... }
impl FilterEntitiesExt<FooFilterValues> for FooFilterEntities { ... }
impl Matchable<FooFilterValues> for Foo { ... }
```

## Filter Component Configuration

The `components.rs` module defines filter component types used in attribute parsing:

```rust
pub enum FilterComponents {
    Text(TextFilterOptions),
    NumberRange(NumberRangeFilterOptions),
    DateRange(DateRangeFilterOptions),
    Faceted(FacetedFilterOptions),
}

pub struct TextFilterOptions {
    pub validation: TextValidation,
}

pub struct NumberRangeFilterOptions {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

pub struct DateRangeFilterOptions {
    // Date range specific options
}

pub struct FacetedFilterOptions {
    pub searchable: bool,
}
```

## Code Generation Flow

```
1. Parse struct with darling
   ↓
2. Extract field metadata (titles, widths, filters)
   ↓
3. Generate column enum
   ↓
4. Generate TableRowMeta impl
   ↓
5. Generate TableRowStyle impl (unless custom_style)
   ↓
6. If delegate = true:
   ├── Generate delegate struct
   └── Generate TableDelegate impl
   ↓
7. If filters = true:
   ├── Generate FilterEntities struct
   ├── Generate FilterValues struct
   ├── Generate FilterEntitiesExt impl
   └── Generate Matchable impl
   ↓
8. If inventory feature:
   └── Generate GpuiTableShape registration
```

## impl_attr.rs Details

The `#[gpui_table_impl]` macro:

1. Finds methods marked with `#[load_more]`
2. Validates method signature (must be async, return appropriate type)
3. Finds consts marked with `#[threshold]`
4. Generates `LoadMoreDelegate` implementation bridging to user code

## Dependencies

- `proc-macro2` - Token stream manipulation
- `quote` - Code generation
- `syn` - Rust syntax parsing
- `darling` - Derive macro helper
- `heck` - Case conversion
- `inventory` - Registry support
- `crate-paths` - Path resolution

## Feature Flags

| Feature | Description |
|---------|-------------|
| `inventory` | Enable registry metadata collection |
