use darling::{FromDeriveInput, FromVariant};
use heck::ToTitleCase as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Path};

pub(crate) fn derive_filterable(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand_derive_filterable(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(filter), supports(enum_any))]
struct FilterableMeta {
    ident: Ident,
    data: darling::ast::Data<FilterableVariant, darling::util::Ignored>,
    #[darling(default)]
    fluent: bool,
}

#[derive(FromVariant)]
#[darling(attributes(filter))]
struct FilterableVariant {
    ident: Ident,
    #[darling(default)]
    label: Option<String>,
    /// Icon component path (e.g., `IconName::Check`).
    #[darling(default)]
    icon: Option<Path>,
}

fn expand_derive_filterable(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let FilterableMeta {
        ident: enum_name,
        data,
        fluent,
    } = FilterableMeta::from_derive_input(&input)?;
    let variants = data.take_enum().ok_or_else(|| {
        syn::Error::new(enum_name.span(), "Filterable can only be derived for enums")
    })?;
    if fluent && !cfg!(feature = "fluent") {
        return Err(syn::Error::new(
            enum_name.span(),
            "`#[filter(fluent)]` requires enabling the `gpui-table/fluent` feature",
        ));
    }

    let mut options = Vec::new();
    let mut variant_name_arms = Vec::new();
    let mut from_filter_string_arms = Vec::new();

    for variant in &variants {
        let variant_ident = &variant.ident;
        let value = variant_ident.to_string(); // Or snake_case? Using variant name for now.

        let label_expr = if fluent {
            quote! {
                gpui_table::core::i18n::localize_message(&Self::#variant_ident)
            }
        } else {
            let label = variant
                .label
                .clone()
                .unwrap_or_else(|| value.clone().to_title_case());
            quote! { #label.to_string() }
        };

        let icon = match &variant.icon {
            Some(path) => {
                quote! { Some(gpui_table::runtime::generated_filters::icon_from_name(#path)) }
            },
            None => quote! { None },
        };

        options.push(quote! {
            gpui_table::core::filter::FacetedFilterOption {
                group: None,
                label: #label_expr,
                value: #value.to_string(),
                count: None,
                icon: #icon,
            }
        });

        // Generate variant_name match arm (to_filter_string)
        variant_name_arms.push(quote! {
            Self::#variant_ident => #value,
        });

        // Generate from_filter_string match arm
        from_filter_string_arms.push(quote! {
            #value => Some(Self::#variant_ident),
        });
    }

    Ok(quote! {
        impl gpui_table::core::filter::FilterValue for #enum_name {
            fn to_filter_string(&self) -> String {
                match self {
                    #(#variant_name_arms)*
                }.to_string()
            }

            fn from_filter_string(s: &str) -> Option<Self> {
                match s {
                    #(#from_filter_string_arms)*
                    _ => None,
                }
            }
        }

        impl gpui_table::core::filter::Filterable for #enum_name {
            fn options() -> Vec<gpui_table::core::filter::FacetedFilterOption> {
                vec![
                    #(#options),*
                ]
            }
        }

        impl #enum_name {
            /// Returns the variant name as a static string.
            /// Useful for matching against filter values in client-side filtering.
            pub fn variant_name(&self) -> &'static str {
                match self {
                    #(#variant_name_arms)*
                }
            }
        }
    })
}
