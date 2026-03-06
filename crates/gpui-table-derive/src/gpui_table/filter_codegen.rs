use crate::components::{FilterComponents, TextValidation};
use crate::gpui_table::meta::FilterFieldMeta;

use quote::quote;
use syn::Ident;

/// Get the filter component type tokens for code generation.
/// For FacetedFilter, the field_ty is required to generate the generic parameter.
///
/// Returns a tuple of (type_tokens, type_with_turbofish) where:
/// - type_tokens: For use in type position (e.g., `Entity<FacetedFilter<T>>`)
/// - type_with_turbofish: For use in expression position (e.g., `FacetedFilter::<T>::new_for()`)
pub(super) fn get_filter_type_tokens(
    filter: &FilterComponents,
    field_ty: Option<&syn::Type>,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => {
            quote! { gpui_table::__deps::gpui_table_component::text_filter::TextFilter }
        },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::__deps::gpui_table_component::number_range_filter::NumberRangeFilter }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::__deps::gpui_table_component::date_range_filter::DateRangeFilter }
        },
        FilterComponents::Faceted(_) => {
            if let Some(ty) = field_ty {
                quote! { gpui_table::__deps::gpui_table_component::faceted_filter::FacetedFilter::<#ty> }
            } else {
                // Fallback for cases where field_ty is not available (shouldn't happen in practice)
                quote! { gpui_table::__deps::gpui_table_component::faceted_filter::FacetedFilter::<String> }
            }
        },
    }
}

/// Get the registry filter type for a given filter component.
#[cfg(feature = "inventory")]
pub(super) fn get_registry_filter_type(filter: &FilterComponents) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => {
            quote! { gpui_table::registry::RegistryFilterType::Text }
        },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::registry::RegistryFilterType::NumberRange }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::registry::RegistryFilterType::DateRange }
        },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::registry::RegistryFilterType::Faceted }
        },
    }
}

/// Get the FilterType enum for runtime filter config.
pub(super) fn get_filter_type_expr(
    filter: &FilterComponents,
    field_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => quote! { gpui_table::filter::FilterType::Text },
        FilterComponents::NumberRange(_) => quote! { gpui_table::filter::FilterType::NumberRange },
        FilterComponents::DateRange(_) => quote! { gpui_table::filter::FilterType::DateRange },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::filter::FilterType::Faceted(<#field_ty as gpui_table::filter::Filterable>::options()) }
        },
    }
}

/// Generate chain method calls for filter options.
pub(super) fn generate_filter_chain_methods(filter: &FilterComponents) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(opts) => {
            let mut chain = quote! {};

            // Generate validation method if specified
            if let Some(ref validation) = opts.validate {
                let validation_chain = match validation {
                    TextValidation::Alphabetic => quote! {
                        use gpui_table::__deps::gpui_table_component::text_filter::TextFilterExt as _;
                        let filter = filter.alphabetic_only(cx);
                    },
                    TextValidation::Numeric => quote! {
                        use gpui_table::__deps::gpui_table_component::text_filter::TextFilterExt as _;
                        let filter = filter.numeric_only(cx);
                    },
                    TextValidation::Alphanumeric => quote! {
                        use gpui_table::__deps::gpui_table_component::text_filter::TextFilterExt as _;
                        let filter = filter.alphanumeric_only(cx);
                    },
                    TextValidation::Custom(path) => quote! {
                        use gpui_table::__deps::gpui_table_component::text_filter::TextFilterExt as _;
                        let filter = filter.validate(#path, cx);
                    },
                };
                chain = quote! { #chain #validation_chain };
            }

            chain
        },
        FilterComponents::NumberRange(opts) => {
            let mut chain = quote! {};

            // Generate .range() call if min or max is specified
            if opts.min.is_some() || opts.max.is_some() {
                let min_val = opts.min.unwrap_or(0.0);
                let max_val = opts.max.unwrap_or(100.0);
                // Convert f64 to string for Decimal parsing at compile time
                let min_str = min_val.to_string();
                let max_str = max_val.to_string();
                chain = quote! {
                    #chain
                    use gpui_table::__deps::gpui_table_component::number_range_filter::NumberRangeFilterExt as _;
                    let filter = filter.range(
                        gpui_table::__deps::rust_decimal::Decimal::from_str_exact(#min_str).unwrap(),
                        gpui_table::__deps::rust_decimal::Decimal::from_str_exact(#max_str).unwrap(),
                        cx,
                    );
                };
            }

            // Generate .step() call if step is specified
            if let Some(step_val) = opts.step {
                let step_str = step_val.to_string();
                chain = quote! {
                    #chain
                    let filter = filter.step(gpui_table::__deps::rust_decimal::Decimal::from_str_exact(#step_str).unwrap(), cx);
                };
            }

            chain
        },
        FilterComponents::DateRange(_opts) => {
            // Date range filter has no configurable options yet
            quote! {}
        },
        FilterComponents::Faceted(opts) => {
            let mut chain = quote! {};

            // Generate .searchable() call if enabled
            if opts.searchable {
                chain = quote! {
                    #chain
                    use gpui_table::__deps::gpui_table_component::faceted_filter::FacetedFilterExt as _;
                    let filter = filter.searchable(cx);
                };
            }

            chain
        },
    }
}

pub(super) fn validate_filter_config(
    filter: &FilterComponents,
    field_ident: &Ident,
) -> syn::Result<()> {
    if let FilterComponents::NumberRange(opts) = filter {
        if let Some(min) = opts.min
            && !min.is_finite()
        {
            return Err(syn::Error::new(
                field_ident.span(),
                "`number_range(min = ...)` must be a finite number",
            ));
        }
        if let Some(max) = opts.max
            && !max.is_finite()
        {
            return Err(syn::Error::new(
                field_ident.span(),
                "`number_range(max = ...)` must be a finite number",
            ));
        }
        if let Some(step) = opts.step {
            if !step.is_finite() {
                return Err(syn::Error::new(
                    field_ident.span(),
                    "`number_range(step = ...)` must be a finite number",
                ));
            }
            if step <= 0.0 {
                return Err(syn::Error::new(
                    field_ident.span(),
                    "`number_range(step = ...)` must be greater than 0",
                ));
            }
        }
        if let (Some(min), Some(max)) = (opts.min, opts.max)
            && min > max
        {
            return Err(syn::Error::new(
                field_ident.span(),
                "`number_range(min = ..., max = ...)` requires min <= max",
            ));
        }
    }

    Ok(())
}

pub(super) fn generate_filter_feature_assertions(
    struct_name: &Ident,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    let requires_rust_decimal = filter_fields
        .iter()
        .any(|f| matches!(&f.filter_config, FilterComponents::NumberRange(_)));
    let requires_chrono = filter_fields
        .iter()
        .any(|f| matches!(&f.filter_config, FilterComponents::DateRange(_)));

    let rust_decimal_assert = if requires_rust_decimal {
        quote! {
            impl #struct_name
            where
                (): gpui_table::__deps::RequiresRustDecimalFeatureOnGpuiTable,
            {}
        }
    } else {
        quote! {}
    };

    let chrono_assert = if requires_chrono {
        quote! {
            impl #struct_name
            where
                (): gpui_table::__deps::RequiresChronoFeatureOnGpuiTable,
            {}
        }
    } else {
        quote! {}
    };

    quote! {
        #rust_decimal_assert
        #chrono_assert
    }
}
