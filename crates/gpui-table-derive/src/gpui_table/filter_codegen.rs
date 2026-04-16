use crate::components::{FilterComponents, TextValidation};

use quote::ToTokens as _;
use quote::quote;
use syn::{GenericArgument, Ident, PathArguments, Type};

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
    match filter {
        FilterComponents::Text(_) => validate_text_filter_field_type(field_ty)?,
        FilterComponents::Faceted(_) => validate_faceted_filter_field_type(field_ty)?,
        FilterComponents::InfiniteFaceted(_) => {
            validate_infinite_faceted_filter_field_type(field_ty)?
        },
        _ => {},
    }

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

            validate_number_range_filter_field_type(field_ty)?;
        }
    }

    #[cfg(feature = "chrono")]
    if matches!(filter, FilterComponents::DateRange(_)) {
        validate_date_range_filter_field_type(field_ty)?;
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

fn validate_text_filter_field_type(field_ty: &Type) -> syn::Result<()> {
    let value_ty = option_inner_type(field_ty).unwrap_or(field_ty);
    if is_string_like_type(value_ty) || !is_obviously_non_text_filter_type(value_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(text())` on `{type_name}` expects `String`, `&str`, or an `Option<...>` / local type that implements `AsRef<str>`"
        ),
    ))
}

#[cfg(feature = "rust_decimal")]
fn validate_number_range_filter_field_type(field_ty: &Type) -> syn::Result<()> {
    let value_ty = option_inner_type(field_ty).unwrap_or(field_ty);
    if is_supported_number_range_type(value_ty) || !is_obviously_non_number_range_type(value_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(number_range(...))` on `{type_name}` requires a field type supported by `gpui_table::core::filter::ToDecimal`; built-in support covers numeric scalars, `rust_decimal::Decimal`, and `Option<...>` of those"
        ),
    ))
}

#[cfg(feature = "chrono")]
fn validate_date_range_filter_field_type(field_ty: &Type) -> syn::Result<()> {
    let value_ty = option_inner_type(field_ty).unwrap_or(field_ty);
    if is_supported_date_range_type(value_ty) || !is_obviously_non_date_range_type(value_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(date_range())` on `{type_name}` requires a field type supported by `gpui_table::core::filter::ToNaiveDate`; built-in support covers `chrono::NaiveDate`, `chrono::NaiveDateTime`, `chrono::DateTime<_>`, and `Option<...>` of those"
        ),
    ))
}

fn validate_faceted_filter_field_type(field_ty: &Type) -> syn::Result<()> {
    if let Some(inner_ty) = option_inner_type(field_ty) {
        return Err(syn::Error::new_spanned(
            field_ty,
            format!(
                "`filter(faceted(...))` does not support `Option<{}>`; use a non-optional field type that implements `gpui_table::core::filter::Filterable`",
                type_name(inner_ty)
            ),
        ));
    }

    if is_bool_type(field_ty) || !is_obviously_non_faceted_filter_type(field_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(faceted(...))` on `{type_name}` requires a non-optional field type that implements `gpui_table::core::filter::Filterable`; `bool` works out of the box and enums can `#[derive(Filterable)]`"
        ),
    ))
}

