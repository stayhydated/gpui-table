//! Code-generation helpers for inventory-registered `gpui-table` shapes.
//!
//! This crate turns `GpuiTableShape` metadata into Rust syntax trees for table
//! stories or scaffolding. It depends on schema metadata and Rust syntax tools,
//! not on the GPUI runtime. Prefer the `try_*` APIs when consuming untrusted or
//! externally produced metadata so invalid shapes report `TableCodegenError`
//! instead of panicking.

pub mod code_gen;
pub mod column;
mod identities;
mod source_path;

pub use code_gen::{TableCodegenError, TableLayout, TableParts, TableShapeAdapter};
pub use identities::{TableIdentities, TableIdentitiesExt};
