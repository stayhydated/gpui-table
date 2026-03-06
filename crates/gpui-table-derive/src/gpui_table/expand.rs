use crate::__crate_paths::gpui::{AnyElement, App, IntoElement, Window};
use crate::__crate_paths::gpui_component::table::{Column, ColumnFixed, ColumnSort};
use crate::gpui_table::delegate::generate_delegate;
#[cfg(feature = "inventory")]
use crate::gpui_table::filter_codegen::get_registry_filter_type;
use crate::gpui_table::filter_codegen::{
    generate_filter_feature_assertions, get_filter_type_expr, validate_filter_config,
};
use crate::gpui_table::filter_entities::{
    generate_filter_entities, generate_matches_filters_method,
};
use crate::gpui_table::meta::{FilterFieldMeta, TableMeta};

use darling::util::Override;
use heck::{ToPascalCase as _, ToTitleCase as _};
use quote::quote;
use syn::Ident;

pub(super) fn expand_gpui_table(meta: TableMeta) -> syn::Result<proc_macro2::TokenStream> {
    let TableMeta {
        ident: struct_name,
        data,
        id,
        title,
        delegate,
        custom_style,
        fluent,
        loading,
        load_more,
        filters: filters_enabled,
    } = meta;

    let table_id = id.unwrap_or_else(|| struct_name.to_string());
    let table_title = title.unwrap_or_else(|| struct_name.to_string());

    let custom_style = match custom_style {
        Some(Override::Explicit(val)) => val,
        Some(Override::Inherit) => true,
        None => false,
    };

    let fields = data.take_struct().unwrap();

    let mut columns_init = Vec::new();
    let mut cell_value_match_arms = Vec::new();
    let mut sort_match_arms = Vec::new();
    let mut column_variants = Vec::new();
    let mut from_usize_arms = Vec::new();
    let mut into_usize_arms = Vec::new();
    let mut filters_init = Vec::new();
    let mut filter_fields: Vec<FilterFieldMeta> = Vec::new();

    #[cfg(feature = "inventory")]
    let mut column_variant_constructions: Vec<proc_macro2::TokenStream> = Vec::new();
    #[cfg(feature = "inventory")]
    let mut filter_variant_constructions: Vec<proc_macro2::TokenStream> = Vec::new();

    let column_enum_name = Ident::new(&format!("{}TableColumn", struct_name), struct_name.span());

    let active_fields: Vec<_> = fields.into_iter().filter(|f| !f.skip).enumerate().collect();

    for (i, field) in active_fields {
        let ident = field.ident.as_ref().unwrap();
        let key = field.col.unwrap_or_else(|| ident.to_string());
        let width = field.width.unwrap_or(100.0);

        if field.ascending && field.descending {
            return Err(syn::Error::new(
                ident.span(),
                "`ascending` and `descending` cannot both be set",
            ));
        }
        if !filters_enabled && field.filter.is_some() {
            return Err(syn::Error::new(
                ident.span(),
                "field-level `filter(...)` requires struct-level `#[gpui_table(filters)]`",
            ));
        }
        if let Some(fixed) = field.fixed.as_deref()
            && !matches!(fixed, "left" | "right")
        {
            return Err(syn::Error::new(
                ident.span(),
                format!("invalid `fixed` value `{fixed}`; expected \"left\" or \"right\""),
            ));
        }

        let title_expr = determine_title_expr(&field.title, ident, &fluent, &struct_name);

        let sortable_chain = if field.descending {
            quote! { .descending() }
        } else if field.ascending {
            quote! { .ascending() }
        } else if field.sortable {
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
        let resizable_chain = match field.resizable {
            Some(val) => quote! { .resizable(#val) },
            None => quote! {},
        };
        let movable_chain = match field.movable {
            Some(val) => quote! { .movable(#val) },
            None => quote! {},
        };

        columns_init.push(quote! {
            #Column::new(#key, #title_expr)
                .width(#width)
                #sortable_chain
                #text_right_chain
                #fixed_chain
                #resizable_chain
                #movable_chain
        });

        cell_value_match_arms.push(quote! {
            #i => Box::new(self.#ident.clone()),
        });

        // Only process filter attributes when filters are enabled at struct level
        if filters_enabled && let Some(ref filter_config) = field.filter {
            validate_filter_config(filter_config, ident)?;
            let filter_type_ts = get_filter_type_expr(filter_config, &field.ty);

            filters_init.push(quote! {
                gpui_table::filter::FilterConfig {
                    column_index: #i,
                    filter_type: #filter_type_ts,
                }
            });

            // Collect filter field metadata for delegate generation
            filter_fields.push(FilterFieldMeta {
                field_ident: ident.clone(),
                filter_config: filter_config.clone(),
                field_type: field.ty.clone(),
            });

            #[cfg(feature = "inventory")]
            {
                let field_name_str = ident.to_string();
                let registry_filter_type = get_registry_filter_type(filter_config);

                filter_variant_constructions.push(quote! {
                    gpui_table::registry::FilterVariant::new(
                        #field_name_str,
                        #registry_filter_type,
                    )
                });
            }
        }

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

        #[cfg(feature = "inventory")]
        {
            use quote::ToTokens as _;
            let field_name_str = ident.to_string();
            let field_type_str = field.ty.to_token_stream().to_string();
            let title_str = field
                .title
                .clone()
                .unwrap_or_else(|| ident.to_string().to_title_case());
            let fixed_variant = match field.fixed.as_deref() {
                Some("left") => quote! { gpui_table::registry::ColumnFixed::Left },
                Some("right") => quote! { gpui_table::registry::ColumnFixed::Right },
                _ => quote! { gpui_table::registry::ColumnFixed::None },
            };
            let sortable = field.sortable;
            column_variant_constructions.push(quote! {
                gpui_table::registry::ColumnVariant::new(
                    #field_name_str,
                    #field_type_str,
                    #title_str,
                    #width,
                    #sortable,
                    #fixed_variant,
                )
            });
        }
    }

    let filter_feature_assertions =
        generate_filter_feature_assertions(&struct_name, &filter_fields);

    let table_title_impl = match &fluent {
        Some(Override::Explicit(key)) => {
            let key_cap = key.to_pascal_case();
            let fluent_enum = Ident::new(
                &format!("{}{}{}Variants", struct_name, key_cap, ""),
                struct_name.span(),
            );
            quote! { fn table_title() -> String {
              use es_fluent::ThisFtl as _;
              #fluent_enum::this_ftl()
              }
            }
        },
        Some(Override::Inherit) => {
            let fluent_enum = Ident::new(&format!("{}", struct_name), struct_name.span());
            quote! { fn table_title() -> String {
              use es_fluent::ThisFtl as _;
              #fluent_enum::this_ftl()
              }
            }
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

    let delegate_impl = if delegate {
        generate_delegate(
            &struct_name,
            &column_enum_name,
            sort_match_arms,
            loading,
            load_more,
            &filter_fields,
        )
    } else {
        quote! {}
    };

    // Generate FilterEntities struct for UI components (only when filters enabled)
    let filter_entities_impl = generate_filter_entities(&struct_name, &filter_fields, &fluent);

    // Generate matches_filters() method on the struct (only when filters enabled)
    let matches_filters_impl = generate_matches_filters_method(&struct_name, &filter_fields);

    #[cfg(feature = "inventory")]
    let shape_impl = {
        quote! {
            gpui_table::registry::inventory::submit! {
                gpui_table::registry::GpuiTableShape::new(
                    stringify!(#struct_name),
                    #table_id,
                    #table_title,
                    &[
                        #(#column_variant_constructions),*
                    ],
                    &[
                        #(#filter_variant_constructions),*
                    ],
                    #load_more,
                    file!()
                )
            }
        }
    };

    #[cfg(not(feature = "inventory"))]
    let shape_impl = quote! {};

    Ok(quote! {
        #filter_feature_assertions
        #column_enum

        impl gpui_table::TableRowMeta for #struct_name {
            const TABLE_ID: &'static str = #table_id;
            const TABLE_TITLE: &'static str = #table_title;

            #table_title_impl

            fn table_columns() -> Vec<#Column> {
                vec![
                    #(#columns_init),*
                ]
            }

            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    #(#cell_value_match_arms)*
                    _ => Box::new(String::new()),
                }
            }

            fn table_filters() -> Vec<gpui_table::filter::FilterConfig> {
                vec![
                    #(#filters_init),*
                ]
            }
        }

        #shape_impl
        #style_impl
        #delegate_impl
        #filter_entities_impl
        #matches_filters_impl
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
                    &format!("{}{}{}Variants", struct_name, key_cap, ""),
                    struct_name.span(),
                )
            },
            Override::Inherit => {
                Ident::new(&format!("{}Variants", struct_name), struct_name.span())
            },
        };

        let field_name = ident.to_string().to_pascal_case();
        let fluent_variant_ident = Ident::new(&field_name, ident.span());

        quote! { { use es_fluent::ToFluentString as _; #fluent_enum_ident::#fluent_variant_ident.to_fluent_string() } }
    } else {
        let raw_title = ident.to_string().to_title_case();
        quote! { #raw_title }
    }
}
