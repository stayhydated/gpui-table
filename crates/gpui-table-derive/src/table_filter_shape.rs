use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};
use std::collections::HashSet;
use syn::{
    Data, DeriveInput, Expr, Generics, Ident, Path, Result, Token, Type, parse::ParseStream,
    parse_quote, punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(base);
    syn::custom_keyword!(field);
    syn::custom_keyword!(fields);
    syn::custom_keyword!(from_base);
    syn::custom_keyword!(into_base);
    syn::custom_keyword!(koruma_newtype);
    syn::custom_keyword!(raw_value);
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let shape = TableFilterShapeDerive::from_input(input)?;
    Ok(shape.expand())
}

struct TableFilterShapeDerive {
    ident: Ident,
    generics: Generics,
    base: Type,
    raw_value: Option<Type>,
    fields: Vec<Type>,
    into_base: Expr,
    from_base: Expr,
    koruma_newtype: bool,
}

impl TableFilterShapeDerive {
    fn from_input(input: DeriveInput) -> Result<Self> {
        let DeriveInput {
            ident,
            generics,
            attrs,
            data,
            ..
        } = input;

        if !matches!(data, Data::Struct(_)) {
            return Err(syn::Error::new_spanned(
                ident,
                "`GpuiTableFilterShape` can only be derived for structs",
            ));
        }

        let mut options = TableFilterShapeOptions::default();
        for attr in attrs
            .iter()
            .filter(|attr| attr.path().is_ident("gpui_table_filter_shape"))
        {
            attr.parse_args_with(|input: ParseStream<'_>| {
                options.parse(input)?;
                Ok(())
            })?;
        }

