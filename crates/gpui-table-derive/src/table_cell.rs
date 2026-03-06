use crate::__crate_paths::gpui::{AnyElement, App, IntoElement, Window};

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn derive_table_cell(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match expand_derive_table_cell(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive_table_cell(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;

    let draw_impl = match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                quote! { self.0.draw(window, cx) }
            },
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                let field_name = &fields.named.first().unwrap().ident;
                quote! { self.#field_name.draw(window, cx) }
            },
            _ => {
                return Err(syn::Error::new(
                    name.span(),
                    "TableCell derive for struct requires exactly one field",
                ));
            },
        },
        syn::Data::Enum(data) => {
            let arms = data
                .variants
                .iter()
                .map(|v| {
                    let v_ident = &v.ident;
                    match &v.fields {
                        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                            Ok(quote! { Self::#v_ident(val) => val.draw(window, cx), })
                        }
                        syn::Fields::Named(fields) if fields.named.len() == 1 => {
                            let f_ident = &fields.named.first().unwrap().ident;
                            Ok(quote! { Self::#v_ident { #f_ident: val } => val.draw(window, cx), })
                        }
                        syn::Fields::Unit => {
                            Ok(quote! { Self::#v_ident => self.to_fluent_string().into_any_element(), })
                        }
                        _ => Err(syn::Error::new(
                            v_ident.span(),
                            "TableCell derive for enum variant requires exactly one field or be a unit variant",
                        )),
                    }
                })
                .collect::<syn::Result<Vec<_>>>()?;

            quote! {
                use #IntoElement;
                use es_fluent::ToFluentString as _;
                match self {
                    #(#arms)*
                }
            }
        },
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                name.span(),
                "TableCell cannot be derived for unions",
            ));
        },
    };

    Ok(quote! {
        impl gpui_table::TableCell for #name {
            fn draw(
                &self,
                window: &mut #Window,
                cx: &mut #App
            ) -> #AnyElement {
                #draw_impl
            }
        }
    })
}
