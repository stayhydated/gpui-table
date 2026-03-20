//! Facade crate for the `gpui-table` ecosystem.
//!
//! This crate re-exports:
//! - `gpui-table-core` traits/types (`TableRowMeta`, `TableCell`, filters, loaders)
//! - `gpui-table-derive` macros (`GpuiTable`, `TableCell`, `Filterable`, `gpui_table_impl`)
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
//! - Filtering SpacetimeDB `Timestamp` with range filters requires the `spacetimedb` feature.
//! - `#[gpui_table(filters)]` is required to generate filter entities and matching logic.
//!
//! These are validated during macro expansion with compile-time errors.

#[cfg(feature = "derive")]
pub use gpui_table_derive::*;

pub use gpui_table_core::*;

pub use gpui_table_core::TableDataLoader;
pub use gpui_table_core::TableLoader;

/// Hidden dependency surface used by macro-generated code.
#[doc(hidden)]
pub mod __deps {
    pub use gpui_table_component;

    #[cfg(feature = "chrono")]
    pub use chrono;
    #[cfg(feature = "rust_decimal")]
    pub use rust_decimal;

    /// Marker trait used by generated code to fail clearly when
    /// `filter(number_range(...))` is used without enabling `gpui-table/rust_decimal`.
    pub trait RequiresRustDecimalFeatureOnGpuiTable {}
    #[cfg(feature = "rust_decimal")]
    impl RequiresRustDecimalFeatureOnGpuiTable for () {}

    /// Marker trait used by generated code to fail clearly when
    /// `filter(date_range(...))` is used without enabling `gpui-table/chrono`.
    pub trait RequiresChronoFeatureOnGpuiTable {}
    #[cfg(feature = "chrono")]
    impl RequiresChronoFeatureOnGpuiTable for () {}
}
