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
pub(super) fn get_registry_filter_type(filter: &FilterComponents) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => {
            quote! { gpui_table::schema::registry::RegistryFilterType::Text }
        },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::schema::registry::RegistryFilterType::NumberRange }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::schema::registry::RegistryFilterType::DateRange }
        },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::schema::registry::RegistryFilterType::Faceted }
        },
        FilterComponents::InfiniteFaceted(_) => {
            quote! { gpui_table::schema::registry::RegistryFilterType::InfiniteFaceted }
        },
    }
}

/// Get the FilterType enum for runtime filter config.
pub(super) fn get_filter_type_expr(
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

/// Generate chain method calls for filter options.
pub(super) fn generate_filter_chain_methods(filter: &FilterComponents) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(opts) => {
            let mut chain = quote! {};

            // Generate validation method if specified
            if let Some(ref validation) = opts.validate {
                let validation_chain = match validation {
                    TextValidation::Alphabetic => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.alphabetic_only(cx);
                    },
                    TextValidation::Numeric => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.numeric_only(cx);
                    },
                    TextValidation::Alphanumeric => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.alphanumeric_only(cx);
                    },
                    TextValidation::Custom(path) => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
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
                #[cfg(feature = "rust_decimal")]
                let min_expr = opts
                    .min
                    .as_ref()
                    .map(|value| value.decimal_tokens("min"))
                    .unwrap_or_else(|| {
                        quote! {
                            gpui_table::__deps::rust_decimal::Decimal::from_i128_with_scale(0, 0)
                        }
                    });
                #[cfg(feature = "rust_decimal")]
                let max_expr = opts
                    .max
                    .as_ref()
                    .map(|value| value.decimal_tokens("max"))
                    .unwrap_or_else(|| {
                        quote! {
                            gpui_table::__deps::rust_decimal::Decimal::from_i128_with_scale(100, 0)
                        }
                    });

                #[cfg(not(feature = "rust_decimal"))]
                let (min_expr, max_expr) = (quote! {}, quote! {});

                chain = quote! {
                    #chain
                    use gpui_table::runtime::generated_filters::number_range_filter::NumberRangeFilterExt as _;
                    let filter = filter.range(
                        #min_expr,
                        #max_expr,
                        cx,
                    );
                };
            }

            // Generate .step() call if step is specified
            if let Some(step_val) = opts.step.as_ref() {
                #[cfg(feature = "rust_decimal")]
                let step_expr = step_val.decimal_tokens("step");
                #[cfg(not(feature = "rust_decimal"))]
                let step_expr = {
                    let _ = step_val;
                    quote! {}
                };

                chain = quote! {
                    #chain
                    let filter = filter.step(#step_expr, cx);
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
                    use gpui_table::runtime::generated_filters::faceted_filter::FacetedFilterExt as _;
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
        #[cfg(not(feature = "rust_decimal"))]
        let _ = opts;

        #[cfg(feature = "rust_decimal")]
        {
            let parsed_min = opts
                .min
                .as_ref()
                .map(|value| value.parse_decimal("min"))
                .transpose()?;
            let parsed_max = opts
                .max
                .as_ref()
                .map(|value| value.parse_decimal("max"))
                .transpose()?;
            let parsed_step = opts
                .step
                .as_ref()
                .map(|value| value.parse_decimal("step"))
                .transpose()?;

            if let Some(step) = parsed_step
                && step <= rust_decimal::Decimal::ZERO
            {
                return Err(syn::Error::new(
                    opts.step
                        .as_ref()
                        .map(|value| value.span())
                        .unwrap_or(field_ident.span()),
                    format!(
                        "`number_range(step = {})` must be greater than 0",
                        step.normalize()
                    ),
                ));
            }

            if let (Some(min), Some(max)) = (parsed_min, parsed_max)
                && min > max
            {
                return Err(syn::Error::new(
                    opts.max
                        .as_ref()
                        .map(|value| value.span())
                        .unwrap_or(field_ident.span()),
                    format!(
                        "`number_range(min = {}, max = {})` requires min <= max",
                        min.normalize(),
                        max.normalize()
                    ),
                ));
            }
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
