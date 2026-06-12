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
