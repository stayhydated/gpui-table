//! Pure filter semantics for the `gpui-table` workspace.
//!
//! This crate intentionally has no GPUI runtime dependency. It owns typed
//! filter wrappers, faceted value conversion, matching traits, and optional
//! feature-gated conversions used by generated and manual filtering flows.
//! Runtime widgets and derive macros build on this layer rather than
//! duplicating matching behavior.

pub mod filter;
#[cfg(feature = "fluent")]
pub mod i18n;
