//! Facade crate for the `gpui-table` ecosystem.
//!
//! Public API is namespaced by responsibility:
//! - `gpui_table::core` for pure filter semantics
//! - `gpui_table::component` for built-in GPUI filter widgets and shape impls
//! - `gpui_table::mcp` for experimental MCP table query integration
//! - `gpui_table::runtime` for GPUI-facing traits/helpers
//! - `gpui_table::schema` for metadata and registry types
//! - root-level proc macros from `gpui-table-derive`, including
//!   `GpuiTableFilterShape` for adapting existing filter shapes
//!
//! The facade owns the stable paths that macro-generated code names. Changes to
//! root re-exports, `runtime::generated_filters`, `registry`, `__deps`, or
//! `__private` must stay aligned with `gpui-table-derive`, examples, and UI
//! compile-fail fixtures.
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
//! - `gpui_table_component::NumberRangeFilter` requires the `rust_decimal` feature.
//! - `gpui_table_component::DateRangeFilter` requires the `chrono` feature.
//! - Filtering SpacetimeDB `Timestamp`/`TimeDuration` with range filters requires the `spacetimedb` feature.
//! - `#[gpui_table(filters)]` is required to generate filter entities and matching logic.
//!
//! These are validated during macro expansion with direct compile-time errors.

#[cfg(feature = "derive")]
pub use gpui_table_derive::*;

pub use gpui_table_component as component;
pub use gpui_table_core as core;
pub use gpui_table_core::filter;
#[cfg(feature = "mcp")]
pub use gpui_table_mcp as mcp;
pub use gpui_table_runtime as runtime;
pub use gpui_table_runtime::{
    FilterEntitiesExt, TableCell, TableDataLoader, TableId, TableLoader, TableRowContextMenu,
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
