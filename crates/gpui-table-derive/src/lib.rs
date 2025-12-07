#[doc(hidden)]
mod __crate_paths;

use __crate_paths::gpui::{AnyElement, App, Context, IntoElement, Window};
use __crate_paths::gpui_component::table::{
    Column, ColumnFixed, ColumnSort, TableDelegate, TableState,
};

use darling::{FromDeriveInput, FromField, util::Override};
use heck::{ToPascalCase as _, ToTitleCase as _};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

#[proc_macro_derive(NamedTableRow, attributes(table))]
pub fn derive_named_table_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match TableMeta::from_derive_input(&input) {
        Ok(meta) => match expand_named_table_row(meta) {
            Ok(ts) => ts.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(table), supports(struct_named))]
struct TableMeta {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, TableColumn>,

    #[darling(default)]
    id: Option<String>,
    #[darling(default)]
    title: Option<String>,

    #[darling(default = "default_delegate")]
    delegate: bool,

    #[darling(default)]
    custom_style: Option<Override<bool>>,

    #[darling(default)]
    fluent: Option<Override<String>>,
}

fn default_delegate() -> bool {
    true
}

#[derive(FromField)]
#[darling(attributes(table))]
struct TableColumn {
    ident: Option<Ident>,

    #[darling(default)]
    col: Option<String>,
    #[darling(default)]
    title: Option<String>,
    #[darling(default)]
    width: Option<f32>,
    #[darling(default)]
    fixed: Option<String>,
    #[darling(default)]
    sortable: bool,
    #[darling(default)]
    text_right: bool,
    #[darling(default)]
    skip: bool,
}

