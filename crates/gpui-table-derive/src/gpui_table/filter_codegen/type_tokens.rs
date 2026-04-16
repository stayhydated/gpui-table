use crate::components::FilterComponents;
#[cfg(feature = "inventory")]
use crate::components::FilterKind;

use quote::quote;

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
    match filter {
        FilterComponents::Text(_) => {
            quote! { gpui_table::runtime::generated_filters::text_filter::TextFilter }
        },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::runtime::generated_filters::number_range_filter::NumberRangeFilter }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::runtime::generated_filters::date_range_filter::DateRangeFilter }
        },
        FilterComponents::Faceted(_) => {
            if let Some(ty) = field_ty {
                quote! { gpui_table::runtime::generated_filters::faceted_filter::FacetedFilter::<#ty> }
            } else {
                // Fallback for cases where field_ty is not available (shouldn't happen in practice)
                quote! { gpui_table::runtime::generated_filters::faceted_filter::FacetedFilter::<String> }
            }
        },
        FilterComponents::InfiniteFaceted(_) => {
            if let Some(ty) = field_ty {
                quote! { gpui_table::runtime::generated_filters::infinite_faceted_filter::InfiniteFacetedFilter::<#ty> }
            } else {
                quote! { gpui_table::runtime::generated_filters::infinite_faceted_filter::InfiniteFacetedFilter::<String> }
            }
        },
    }
}

/// Get the registry filter type for a given filter component.
#[cfg(feature = "inventory")]
pub(in crate::gpui_table) fn get_registry_filter_type(
    filter: &FilterComponents,
) -> proc_macro2::TokenStream {
    match filter.kind() {
        FilterKind::Text => {
            quote! { gpui_table::schema::registry::RegistryFilterType::Text }
        },
        FilterKind::NumberRange => {
            quote! { gpui_table::schema::registry::RegistryFilterType::NumberRange }
        },
        FilterKind::DateRange => {
            quote! { gpui_table::schema::registry::RegistryFilterType::DateRange }
        },
        FilterKind::Faceted => {
            quote! { gpui_table::schema::registry::RegistryFilterType::Faceted }
        },
        FilterKind::InfiniteFaceted => {
            quote! { gpui_table::schema::registry::RegistryFilterType::InfiniteFaceted }
        },
    }
}

/// Get the FilterType enum for runtime filter config.
pub(in crate::gpui_table) fn get_filter_type_expr(
    filter: &FilterComponents,
    field_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => quote! { gpui_table::core::filter::FilterType::Text },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::core::filter::FilterType::NumberRange }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::core::filter::FilterType::DateRange }
        },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::core::filter::FilterType::Faceted(<#field_ty as gpui_table::core::filter::Filterable>::options()) }
        },
        FilterComponents::InfiniteFaceted(_) => {
            quote! { gpui_table::core::filter::FilterType::InfiniteFaceted }
        },
    }
}
