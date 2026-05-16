mod components;
mod filterable;
mod gpui_table;
mod impl_attr;
mod table_cell;

use proc_macro::TokenStream;

/// Attribute macro for table delegate impl blocks.
///
/// This macro processes an `impl` block for a table delegate and generates
/// the appropriate `TableDelegate` trait method implementations based on
/// inner attributes.
///
/// # Supported Attributes
///
/// - `#[load_more]` - Marks a method as the load_more handler
/// - `#[threshold]` - Marks a const as the load_more threshold value
///
/// Note: the table struct must set `#[gpui_table(load_more)]` for the generated
/// delegate to call these load_more hooks.
///
/// # Example
///
/// ```ignore
/// #[gpui_table_impl]
/// impl ProductTableDelegate {
///     #[threshold]
///     const LOAD_MORE_THRESHOLD: usize = 20;
///
///     #[load_more]
///     pub fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
///         // Load more data...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn gpui_table_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_attr::gpui_table_impl(attr.into(), item.into()).into()
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
