//! GPUI-facing runtime contracts for generated and manual table integrations.
//!
//! This crate owns row rendering traits, load-more traits, default cell
//! rendering, filter-shape contracts, and the stable
//! [`generated_filters`] facade targeted by `#[derive(gpui_table::GpuiTable)]`
//! output. Generated code should keep using this runtime facade instead of
//! depending on concrete component internals.

mod cell;
pub mod generated_filters;
mod load;
mod row;
pub mod shape;

pub use cell::{DisplayCell, FormattedCell, TableCell};
pub use generated_filters::FilterEntitiesExt;
pub use load::{TableDataLoader, TableLoader};
pub use row::{
    TableRowContextMenu, TableRowGeneratedContextMenu, TableRowMeta, TableRowStyle,
    default_render_cell, default_render_row,
};
pub use shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape, McpInput, McpInputShape,
    McpPrimitiveKind,
};

/// Private module for macro internals. Not part of public API.
#[doc(hidden)]
pub mod __private {
    pub use crate::load::LoadMoreDelegate;
}
