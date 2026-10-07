//! GPUI-facing runtime contracts for generated and manual table integrations.
//!
//! This crate owns row rendering traits, load-more traits, default cell
//! rendering, filter-shape contracts, and generic generated-filter helpers.
//! Built-in filter widget shape implementations live in `gpui-table-component`
//! so this runtime crate can be used without depending on concrete components.

mod cell;
pub mod generated_filters;
mod load;
pub mod ordering;
mod row;
pub use ordering::{
    OrderedTableDelegate, ResolvedOrderedTableDelegate, ResolvedTableColumn, ResolvedTableDelegate,
    TableRowSelection, set_table_ordering, set_table_resolved_columns,
    set_table_resolved_sort_columns,
};
pub mod shape;

pub use cell::{DisplayCell, FormattedCell, TableCell};
pub use generated_filters::{
    FilterEntitiesExt, FilterSidebarData, FilterSidebarGroup, FilterSidebarItem, icon_from_name,
};
pub use gpui_table_schema::registry::TableId;
pub use load::{TableDataLoader, TableLoader};
pub use row::{
    TableRowContextMenu, TableRowGeneratedContextMenu, TableRowMeta, TableRowStyle,
    default_render_cell, default_render_row,
};
pub use shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape, McpInput, McpInputShape,
    McpPrimitiveKind, McpRangeBoundKind,
};

/// Private module for macro internals. Not part of public API.
#[doc(hidden)]
pub mod __private {
    pub use crate::load::LoadMoreDelegate;
}