fn validate_infinite_faceted_filter_field_type(field_ty: &Type) -> syn::Result<()> {
    if let Some(inner_ty) = option_inner_type(field_ty) {
        return Err(syn::Error::new_spanned(
            field_ty,
            format!(
                "`filter(infinite_faceted_filter())` does not support `Option<{}>`; use the selected item type directly",
                type_name(inner_ty)
            ),
        ));
    }

    if !is_obviously_non_infinite_faceted_filter_type(field_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(infinite_faceted_filter())` on `{type_name}` requires a non-optional field type that implements `gpui_form_component::infinite_select::InfiniteSelect`, `PartialEq`, and `Send`"
        ),
    ))
}

fn type_name(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn normalized_type(ty: &Type) -> &Type {
    match ty {
        Type::Group(group) => normalized_type(&group.elem),
        Type::Paren(paren) => normalized_type(&paren.elem),
        _ => ty,
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = normalized_type(ty) else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };

    Some(inner)
}

fn has_last_segment(ty: &Type, expected: &[&str]) -> bool {
    let Type::Path(type_path) = normalized_type(ty) else {
        return false;
    };

    type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| expected.iter().any(|name| segment.ident == name))
}

fn is_string_like_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(reference) => is_string_like_type(&reference.elem),
        _ => has_last_segment(ty, &["String", "str"]),
    }
}

fn is_bool_type(ty: &Type) -> bool {
    has_last_segment(ty, &["bool"])
}

fn is_char_type(ty: &Type) -> bool {
    has_last_segment(ty, &["char"])
}

fn is_numeric_scalar_type(ty: &Type) -> bool {
    has_last_segment(
        ty,
        &[
            "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32", "f64",
        ],
    )
}

fn is_decimal_type(ty: &Type) -> bool {
    has_last_segment(ty, &["Decimal"])
}

fn is_chrono_date_type(ty: &Type) -> bool {
    has_last_segment(ty, &["NaiveDate", "NaiveDateTime", "DateTime"])
}

fn is_collection_type(ty: &Type) -> bool {
    has_last_segment(ty, &["Vec", "HashSet", "BTreeSet", "HashMap", "BTreeMap"])
}

fn is_structural_non_scalar_type(ty: &Type) -> bool {
    matches!(
        normalized_type(ty),
        Type::Array(_) | Type::Slice(_) | Type::Tuple(_)
    )
}

#[cfg(feature = "rust_decimal")]
fn is_supported_number_range_type(ty: &Type) -> bool {
    is_numeric_scalar_type(ty) || is_decimal_type(ty) || contains_spacetimedb_temporal_type(ty)
}

fn is_supported_date_range_type(ty: &Type) -> bool {
    is_chrono_date_type(ty) || contains_spacetimedb_temporal_type(ty)
}

fn is_obviously_non_text_filter_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(reference) => is_obviously_non_text_filter_type(&reference.elem),
        _ => {
            is_bool_type(ty)
                || is_char_type(ty)
                || is_numeric_scalar_type(ty)
                || is_decimal_type(ty)
                || is_chrono_date_type(ty)
                || contains_spacetimedb_temporal_type(ty)
                || is_collection_type(ty)
                || is_structural_non_scalar_type(ty)
        },
    }
}

#[cfg(feature = "rust_decimal")]
fn is_obviously_non_number_range_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(_) => true,
        _ => {
            is_string_like_type(ty)
                || is_bool_type(ty)
                || is_char_type(ty)
                || is_chrono_date_type(ty)
                || is_collection_type(ty)
                || is_structural_non_scalar_type(ty)
        },
    }
}

fn is_obviously_non_date_range_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(_) => true,
        _ => {
            is_string_like_type(ty)
                || is_bool_type(ty)
                || is_char_type(ty)
                || is_numeric_scalar_type(ty)
                || is_decimal_type(ty)
                || is_collection_type(ty)
                || is_structural_non_scalar_type(ty)
        },
    }
}

fn is_obviously_non_faceted_filter_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(_) => true,
        _ => {
            is_string_like_type(ty)
                || is_char_type(ty)
                || is_numeric_scalar_type(ty)
                || is_decimal_type(ty)
                || is_chrono_date_type(ty)
                || contains_spacetimedb_temporal_type(ty)
                || is_collection_type(ty)
                || is_structural_non_scalar_type(ty)
        },
    }
}

fn is_obviously_non_infinite_faceted_filter_type(ty: &Type) -> bool {
    match normalized_type(ty) {
        Type::Reference(_) => true,
        _ => {
            is_string_like_type(ty)
                || is_bool_type(ty)
                || is_char_type(ty)
                || is_numeric_scalar_type(ty)
                || is_decimal_type(ty)
                || is_chrono_date_type(ty)
                || contains_spacetimedb_temporal_type(ty)
                || is_collection_type(ty)
                || is_structural_non_scalar_type(ty)
        },
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
