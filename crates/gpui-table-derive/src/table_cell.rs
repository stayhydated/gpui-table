use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, DeriveInput, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub(crate) fn derive_table_cell(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match expand_derive_table_cell(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive_table_cell(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let use_fluent_for_unit_variants =
        cfg!(feature = "fluent") && has_derive_named(&input, "EsFluent");
    let use_display_for_unit_variants = has_derive_named(&input, "Display");
    let options = TableCellOptions::parse_attrs(&input.attrs)?;
    let name = input.ident;

    let draw_impl = if let Some(render_override) = options.render_override() {
        render_override.draw_tokens()
    } else {
        match input.data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    quote! { self.0.draw(window, cx) }
                },
                syn::Fields::Named(fields) if fields.named.len() == 1 => {
                    let field_name = fields
                        .named
                        .first()
                        .and_then(|field| field.ident.clone())
                        .ok_or_else(|| {
                            syn::Error::new(
                                name.span(),
                                "TableCell derive could not resolve the single named field",
                            )
                        })?;
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
                            let f_ident = fields
                                .named
                                .first()
                                .and_then(|field| field.ident.clone())
                                .ok_or_else(|| {
                                    syn::Error::new(
                                        v_ident.span(),
                                        "TableCell derive could not resolve the single named enum field",
                                    )
                                })?;
                            Ok(quote! { Self::#v_ident { #f_ident: val } => val.draw(window, cx), })
                        }
                        syn::Fields::Unit => {
                            let render_unit_variant = if use_fluent_for_unit_variants {
                                quote! {
                                    gpui_table::core::i18n::localize_message(self)
                                        .into_any_element()
                                }
                            } else if use_display_for_unit_variants {
                                quote! { self.to_string().into_any_element() }
                            } else {
                                let variant_name = v_ident.to_string();
                                quote! { #variant_name.into_any_element() }
                            };
                            Ok(quote! { Self::#v_ident => #render_unit_variant, })
                        }
                        _ => Err(syn::Error::new(
                            v_ident.span(),
                            "TableCell derive for enum variant requires exactly one field or be a unit variant",
                        )),
                    }
                })
                .collect::<syn::Result<Vec<_>>>()?;

                quote! {
                    use ::gpui::IntoElement;
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
        }
    };

    Ok(quote! {
        impl gpui_table::runtime::TableCell for #name {
            fn draw(
                &self,
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::App
            ) -> ::gpui::AnyElement {
                #draw_impl
            }
        }
    })
}

#[derive(Clone)]
enum TableCellRender {
    Display,
    Format(Path),
}

impl TableCellRender {
    fn draw_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Display => quote! {
                use ::gpui::IntoElement;
                self.to_string().into_any_element()
            },
            Self::Format(formatter) => quote! {
                use ::gpui::IntoElement;
                #formatter(self).to_string().into_any_element()
            },
        }
    }
}

#[derive(Default)]
struct TableCellOptions {
    render: Option<TableCellRender>,
}

impl TableCellOptions {
    fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut options = Self::default();

        for attr in attrs
            .iter()
            .filter(|attr| attr.path().is_ident("table_cell"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("display") {
                    options.set_render(TableCellRender::Display, &meta.path)
                } else if meta.path.is_ident("format") {
                    let value = meta.value()?;
                    let formatter = value.parse::<PathValue>()?.0;
                    options.set_render(TableCellRender::Format(formatter), &meta.path)
                } else {
                    Err(meta.error("unsupported table_cell option"))
                }
            })?;
        }

        Ok(options)
    }

    fn render_override(&self) -> Option<&TableCellRender> {
        self.render.as_ref()
    }

    fn set_render(&mut self, render: TableCellRender, path: &Path) -> syn::Result<()> {
        if self.render.is_some() {
            return Err(syn::Error::new_spanned(
                path,
                "`table_cell` accepts only one render mode; use either `display` or `format = ...`",
            ));
        }
        self.render = Some(render);
        Ok(())
    }
}

struct PathValue(Path);

impl Parse for PathValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            lit.parse::<Path>().map(Self)
        } else {
            input.parse::<Path>().map(Self)
        }
    }
}

fn has_derive_named(input: &DeriveInput, expected: &str) -> bool {
    input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }

        attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|path| path.segments.last())
                    .any(|segment| segment.ident == expected)
            })
            .unwrap_or(false)
    })
}
