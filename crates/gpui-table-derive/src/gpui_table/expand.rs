use crate::gpui_table::delegate::generate_delegate;
use crate::gpui_table::filter_codegen::get_filter_type_expr;
#[cfg(feature = "inventory")]
use crate::gpui_table::filter_codegen::get_registry_filter_type;
use crate::gpui_table::filter_entities::generate_filter_entities;
use crate::gpui_table::filter_matching::generate_matches_filters_method;
#[cfg(feature = "mcp")]
use crate::gpui_table::mcp::generate_mcp_impl;
use crate::gpui_table::meta::{FilterFieldMeta, TableMeta};

use darling::util::Override;
use heck::{ToPascalCase as _, ToSnakeCase as _, ToTitleCase as _};
#[cfg(feature = "inventory")]
use quote::ToTokens as _;
use quote::quote;
use syn::{DeriveInput, Ident};

pub(super) fn expand_gpui_table(
    meta: TableMeta,
    original_input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    #[cfg(not(feature = "mcp"))]
    let _ = original_input;

    let TableMeta {
        ident: struct_name,
        data,
        id,
        title,
        delegate,
        custom_context_menu,
        context_menu_row_id,
        context_menu_route,
        context_menu_label,
        context_menu_route_fn,
        context_menu_label_fn,
        fluent,
        loading,
        load_more,
        filters: filters_enabled,
        mcp,
    } = meta;

    if let Some(mcp) = mcp.as_ref() {
        mcp.validate(struct_name.span())?;
    }
    let mcp_enabled = mcp.is_some();
    #[cfg(not(feature = "mcp"))]
    if mcp_enabled {
        return Err(syn::Error::new(
            struct_name.span(),
            "`#[gpui_table(mcp)]` requires the `gpui-table/mcp` feature",
        ));
    }
    let filters_effective = filters_enabled || mcp_enabled;

    let table_id = id.unwrap_or_else(|| struct_name.to_string().to_snake_case());
    let table_title = title.unwrap_or_else(|| struct_name.to_string());

    let custom_context_menu = match custom_context_menu {
        Some(Override::Explicit(val)) => val,
        Some(Override::Inherit) => true,
        None => false,
    };

    let fields = data.take_struct().unwrap();
    let all_field_idents: Vec<Ident> = fields
        .iter()
        .filter_map(|field| field.ident.clone())
        .collect();
    let marked_context_menu_id_fields: Vec<Ident> = fields
        .iter()
        .filter(|field| field.context_menu_id)
        .filter_map(|field| field.ident.clone())
        .collect();

    if context_menu_route.is_some() && context_menu_route_fn.is_some() {
        return Err(syn::Error::new(
            struct_name.span(),
            "`context_menu_route` and `context_menu_route_fn` cannot both be set",
        ));
    }

    if context_menu_label.is_some() && context_menu_label_fn.is_some() {
        return Err(syn::Error::new(
            struct_name.span(),
            "`context_menu_label` and `context_menu_label_fn` cannot both be set",
        ));
    }

    if marked_context_menu_id_fields.len() > 1 {
        return Err(syn::Error::new(
            struct_name.span(),
            "only one field can be marked with `#[gpui_table(context_menu_id)]`",
        ));
    }

    let marked_context_menu_id_field = marked_context_menu_id_fields.into_iter().next();

    let context_menu_value_ident = match (context_menu_row_id, marked_context_menu_id_field) {
        (Some(_), Some(_)) => {
            return Err(syn::Error::new(
                struct_name.span(),
                "`context_menu_row_id` cannot be combined with field-level `#[gpui_table(context_menu_id)]`",
            ));
        },
        (Some(row_id), None) => {
            let row_id_ident = syn::parse_str::<Ident>(&row_id).map_err(|_| {
                syn::Error::new(
                    struct_name.span(),
                    format!(
                        "`context_menu_row_id` value `{row_id}` is not a valid field identifier"
                    ),
                )
            })?;

            if !all_field_idents.contains(&row_id_ident) {
                return Err(syn::Error::new(
                    struct_name.span(),
                    format!(
                        "`context_menu_row_id` field `{row_id}` was not found on `{}`",
                        struct_name
                    ),
                ));
            }
            Some(row_id_ident)
        },
        (None, Some(field_ident)) => Some(field_ident),
        (None, None) => None,
    };

    let has_route_source = context_menu_route.is_some() || context_menu_route_fn.is_some();
    if context_menu_value_ident.is_none()
        && (has_route_source || context_menu_label.is_some() || context_menu_label_fn.is_some())
    {
        return Err(syn::Error::new(
            struct_name.span(),
            "context-menu generation requires a row-id source via `context_menu_row_id` or field `#[gpui_table(context_menu_id)]`",
        ));
    }

    if context_menu_value_ident.is_some() && !has_route_source {
        if context_menu_label.is_some() || context_menu_label_fn.is_some() {
            return Err(syn::Error::new(
                struct_name.span(),
                "`context_menu_label`/`context_menu_label_fn` requires `context_menu_route` or `context_menu_route_fn`",
            ));
        }
        return Err(syn::Error::new(
            struct_name.span(),
            "context-menu row-id source requires `context_menu_route` or `context_menu_route_fn`",
        ));
    }

    if let Some(route) = context_menu_route.as_ref()
        && !route.contains("{id}")
    {
        return Err(syn::Error::new(
            struct_name.span(),
            "`context_menu_route` must contain `{id}` placeholder",
        ));
    }

    let context_menu_link = if let Some(context_menu_value_ident) = context_menu_value_ident {
        let href_expr = if let Some(route) = context_menu_route {
            quote! { #route.replace("{id}", &context_menu_value.to_string()) }
        } else if let Some(route_fn) = context_menu_route_fn {
            quote! { (#route_fn)(context_menu_value).to_string() }
        } else {
            return Err(syn::Error::new(
                struct_name.span(),
                "internal error: context-menu link expected a route source",
            ));
        };
        let label_expr = if let Some(label_fn) = context_menu_label_fn {
            quote! { (#label_fn)(context_menu_value).to_string() }
        } else {
            let label = context_menu_label.unwrap_or_else(|| "Open".to_string());
            quote! { #label.to_string() }
        };
        Some((context_menu_value_ident, href_expr, label_expr))
    } else {
        None
    };

    let mut columns_init = Vec::new();
    let mut cell_value_match_arms = Vec::new();
    let mut sort_match_arms = Vec::new();
    let mut column_variants = Vec::new();
    let mut from_usize_arms = Vec::new();
    let mut into_usize_arms = Vec::new();
    let mut style_match_arms = Vec::new();
    let mut filters_init = Vec::new();
    let mut filter_fields: Vec<FilterFieldMeta> = Vec::new();
    let mut filter_shape_type_checks = Vec::new();

    #[cfg(feature = "inventory")]
    let mut column_variant_constructions: Vec<proc_macro2::TokenStream> = Vec::new();
    #[cfg(feature = "inventory")]
    let mut filter_variant_constructions: Vec<proc_macro2::TokenStream> = Vec::new();

    let column_enum_name = Ident::new(&format!("{}TableColumn", struct_name), struct_name.span());

    let active_fields: Vec<_> = fields.into_iter().filter(|f| !f.skip).enumerate().collect();

    for (i, field) in active_fields {
        let ident = field.ident.as_ref().unwrap();
        let style = field.style.clone();
        let key = field.col.unwrap_or_else(|| ident.to_string());
        let width = field.width.unwrap_or(100.0);

        if field.ascending && field.descending {
            return Err(syn::Error::new(
                ident.span(),
                "`ascending` and `descending` cannot both be set",
            ));
        }
        if !filters_effective && field.filter.is_some() {
            return Err(syn::Error::new(
                ident.span(),
                "field-level `filter` or `filter(...)` requires struct-level `#[gpui_table(filters)]` or `#[gpui_table(mcp)]`",
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
            Some("left") => {
                quote! { .fixed(::gpui_component::table::ColumnFixed::Left) }
            },
            Some("right") => {
                quote! { .fixed(::gpui_component::table::ColumnFixed::Right) }
            },
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
            ::gpui_component::table::Column::new(#key, #title_expr)
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
        if field.validation.is_some() && field.filter.is_none() {
            return Err(syn::Error::new(
                ident.span(),
                "`#[koruma(...)]` table validation only applies to fields with `#[gpui_table(filter(...))]`",
            ));
        }
        if field.validation.is_some() && !mcp_enabled {
            return Err(syn::Error::new(
                ident.span(),
                "`#[koruma(...)]` table filter validation requires `#[gpui_table(mcp)]`",
            ));
        }

        if filters_effective && let Some(ref filter_options) = field.filter {
            let filter_config = filter_options.resolve(ident.to_string(), field.ty.clone());
            filter_config.validate_feature_gate()?;
            filter_shape_type_checks.push(filter_config.type_check_tokens());
            let filter_type_ts = get_filter_type_expr(&filter_config);

            filters_init.push(quote! {
                gpui_table::core::filter::FilterConfig {
                    column_index: #i,
                    filter_type: #filter_type_ts,
                }
            });

            // Collect filter field metadata for delegate generation
            filter_fields.push(FilterFieldMeta {
                field_ident: ident.clone(),
                filter_config: filter_config.clone(),
                validation: field.validation.clone(),
            });

            #[cfg(feature = "inventory")]
            {
                let field_name_str = ident.to_string();
                let field_type_str = field.ty.to_token_stream().to_string();
                let registry_filter_type = get_registry_filter_type(&filter_config);
                let shape_path = filter_config.shape_path_tokens();
                let component_path = filter_config.component_path_tokens();

                filter_variant_constructions.push(quote! {
                    gpui_table::schema::registry::FilterVariant::new(
                        gpui_table::schema::registry::ComponentShapeUse::for_field(
                            #field_name_str,
                            #shape_path,
                        )
                        .with_field_type(
                            gpui_table::schema::registry::RustType::from_macro_tokens_unchecked(#field_type_str)
                        ),
                        #registry_filter_type,
                        #component_path,
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
                            ::gpui_component::table::ColumnSort::Ascending => {
                                a_val.partial_cmp(b_val).unwrap_or(std::cmp::Ordering::Equal)
                            },
                            ::gpui_component::table::ColumnSort::Descending => {
                                b_val.partial_cmp(a_val).unwrap_or(std::cmp::Ordering::Equal)
                            },
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

        if let Some(style) = style {
            style_match_arms.push(quote! {
                #column_enum_name::#variant_ident => {
                    (#style)(self, &self.#ident, window, cx).into_any_element()
                },
            });
        }

        #[cfg(feature = "inventory")]
        {
            let field_name_str = ident.to_string();
            let field_type_str = field.ty.to_token_stream().to_string();
            let title_str = field
                .title
                .clone()
                .unwrap_or_else(|| ident.to_string().to_title_case());
            let fixed_variant = match field.fixed.as_deref() {
                Some("left") => quote! { gpui_table::schema::registry::ColumnFixed::Left },
                Some("right") => quote! { gpui_table::schema::registry::ColumnFixed::Right },
                _ => quote! { gpui_table::schema::registry::ColumnFixed::None },
            };
            let sortable = field.sortable;
            column_variant_constructions.push(quote! {
                gpui_table::schema::registry::ColumnVariant::new(
                    #field_name_str,
                    gpui_table::schema::registry::RustType::from_macro_tokens_unchecked(#field_type_str),
                    #title_str,
                    #width,
                    #sortable,
                    #fixed_variant,
                )
            });
        }
    }

    let table_title_impl = match &fluent {
        Some(Override::Explicit(_)) | Some(Override::Inherit) => {
            quote! {
                fn table_title() -> String {
                    gpui_table::component::i18n::fallback_label::<Self>()
                }
            }
        },
        None => quote! { fn table_title() -> String { Self::TABLE_TITLE.to_string() } },
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

    let style_impl = if style_match_arms.is_empty() {
        quote! {
            impl gpui_table::runtime::TableRowStyle for #struct_name {
                type ColumnId = #column_enum_name;

                fn render_table_cell(
                    &self,
                    col: Self::ColumnId,
                    window: &mut ::gpui::Window,
                    cx: &mut ::gpui::App,
                ) -> ::gpui::AnyElement {
                    use ::gpui::IntoElement as _;
                    gpui_table::runtime::default_render_cell(self, col.into(), window, cx).into_any_element()
                }
            }
        }
    } else {
        quote! {
            impl gpui_table::runtime::TableRowStyle for #struct_name {
                type ColumnId = #column_enum_name;

                fn render_table_cell(
                    &self,
                    col: Self::ColumnId,
                    window: &mut ::gpui::Window,
                    cx: &mut ::gpui::App,
                ) -> ::gpui::AnyElement {
                    use ::gpui::IntoElement as _;

                    match col {
                        #(#style_match_arms)*
                        _ => gpui_table::runtime::default_render_cell(self, col.into(), window, cx)
                            .into_any_element(),
                    }
                }
            }
        }
    };

    let generated_context_menu_impl =
        if let Some((context_menu_row_ident, context_menu_href_expr, context_menu_label_expr)) =
            context_menu_link
        {
            quote! {
                impl gpui_table::runtime::TableRowGeneratedContextMenu for #struct_name {
                    fn render_generated_table_context_menu(
                        &self,
                        _row_ix: usize,
                        menu: ::gpui_component::menu::PopupMenu,
                        _window: &mut ::gpui::Window,
                        _cx: &mut ::gpui::App,
                    ) -> ::gpui_component::menu::PopupMenu {
                        let context_menu_value = &self.#context_menu_row_ident;
                        let href = #context_menu_href_expr;
                        let label = #context_menu_label_expr;
                        menu.link(label, href)
                    }
                }
            }
        } else {
            quote! {
                impl gpui_table::runtime::TableRowGeneratedContextMenu for #struct_name {}
            }
        };

    let context_menu_impl = if !custom_context_menu {
        quote! {
            impl gpui_table::runtime::TableRowContextMenu for #struct_name {
                fn render_table_context_menu(
                    &self,
                    row_ix: usize,
                    menu: ::gpui_component::menu::PopupMenu,
                    window: &mut ::gpui::Window,
                    cx: &mut ::gpui::App,
                ) -> ::gpui_component::menu::PopupMenu {
                    use gpui_table::runtime::TableRowGeneratedContextMenu as _;
                    self.render_generated_table_context_menu(row_ix, menu, window, cx)
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

    #[cfg(feature = "mcp")]
    let mcp_impl = if let Some(mcp_options) = mcp.as_ref() {
        generate_mcp_impl(
            &struct_name,
            &table_id,
            &table_title,
            &filter_fields,
            Some(mcp_options),
            original_input,
        )?
    } else {
        quote! {}
    };

    #[cfg(not(feature = "mcp"))]
    let mcp_impl = quote! {};

    #[cfg(feature = "inventory")]
    let uses_fluent_labels = fluent.is_some();

    #[cfg(feature = "inventory")]
    let shape_impl = {
        quote! {
            gpui_table::schema::registry::inventory::submit! {
                gpui_table::schema::registry::GpuiTableShape::new(
                    stringify!(#struct_name),
                    #table_id,
                    #table_title,
                    #uses_fluent_labels,
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
        #(#filter_shape_type_checks)*

        #column_enum

        impl gpui_table::TableRowMeta for #struct_name {
            const TABLE_ID: &'static str = #table_id;
            const TABLE_TITLE: &'static str = #table_title;

            #table_title_impl

            fn table_columns() -> Vec<::gpui_component::table::Column> {
                vec![
                    #(#columns_init),*
                ]
            }

            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::runtime::TableCell + '_> {
                match col_ix {
                    #(#cell_value_match_arms)*
                    _ => Box::new(String::new()),
                }
            }

            fn table_filters() -> Vec<gpui_table::core::filter::FilterConfig> {
                vec![
                    #(#filters_init),*
                ]
            }
        }

        #shape_impl
        #style_impl
        #generated_context_menu_impl
        #context_menu_impl
        #delegate_impl
        #filter_entities_impl
        #matches_filters_impl
        #mcp_impl
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

        quote! {
            gpui_table::component::i18n::fallback_message(
                &#fluent_enum_ident::#fluent_variant_ident
            )
        }
    } else {
        let raw_title = ident.to_string().to_title_case();
        quote! { #raw_title }
    }
}
