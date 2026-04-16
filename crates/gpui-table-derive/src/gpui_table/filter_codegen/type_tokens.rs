use crate::components::FilterComponents;

/// Get the filter component type tokens for code generation.
/// For FacetedFilter, the field_ty is required to generate the generic parameter.
///
/// Returns a tuple of (type_tokens, type_with_turbofish) where:
/// - type_tokens: For use in type position (e.g., `Entity<FacetedFilter<T>>`)
/// - type_with_turbofish: For use in expression position (e.g., `FacetedFilter::<T>::new_for()`)
pub(in crate::gpui_table) fn get_filter_type_tokens(
    filter: &FilterComponents,
    field_ty: Option<&syn::Type>,
) -> proc_macro2::TokenStream {
    filter.component_type_tokens(field_ty)
}

/// Get the registry filter type for a given filter component.
#[cfg(feature = "inventory")]
pub(in crate::gpui_table) fn get_registry_filter_type(
    filter: &FilterComponents,
) -> proc_macro2::TokenStream {
    filter.registry_filter_type_tokens()
}

/// Get the FilterType enum for runtime filter config.
pub(in crate::gpui_table) fn get_filter_type_expr(
    filter: &FilterComponents,
    field_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    filter.runtime_filter_type_expr(field_ty)
}