fn expand_named_table_row(meta: TableMeta) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &meta.ident;

    let table_id = meta.id.unwrap_or_else(|| struct_name.to_string());
    let table_title = meta.title.unwrap_or_else(|| struct_name.to_string());

    let custom_style = match meta.custom_style {
        Some(Override::Explicit(val)) => val,
        Some(Override::Inherit) => true,
        None => false,
    };

    let fields = meta.data.take_struct().unwrap();

    let mut columns_init = Vec::new();
    let mut cell_value_match_arms = Vec::new();
    let mut sort_match_arms = Vec::new();
    let mut column_variants = Vec::new();
    let mut from_usize_arms = Vec::new();
    let mut into_usize_arms = Vec::new();

    let column_enum_name = Ident::new(&format!("{}TableColumn", struct_name), struct_name.span());

    let active_fields: Vec<_> = fields.into_iter().filter(|f| !f.skip).enumerate().collect();

    for (i, field) in active_fields {
        let ident = field.ident.as_ref().unwrap();
        let key = field.col.unwrap_or_else(|| ident.to_string());
        let width = field.width.unwrap_or(100.0);

        let title_expr = determine_title_expr(&field.title, ident, &meta.fluent, struct_name);

        let sortable_chain = if field.sortable {
            quote! { .sortable() }
        } else {
            quote! {}
        };
        let text_right_chain = if field.text_right {
            quote! { .text_right() }
        } else {
            quote! {}
        };

        let fixed_chain = match field.fixed.as_deref() {
            Some("left") => quote! { .fixed(#ColumnFixed::Left) },
            Some("right") => quote! { .fixed(#ColumnFixed::Right) },
            _ => quote! {},
        };

        columns_init.push(quote! {
            #Column::new(#key, #title_expr)
                .width(#width)
                #sortable_chain
                #text_right_chain
                #fixed_chain
        });

        cell_value_match_arms.push(quote! {
            #i => Box::new(self.#ident.clone()),
        });

        if field.sortable {
            sort_match_arms.push(quote! {
                #i => {
                    self.rows.sort_by(|a, b| {
                        let a_val = &a.#ident;
                        let b_val = &b.#ident;
                        match sort {
                            #ColumnSort::Ascending => a_val.partial_cmp(b_val).unwrap_or(std::cmp::Ordering::Equal),
                            #ColumnSort::Descending => b_val.partial_cmp(a_val).unwrap_or(std::cmp::Ordering::Equal),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                }
            });
        }

        let variant_name = ident.to_string().to_pascal_case();
        let variant_ident = Ident::new(&variant_name, ident.span());

        column_variants.push(quote! { #variant_ident });

        from_usize_arms.push(quote! { #i => #column_enum_name::#variant_ident, });
        into_usize_arms.push(quote! { #column_enum_name::#variant_ident => #i, });
    }

    let table_title_impl = match &meta.fluent {
        Some(Override::Explicit(key)) => {
            let key_cap = key.to_pascal_case();
            let fluent_enum = Ident::new(
                &format!("{}{}{}KvFtl", struct_name, key_cap, ""),
                struct_name.span(),
            );
            quote! { fn table_title() -> String { #fluent_enum::this_ftl() } }
        },
        Some(Override::Inherit) => {
            let fluent_enum = Ident::new(&format!("{}", struct_name), struct_name.span());
            quote! { fn table_title() -> String { #fluent_enum::this_ftl() } }
        },
        None => {
            quote! { fn table_title() -> String { Self::TABLE_TITLE.to_string() } }
        },
    };

    let column_enum = quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum #column_enum_name {
            #(#column_variants),*
        }

        impl From<usize> for #column_enum_name {
            fn from(ix: usize) -> Self {
                match ix {
                    #(#from_usize_arms)*
                    _ => panic!("Invalid column index: {}", ix),
                }
            }
        }

        impl From<#column_enum_name> for usize {
            fn from(col: #column_enum_name) -> Self {
                match col {
                    #(#into_usize_arms)*
                }
            }
        }
    };

    let style_impl = if !custom_style {
        quote! {
            impl gpui_table::TableRowStyle for #struct_name {
                type ColumnId = #column_enum_name;

                fn render_table_cell(
                    &self,
                    col: Self::ColumnId,
                    window: &mut #Window,
                    cx: &mut #App,
                ) -> #AnyElement {
                    use #IntoElement;
                    gpui_table::default_render_cell(self, col.into(), window, cx).into_any_element()
                }
            }
        }
    } else {
        quote! {}
    };

    let delegate_impl = if meta.delegate {
        generate_delegate(struct_name, &column_enum_name, sort_match_arms)
    } else {
        quote! {}
    };

    Ok(quote! {
        #column_enum

        impl gpui_table::TableRowMeta for #struct_name {
            const TABLE_ID: &'static str = #table_id;
            const TABLE_TITLE: &'static str = #table_title;

            #table_title_impl

            fn table_columns() -> &'static [#Column] {
                static COLUMNS: std::sync::OnceLock<Vec<#Column>> = std::sync::OnceLock::new();
                COLUMNS.get_or_init(|| vec![
                    #(#columns_init),*
                ])
            }

            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    #(#cell_value_match_arms)*
                    _ => Box::new(String::new()),
                }
            }
        }

        #style_impl
        #delegate_impl
    })
}

fn determine_title_expr(
    title_attr: &Option<String>,
    ident: &Ident,
    fluent_config: &Option<Override<String>>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    if let Some(t) = title_attr {
        return quote! { #t };
    }

    if let Some(fluent) = fluent_config {
        let fluent_enum_ident = match fluent {
            Override::Explicit(key) => {
                let key_cap = key.to_pascal_case();
                Ident::new(
                    &format!("{}{}{}KvFtl", struct_name, key_cap, ""),
                    struct_name.span(),
                )
            },
            Override::Inherit => Ident::new(&format!("{}KvFtl", struct_name), struct_name.span()),
        };

        let field_name = ident.to_string().to_pascal_case();
        let fluent_variant_ident = Ident::new(&field_name, ident.span());

        quote! { #fluent_enum_ident::#fluent_variant_ident.to_string() }
    } else {
        let raw_title = ident.to_string().to_title_case();
        quote! { #raw_title }
    }
}

fn generate_delegate(
    struct_name: &Ident,
    column_enum_name: &Ident,
    sort_arms: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let delegate_name = Ident::new(&format!("{}TableDelegate", struct_name), struct_name.span());

    quote! {
        #[derive(gpui_table::derive_new::new)]
        pub struct #delegate_name {
            pub rows: Vec<#struct_name>,
            #[new(default)]
            pub visible_rows: std::ops::Range<usize>,
            #[new(default)]
            pub visible_cols: std::ops::Range<usize>,
            #[new(default)]
            pub eof: bool,
            #[new(default)]
            pub loading: bool,
            #[new(default)]
            pub full_loading: bool,
        }

        impl #TableDelegate for #delegate_name {
            fn columns_count(&self, _: &#App) -> usize {
                <#struct_name as gpui_table::TableRowMeta>::table_columns().len()
            }

            fn rows_count(&self, _: &#App) -> usize {
                self.rows.len()
            }

            fn column(&self, col_ix: usize, _: &#App) -> &#Column {
                &<#struct_name as gpui_table::TableRowMeta>::table_columns()[col_ix]
            }

            fn render_td(
                &self,
                row_ix: usize,
                col_ix: usize,
                window: &mut #Window,
                cx: &mut #App,
            ) -> impl #IntoElement {
                use gpui_table::TableRowStyle;
                self.rows[row_ix].render_table_cell(#column_enum_name::from(col_ix), window, cx)
            }

            fn visible_rows_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                self.visible_rows = visible_range;
            }

            fn visible_columns_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                self.visible_cols = visible_range;
            }

            fn is_eof(&self, _: &#App) -> bool {
                self.eof
            }

            fn loading(&self, _: &#App) -> bool {
                self.loading
            }

            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: #ColumnSort,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                match col_ix {
                    #(#sort_arms)*
                    _ => {}
                }
            }
        }
    }
}
