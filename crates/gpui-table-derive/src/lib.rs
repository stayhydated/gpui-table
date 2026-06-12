//! Proc macros for the `gpui-table` derive workflow.
//!
//! `#[derive(GpuiTable)]` emits generated table delegates, column enums, row
//! metadata, optional filter entities and values, optional inventory metadata,
//! optional context-menu helpers, and optional MCP query metadata. The emitted
//! code depends on stable paths re-exported by the `gpui-table` facade.
//!
//! `#[derive(Filterable)]`, `#[derive(TableCell)]`, `#[derive(McpFilterShape)]`
//! when the `mcp` feature is enabled, and `#[gpui_table_impl]` provide the
//! supporting generated contracts used by the public README examples and UI
//! compile-fail fixtures.

mod components;
mod filterable;
mod gpui_table;
mod impl_attr;
#[cfg(feature = "mcp")]
mod mcp_filter_shape;
#[cfg(feature = "mcp")]
mod mcp_handlers;
mod table_cell;

use proc_macro::TokenStream;

/// Attribute macro for table delegate impl blocks.
///
/// This macro processes a `TableLoader` impl for a generated table delegate and
/// wires it into the generated `TableDelegate` implementation.
///
/// Note: the table struct must set `#[gpui_table(load_more)]` for the generated
/// delegate to call these load_more hooks.
///
/// # Example
///
/// ```ignore
/// use gpui_table::runtime::TableLoader;
///
/// #[gpui_table_impl]
/// impl TableLoader for ProductTableDelegate {
///     const THRESHOLD: usize = 20;
///
///     pub fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
///         // Load more data...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn gpui_table_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_attr::gpui_table_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "mcp")]
#[proc_macro_attribute]
pub fn mcp_query(attr: TokenStream, item: TokenStream) -> TokenStream {
    match mcp_handlers::expand_query(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[cfg(feature = "mcp")]
#[proc_macro_derive(McpFilterShape)]
pub fn derive_mcp_filter_shape(input: TokenStream) -> TokenStream {
    match mcp_filter_shape::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(GpuiTable, attributes(gpui_table))]
pub fn derive_gpui_table(input: TokenStream) -> TokenStream {
    gpui_table::derive_gpui_table(input)
}

#[proc_macro_derive(Filterable, attributes(filter))]
pub fn derive_filterable(input: TokenStream) -> TokenStream {
    filterable::derive_filterable(input)
}

#[proc_macro_derive(TableCell, attributes(table_cell))]
pub fn derive_table_cell(input: TokenStream) -> TokenStream {
    table_cell::derive_table_cell(input)
}
