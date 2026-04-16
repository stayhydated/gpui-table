//! Facade crate for the `gpui-table` ecosystem.
//!
//! Public API is namespaced by responsibility:
//! - `gpui_table::core` for pure filter semantics
//! - `gpui_table::runtime` for GPUI-facing traits/helpers
//! - `gpui_table::schema` for metadata and registry types
//! - root-level proc macros from `gpui-table-derive`
//!
//! # Quick Start
//!
//! ```ignore
//! use gpui_table::GpuiTable;
//!
//! #[derive(Clone, GpuiTable)]
//! struct User {
//!     name: String,
//!     age: u8,
//! }
//! ```
//!
//! # Filter Feature Requirements
//!
//! - `filter(number_range(...))` requires the `rust_decimal` feature.
//! - `filter(date_range(...))` requires the `chrono` feature.
//! - Filtering SpacetimeDB `Timestamp`/`TimeDuration` with range filters requires the `spacetimedb` feature.
//! - `#[gpui_table(filters)]` is required to generate filter entities and matching logic.
//!
//! These are validated during macro expansion with direct compile-time errors.

#[cfg(feature = "derive")]
pub use gpui_table_derive::*;

pub use gpui_table_core as core;
pub use gpui_table_core::filter;
pub use gpui_table_runtime as runtime;
pub use gpui_table_runtime::{
    FilterEntitiesExt, TableCell, TableDataLoader, TableLoader, TableRowContextMenu,
    TableRowGeneratedContextMenu, TableRowMeta, TableRowStyle,
};
pub use gpui_table_schema as schema;
pub use gpui_table_schema::registry;

/// Hidden dependency surface used by macro-generated code.
#[doc(hidden)]
pub mod __deps {
    #[cfg(feature = "chrono")]
    pub use chrono;
    #[cfg(feature = "rust_decimal")]
    pub use rust_decimal;
}

/// Hidden runtime bridge used by macro-generated code.
#[doc(hidden)]
pub mod __private {
    pub use gpui_table_runtime::__private::LoadMoreDelegate;
}
