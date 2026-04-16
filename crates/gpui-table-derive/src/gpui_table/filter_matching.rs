use crate::components::FilterComponents;
use crate::gpui_table::meta::FilterFieldMeta;

use quote::quote;
use syn::{GenericArgument, Ident, PathArguments, Type};

/// Generate the matches_filters() method on the struct.
/// This method checks if all filter values match the struct's fields.
pub(super) fn generate_matches_filters_method(
    struct_name: &Ident,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    if filter_fields.is_empty() {
        return quote! {};
    }

    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());

    let match_exprs: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let is_option = option_inner_type(&f.field_type).is_some();

            match &f.filter_config {
                FilterComponents::Text(_) => {
                    if is_option {
                        quote! {
                            if filters.#field_ident.is_active() {
                                self.#field_ident
                                    .as_ref()
                                    .is_some_and(|value| filters.#field_ident.matches(value.as_ref()))
                            } else {
                                true
                            }
                        }
                    } else {
                        quote! { filters.#field_ident.matches(self.#field_ident.as_ref()) }
                    }
                }
                FilterComponents::NumberRange(_) => {
                    if is_option {
                        quote! {
                            if filters.#field_ident.is_active() {
                                self.#field_ident
                                    .as_ref()
                                    .map(|value| {
                                        filters.#field_ident.matches(
                                            &gpui_table::core::filter::ToDecimal::to_decimal(value),
                                        )
                                    })
                                    .unwrap_or(false)
                            } else {
                                true
                            }
                        }
                    } else {
                        quote! { filters.#field_ident.matches(&gpui_table::core::filter::ToDecimal::to_decimal(&self.#field_ident)) }
                    }
                }
                FilterComponents::DateRange(_) => {
                    if is_option {
                        quote! {
                            if filters.#field_ident.is_active() {
                                self.#field_ident
                                    .as_ref()
                                    .map(|value| {
                                        filters.#field_ident.matches(
                                            &gpui_table::core::filter::ToNaiveDate::to_naive_date(value),
                                        )
                                    })
                                    .unwrap_or(false)
                            } else {
                                true
                            }
                        }
                    } else {
                        quote! { filters.#field_ident.matches(&gpui_table::core::filter::ToNaiveDate::to_naive_date(&self.#field_ident)) }
                    }
                }
                FilterComponents::Faceted(_) => {
                    quote! { filters.#field_ident.matches(&self.#field_ident) }
                }
                FilterComponents::InfiniteFaceted(_) => {
                    quote! { filters.#field_ident.matches(&self.#field_ident) }
                }
            }
        })
        .collect();

    quote! {
        impl gpui_table::core::filter::Matchable<#filter_values_name> for #struct_name {
            fn matches_filters(&self, filters: &#filter_values_name) -> bool {
                #((#match_exprs))&&*
            }
        }
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
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
