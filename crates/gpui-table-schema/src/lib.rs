//! UI-neutral metadata shared across the `gpui-table` workspace.
//!
//! The schema crate describes filters and inventory-backed table shapes without
//! depending on GPUI. Derive output submits `'static` registry values here, and
//! tooling such as `gpui-table-prototyping-core` consumes those values to build
//! examples or scaffolding.

pub mod filter;
pub mod registry;
