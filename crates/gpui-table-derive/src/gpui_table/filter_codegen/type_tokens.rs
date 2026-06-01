use crate::components::ResolvedFilterShape;

pub(in crate::gpui_table) fn get_filter_type_tokens(
    filter: &ResolvedFilterShape,
) -> proc_macro2::TokenStream {
    filter.component_type_tokens()
}

#[cfg(feature = "inventory")]
pub(in crate::gpui_table) fn get_registry_filter_type(
    filter: &ResolvedFilterShape,
) -> proc_macro2::TokenStream {
    filter.registry_kind_tokens()
}

pub(in crate::gpui_table) fn get_filter_type_expr(
    filter: &ResolvedFilterShape,
) -> proc_macro2::TokenStream {
    filter.runtime_filter_type_expr()
}
