use crate::components::FilterComponents;

use quote::ToTokens as _;
use syn::{GenericArgument, Ident, PathArguments, Type};

pub(in crate::gpui_table) fn validate_filter_config(
    filter: &FilterComponents,
    field_ident: &Ident,
    field_ty: &syn::Type,
) -> syn::Result<()> {
    match filter {
        FilterComponents::Text(_) => validate_text_filter_field_type(field_ty)?,
        FilterComponents::Faceted(_) => validate_faceted_filter_field_type(field_ty)?,
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
                filter.attribute_syntax()
            ),
        ));
    }

    Ok(())
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
    let value_ty = option_inner_type(field_ty).unwrap_or(field_ty);
    let value_ty = vec_inner_type(value_ty).unwrap_or(value_ty);
    if is_bool_type(value_ty) || !is_obviously_non_faceted_filter_type(value_ty) {
        return Ok(());
    }

    let type_name = type_name(field_ty);
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`filter(faceted(...))` on `{type_name}` requires a field type that implements `gpui_table::core::filter::Filterable`; `Option<T>` and `Vec<T>` work when `T` implements `Filterable`, `bool` works out of the box, and enums can `#[derive(Filterable)]`"
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

fn vec_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = normalized_type(ty) else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Vec" {
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

#[cfg(feature = "chrono")]
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

#[cfg(feature = "chrono")]
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