        let base = options.base.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                "missing `#[gpui_table_filter_shape(base = ...)]` option",
            )
        })?;
        if options.fields.is_empty() {
            return Err(syn::Error::new(
                ident.span(),
                "`GpuiTableFilterShape` requires `field = ...` or `fields(...)` metadata",
            ));
        }
        validate_unique_fields(&options.fields)?;

        Ok(Self {
            ident,
            generics,
            base,
            raw_value: options.raw_value,
            fields: options.fields,
            into_base: options
                .into_base
                .unwrap_or_else(|| parse_quote!(::core::convert::identity)),
            from_base: options
                .from_base
                .unwrap_or_else(|| parse_quote!(::core::convert::identity)),
            koruma_newtype: options.koruma_newtype,
        })
    }

    fn expand(self) -> TokenStream {
        let Self {
            ident,
            generics,
            base,
            raw_value,
            fields,
            into_base,
            from_base,
            koruma_newtype,
        } = self;

        let facade_crate = resolve_crate_path("gpui-table", "::gpui_table");
        let raw_value = raw_value
            .map(|raw_value| quote! { #raw_value })
            .unwrap_or_else(|| {
                quote! {
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue
                }
            });
        let field_support_impls = fields.iter().map(|field| {
            field_support_impl_tokens(
                &facade_crate,
                &ident,
                &generics,
                &base,
                field,
                koruma_newtype,
            )
        });
        let mcp_impl = mcp_impl_tokens(&facade_crate, &ident, &generics);
        let mcp_koruma_newtype_impls = if koruma_newtype {
            fields
                .iter()
                .map(|field| {
                    mcp_koruma_newtype_impl_tokens(&facade_crate, &ident, &generics, field)
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_generics #facade_crate::runtime::shape::ComponentShapeMetadata
                for #ident #ty_generics
                #where_clause
            {
                const MCP_INPUT: #facade_crate::runtime::shape::McpInput =
                    <#base as #facade_crate::runtime::shape::ComponentShapeMetadata>::MCP_INPUT;
            }

            impl #impl_generics #facade_crate::runtime::shape::DeclaredComponentShape
                for #ident #ty_generics
                #where_clause
            {
            }

            impl #impl_generics #facade_crate::runtime::shape::GpuiTableFilterShape
                for #ident #ty_generics
                #where_clause
            {
                type Component =
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::Component;
                type RawValue = #raw_value;
                type FilterValue =
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::FilterValue;

                const FILTER_TYPE: #facade_crate::schema::registry::RegistryFilterType =
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::FILTER_TYPE;

                fn new_for(
                    title: impl Fn(&::gpui::App) -> String + 'static,
                    value: Self::RawValue,
                    on_change: impl Fn(Self::RawValue, &mut ::gpui::Window, &mut ::gpui::App) + 'static,
                    cx: &mut ::gpui::App,
                ) -> ::gpui::Entity<Self::Component> {
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::new_for(
                        title,
                        (#into_base)(value),
                        move |__gpui_table_base_value, window, cx| {
                            on_change((#from_base)(__gpui_table_base_value), window, cx);
                        },
                        cx,
                    )
                }

                fn read_value(
                    entity: &::gpui::Entity<Self::Component>,
                    cx: &::gpui::App,
                ) -> Self::RawValue {
                    (#from_base)(
                        <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::read_value(
                            entity,
                            cx,
                        )
                    )
                }

                fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::wrap_value(
                        (#into_base)(value),
                    )
                }

                fn reset_silent(
                    entity: &::gpui::Entity<Self::Component>,
                    window: &mut ::gpui::Window,
                    cx: &mut ::gpui::App,
                ) {
                    <#base as #facade_crate::runtime::shape::GpuiTableFilterShape>::reset_silent(
                        entity,
                        window,
                        cx,
                    );
                }
            }

            impl #impl_generics #facade_crate::runtime::shape::DeclaredGpuiTableFilterShape
                for #ident #ty_generics
                #where_clause
            {
            }

            #(#field_support_impls)*

            #mcp_impl

            #(#mcp_koruma_newtype_impls)*
        }
    }
}

fn field_support_impl_tokens(
    facade_crate: &Path,
    ident: &Ident,
    generics: &Generics,
    base: &Type,
    field: &Type,
    koruma_newtype: bool,
) -> TokenStream {
    let mut generics = generics.clone();
    let matched_field = if koruma_newtype {
        generics.make_where_clause().predicates.push(parse_quote! {
            #field: ::koruma::NewtypeValue
        });
        generics.make_where_clause().predicates.push(parse_quote! {
            #base: #facade_crate::runtime::shape::GpuiTableFilterShapeFor<
                <#field as ::koruma::NewtypeValue>::Inner
            >
        });
        quote! { <#field as ::koruma::NewtypeValue>::as_inner(field) }
    } else {
        generics.make_where_clause().predicates.push(parse_quote! {
            #base: #facade_crate::runtime::shape::GpuiTableFilterShapeFor<#field>
        });
        quote! { field }
    };
    let delegated_field = if koruma_newtype {
        quote! { <#field as ::koruma::NewtypeValue>::Inner }
    } else {
        quote! { #field }
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #facade_crate::runtime::shape::GpuiTableFilterShapeFor<#field>
            for #ident #ty_generics
            #where_clause
        {
            fn filter_type() -> #facade_crate::core::filter::FilterType {
                <#base as #facade_crate::runtime::shape::GpuiTableFilterShapeFor<
                    #delegated_field
                >>::filter_type()
            }

            fn matches_field(field: &#field, value: &Self::FilterValue) -> bool {
                <#base as #facade_crate::runtime::shape::GpuiTableFilterShapeFor<
                    #delegated_field
                >>::matches_field(
                    #matched_field,
                    value,
                )
            }
        }
    }
}

fn mcp_koruma_newtype_impl_tokens(
    facade_crate: &Path,
    ident: &Ident,
    generics: &Generics,
    field: &Type,
) -> TokenStream {
    if !cfg!(feature = "mcp") {
        return quote! {};
    }

    let mut generics = generics.clone();
    let self_type = {
        let (_, ty_generics, _) = generics.split_for_impl();
        quote! { #ident #ty_generics }
    };
    generics.make_where_clause().predicates.push(parse_quote! {
        #field: ::koruma::NewtypeValue<
            Inner = <#self_type as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue
        >
    });
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #facade_crate::mcp::McpKorumaNewtypeFilterValidation<#field>
            for #ident #ty_generics
            #where_clause
        {
            fn validate_koruma_newtype_filter(
                value: &<Self as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue
            ) -> bool {
                <#field as ::koruma::NewtypeValue>::validate_inner(value).is_ok()
            }
        }
    }
}

fn mcp_impl_tokens(facade_crate: &Path, ident: &Ident, generics: &Generics) -> TokenStream {
    if !cfg!(feature = "mcp") {
        return quote! {};
    }

    let mut generics = generics.clone();
    let self_type = {
        let (_, ty_generics, _) = generics.split_for_impl();
        quote! { #ident #ty_generics }
    };
    generics.make_where_clause().predicates.push(parse_quote! {
        <#self_type as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue:
            #facade_crate::mcp::McpToolValue
    });
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #facade_crate::mcp::McpFilterShape for #ident #ty_generics
            #where_clause
        {
            fn input_schema(
                filter: #facade_crate::mcp::McpTableFilter
            ) -> #facade_crate::mcp::McpSchema {
                #facade_crate::mcp::default_filter_shape_input_schema::<Self>(filter)
            }

            fn decode_filter(
                field: &'static str,
                value: #facade_crate::mcp::McpAny,
            ) -> Result<
                <Self as #facade_crate::runtime::shape::GpuiTableFilterShape>::FilterValue,
                #facade_crate::mcp::McpToolError,
            > {
                #facade_crate::mcp::decode_raw_filter_shape::<Self>(field, value)
            }
        }

        impl #impl_generics #facade_crate::mcp::McpFilterShapeValidation for #ident #ty_generics
            #where_clause
        {
            fn decode_filter_with_validation<Validate>(
                field: &'static str,
                value: #facade_crate::mcp::McpAny,
                validate: Validate,
            ) -> Result<
                <Self as #facade_crate::runtime::shape::GpuiTableFilterShape>::FilterValue,
                #facade_crate::mcp::McpToolError,
            >
            where
                Validate: FnOnce(
                    &<Self as #facade_crate::runtime::shape::GpuiTableFilterShape>::RawValue
                ) -> Result<(), #facade_crate::mcp::McpToolError>,
            {
                #facade_crate::mcp::decode_raw_filter_shape_with_validation::<Self, _>(
                    field,
                    value,
                    validate,
                )
            }
        }
    }
}

#[derive(Default)]
struct TableFilterShapeOptions {
    base: Option<Type>,
    raw_value: Option<Type>,
    fields: Vec<Type>,
    into_base: Option<Expr>,
    from_base: Option<Expr>,
    koruma_newtype: bool,
}

impl TableFilterShapeOptions {
    fn parse(&mut self, input: ParseStream<'_>) -> Result<()> {
        while !input.is_empty() {
            if input.peek(kw::base) {
                let key = input.parse::<kw::base>()?;
                input.parse::<Token![=]>()?;
                set_once(&mut self.base, input.parse()?, key, "base")?;
            } else if input.peek(kw::raw_value) {
                let key = input.parse::<kw::raw_value>()?;
                input.parse::<Token![=]>()?;
                set_once(&mut self.raw_value, input.parse()?, key, "raw_value")?;
            } else if input.peek(kw::field) {
                input.parse::<kw::field>()?;
                input.parse::<Token![=]>()?;
                self.fields.push(input.parse()?);
            } else if input.peek(kw::fields) {
                let key = input.parse::<kw::fields>()?;
                let content;
                syn::parenthesized!(content in input);
                let fields = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                if fields.is_empty() {
                    return Err(syn::Error::new_spanned(
                        key,
                        "`fields(...)` requires at least one field type",
                    ));
                }
                self.fields.extend(fields);
            } else if input.peek(kw::into_base) {
                let key = input.parse::<kw::into_base>()?;
                input.parse::<Token![=]>()?;
                set_once(&mut self.into_base, input.parse()?, key, "into_base")?;
            } else if input.peek(kw::from_base) {
                let key = input.parse::<kw::from_base>()?;
                input.parse::<Token![=]>()?;
                set_once(&mut self.from_base, input.parse()?, key, "from_base")?;
            } else if input.peek(kw::koruma_newtype) {
                let key = input.parse::<kw::koruma_newtype>()?;
                if self.koruma_newtype {
                    return Err(syn::Error::new_spanned(
                        key,
                        "duplicate `koruma_newtype` option",
                    ));
                }
                self.koruma_newtype = true;
            } else {
                return Err(input.error(
                    "expected `base = ...`, `raw_value = ...`, `field = ...`, `fields(...)`, `into_base = ...`, `from_base = ...`, or `koruma_newtype`",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(())
    }
}

fn set_once<T, S: quote::ToTokens>(
    slot: &mut Option<T>,
    value: T,
    span: S,
    option: &str,
) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(syn::Error::new_spanned(
            span,
            format!("duplicate `{option}` option"),
        ));
    }

    Ok(())
}

fn validate_unique_fields(fields: &[Type]) -> Result<()> {
    let mut seen = HashSet::new();
    for field in fields {
        if !seen.insert(field.to_token_stream().to_string()) {
            return Err(syn::Error::new_spanned(
                field,
                "duplicate `GpuiTableFilterShape` field type",
            ));
        }
    }

    Ok(())
}

fn resolve_crate_path(package_name: &str, fallback: &str) -> Path {
    let path = match crate_name(package_name) {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(name)) => format!("::{name}"),
        Err(_) => fallback.to_string(),
    };

    syn::parse_str(&path).expect("crate path resolver produced a valid Rust path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_emits_runtime_shape_and_field_support() {
        let input: DeriveInput = parse_quote! {
            #[gpui_table_filter_shape(
                base = gpui_table_component::TextFilter,
                raw_value = PrefixText,
                fields(String, Option<String>),
                into_base = |value: PrefixText| value.0,
                from_base = PrefixText
            )]
            struct PrefixTextFilter;
        };

        let expanded = TableFilterShapeDerive::from_input(input)
            .expect("derive input should parse")
            .expand()
            .to_string();

        assert!(expanded.contains("GpuiTableFilterShape"));
        assert!(expanded.contains("DeclaredGpuiTableFilterShape"));
        assert!(expanded.contains("GpuiTableFilterShapeFor < String >"));
        assert!(expanded.contains("GpuiTableFilterShapeFor < Option < String > >"));
        assert!(expanded.contains("TextFilter"));
        assert!(expanded.contains("PrefixText"));
    }

    #[test]
    fn derive_emits_koruma_newtype_field_support() {
        let input: DeriveInput = parse_quote! {
            #[gpui_table_filter_shape(
                base = gpui_table_component::TextFilter,
                field = Email,
                koruma_newtype
            )]
            struct EmailTextFilter;
        };

        let expanded = TableFilterShapeDerive::from_input(input)
            .expect("derive input should parse")
            .expand()
            .to_string();

        assert!(expanded.contains("Email : :: koruma :: NewtypeValue"));
        assert!(expanded.contains("< Email as :: koruma :: NewtypeValue > :: Inner"));
        #[cfg(feature = "mcp")]
        assert!(expanded.contains("validate_inner"));
    }

    #[test]
    fn base_option_is_required() {
        let input: DeriveInput = parse_quote! {
            #[gpui_table_filter_shape(field = String)]
            struct PrefixTextFilter;
        };

        let error = match TableFilterShapeDerive::from_input(input) {
            Ok(_) => panic!("missing base should fail"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("missing `#[gpui_table_filter_shape(base = ...)]` option")
        );
    }

    #[test]
    fn fields_requires_values() {
        let input: DeriveInput = parse_quote! {
            #[gpui_table_filter_shape(
                base = gpui_table_component::TextFilter,
                fields()
            )]
            struct PrefixTextFilter;
        };

        let error = match TableFilterShapeDerive::from_input(input) {
            Ok(_) => panic!("fields should fail"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("requires at least one field type")
        );
    }

    #[test]
    fn duplicate_fields_are_rejected() {
        let input: DeriveInput = parse_quote! {
            #[gpui_table_filter_shape(
                base = gpui_table_component::TextFilter,
                field = String,
                fields(String)
            )]
            struct PrefixTextFilter;
        };

        let error = match TableFilterShapeDerive::from_input(input) {
            Ok(_) => panic!("duplicate fields should fail"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("duplicate `GpuiTableFilterShape` field type")
        );
    }
}
