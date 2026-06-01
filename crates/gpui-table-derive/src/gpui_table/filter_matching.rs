use crate::gpui_table::meta::FilterFieldMeta;

use quote::quote;
use syn::Ident;

/// Generate the matches_filters() method on the struct.
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
        .map(|f| f.filter_config.matches_field_expr(&f.field_ident))
        .collect();

    quote! {
        impl gpui_table::core::filter::Matchable<#filter_values_name> for #struct_name {
            fn matches_filters(&self, filters: &#filter_values_name) -> bool {
                #((#match_exprs))&&*
            }
        }
    }
}
