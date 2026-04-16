use crate::components::{FilterComponents, TextValidation};

use quote::ToTokens as _;
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
        FilterComponents::InfiniteFaceted(_) => {
            if let Some(ty) = field_ty {
                quote! { gpui_table::__deps::gpui_table_component::infinite_faceted_filter::InfiniteFacetedFilter::<#ty> }
            } else {
                quote! { gpui_table::__deps::gpui_table_component::infinite_faceted_filter::InfiniteFacetedFilter::<String> }
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
        FilterComponents::InfiniteFaceted(_) => {
            quote! { gpui_table::registry::RegistryFilterType::InfiniteFaceted }
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
        FilterComponents::InfiniteFaceted(_) => {
            quote! { gpui_table::filter::FilterType::InfiniteFaceted }
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
        FilterComponents::InfiniteFaceted(_opts) => {
            quote! {}
        },
    }
}

pub(super) fn validate_filter_config(
    filter: &FilterComponents,
    field_ident: &Ident,
    field_ty: &syn::Type,
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

    #[cfg(not(feature = "rust_decimal"))]
    if matches!(filter, FilterComponents::NumberRange(_)) {
        return Err(syn::Error::new(
            field_ident.span(),
            "`filter(number_range(...))` requires enabling the `gpui-table/rust_decimal` feature",
        ));
    }

    #[cfg(not(feature = "chrono"))]
    if matches!(filter, FilterComponents::DateRange(_)) {
        return Err(syn::Error::new(
            field_ident.span(),
            "`filter(date_range())` requires enabling the `gpui-table/chrono` feature",
        ));
    }

    #[cfg(not(feature = "spacetimedb"))]
    if matches!(
        filter,
        FilterComponents::NumberRange(_) | FilterComponents::DateRange(_)
    ) && contains_spacetimedb_temporal_type(field_ty)
    {
        let type_name = field_ty.to_token_stream().to_string();
        return Err(syn::Error::new(
            field_ident.span(),
            format!(
                "`filter({})` on `{type_name}` requires enabling the `gpui-table/spacetimedb` feature",
                filter_name(filter)
            ),
        ));
    }

    Ok(())
}

fn filter_name(filter: &FilterComponents) -> &'static str {
    match filter {
        FilterComponents::Text(_) => "text()",
        FilterComponents::NumberRange(_) => "number_range(...)",
        FilterComponents::DateRange(_) => "date_range()",
        FilterComponents::Faceted(_) => "faceted(...)",
        FilterComponents::InfiniteFaceted(_) => "infinite_faceted_filter()",
    }
}

fn contains_spacetimedb_temporal_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            let Some(last_segment) = type_path.path.segments.last() else {
                return false;
            };

            if last_segment.ident == "Option"
                && let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
            {
                return args.args.iter().any(|arg| match arg {
                    syn::GenericArgument::Type(inner_ty) => {
                        contains_spacetimedb_temporal_type(inner_ty)
                    },
                    _ => false,
                });
            }

            let last_ident = last_segment.ident.to_string();
            matches!(last_ident.as_str(), "Timestamp" | "TimeDuration")
                && type_path.path.segments.iter().any(|segment| {
                    matches!(
                        segment.ident.to_string().as_str(),
                        "spacetimedb_lib" | "spacetimedb"
                    )
                })
        },
        syn::Type::Group(group) => contains_spacetimedb_temporal_type(&group.elem),
        syn::Type::Paren(paren) => contains_spacetimedb_temporal_type(&paren.elem),
        syn::Type::Reference(reference) => contains_spacetimedb_temporal_type(&reference.elem),
        _ => false,
    }
}
