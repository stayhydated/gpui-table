use crate::mcp_handlers::resolve_crate_path;

use quote::quote;
use syn::{DeriveInput, parse_quote};

pub fn expand(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;

    let ident = input.ident;
    let facade_crate = resolve_crate_path("gpui-table", "::gpui_table");
    let mut generics = input.generics;
    let self_type = {
        let (_, ty_generics, _) = generics.split_for_impl();
        quote! { #ident #ty_generics }
    };

    generics.make_where_clause().predicates.push(parse_quote!(
        <#self_type as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue:
            #facade_crate::mcp::DeserializeOwned
                + #facade_crate::mcp::McpJsonSchema
    ));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #facade_crate::mcp::McpFilterShape for #ident #ty_generics
            #where_clause
        {
            fn input_schema(
                filter: #facade_crate::mcp::McpTableFilter
            ) -> #facade_crate::mcp::serde_json::Value {
                #facade_crate::mcp::default_filter_shape_input_schema::<Self>(filter)
            }

            fn decode_filter(
                field: &'static str,
                value: #facade_crate::mcp::serde_json::Value,
            ) -> Result<
                <Self as #facade_crate::runtime::shape::GpuiTableFilterShape>::FilterValue,
                #facade_crate::mcp::McpToolError,
            > {
                #facade_crate::mcp::decode_raw_filter_shape::<Self>(field, value)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    #[test]
    fn derive_uses_raw_value_trait_defaults() {
        let input: DeriveInput = syn::parse_quote! {
            struct LocalTextFilter;
        };

        let expanded = expand(input.to_token_stream())
            .expect("derive should expand")
            .to_string();

        assert!(expanded.contains("McpFilterShape"));
        assert!(expanded.contains("DeserializeOwned"));
        assert!(expanded.contains("McpJsonSchema"));
        assert!(expanded.contains("default_filter_shape_input_schema"));
        assert!(expanded.contains("decode_raw_filter_shape"));
    }
}
