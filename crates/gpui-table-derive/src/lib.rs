#[doc(hidden)]
mod __crate_paths;

use __crate_paths::gpui::{AnyElement, App, Context, IntoElement, Window};
use __crate_paths::gpui_component::table::{
    Column, ColumnFixed, ColumnSort, TableDelegate, TableState,
};

use darling::{FromDeriveInput, FromField, FromVariant, util::Override};
use heck::{ToPascalCase as _, ToTitleCase as _};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Path};

#[proc_macro_derive(GpuiTable, attributes(gpui_table))]
pub fn derive_gpui_table(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match TableMeta::from_derive_input(&input) {
        Ok(meta) => match expand_gpui_table(meta) {
            Ok(ts) => ts.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(gpui_table), supports(struct_named))]
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

    #[darling(default)]
    load_more: Option<Path>,
    #[darling(default)]
    eof: Option<Ident>,
    #[darling(default)]
    loading: Option<Ident>,
    #[darling(default)]
    threshold: Option<usize>,
    #[darling(default)]
    load_more_threshold: Option<usize>,
}

fn default_delegate() -> bool {
    true
}

#[derive(FromField)]
#[darling(attributes(gpui_table))]
struct TableColumn {
    ident: Option<Ident>,
    ty: syn::Type,

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
    ascending: bool,
    #[darling(default)]
    descending: bool,
    #[darling(default)]
    text_right: bool,
    #[darling(default)]
    resizable: Option<bool>,
    #[darling(default)]
    movable: Option<bool>,
    #[darling(default)]
    skip: bool,
    /// Filter component type path (e.g., `TextFilter`, `FacetedFilter`).
    /// The type must implement `TableFilterComponent`.
    #[darling(default)]
    filter: Option<Path>,
}

/// Filter field metadata for delegate generation.
#[derive(Clone)]
struct FilterFieldMeta {
    /// The field name identifier
    field_ident: Ident,
    /// The filter component type path (used for registry)
    #[allow(dead_code)]
    filter_type: Path,
    /// The value type for this filter (derived from TableFilterComponent::Value)
    value_type: proc_macro2::TokenStream,
}

/// Resolve the filter component path.
/// The path is used as-is - users must import the filter type they want to use.
fn resolve_filter_path(filter_path: &Path) -> proc_macro2::TokenStream {
    quote! { #filter_path }
}

/// Get the registry filter type for a given filter component.
#[cfg(feature = "inventory")]
fn get_registry_filter_type(filter_path: &Path) -> proc_macro2::TokenStream {
    let resolved = resolve_filter_path(filter_path);
    quote! { <#resolved as gpui_table::components::TableFilterComponent>::FILTER_TYPE }
}

/// Get the FilterType enum for runtime filter config.
fn get_filter_type_expr(filter_path: &Path, field_ty: &syn::Type) -> proc_macro2::TokenStream {
    let path_str = filter_path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    // For faceted filters, we need to include the options
    if path_str == "FacetedFilter"
        || path_str.ends_with("::FacetedFilter")
        || path_str.contains("faceted_filter")
    {
        quote! { gpui_table::filter::FilterType::Faceted(<#field_ty as gpui_table::filter::Filterable>::options()) }
    } else {
        let resolved = resolve_filter_path(filter_path);
        // Use the FILTER_TYPE constant to determine the runtime type
        quote! {
            match <#resolved as gpui_table::components::TableFilterComponent>::FILTER_TYPE {
                gpui_table::registry::RegistryFilterType::Text => gpui_table::filter::FilterType::Text,
                gpui_table::registry::RegistryFilterType::Faceted => gpui_table::filter::FilterType::Faceted(vec![]),
                gpui_table::registry::RegistryFilterType::NumberRange => gpui_table::filter::FilterType::NumberRange,
                gpui_table::registry::RegistryFilterType::DateRange => gpui_table::filter::FilterType::DateRange,
            }
        }
    }
}

fn expand_gpui_table(meta: TableMeta) -> syn::Result<proc_macro2::TokenStream> {
    let TableMeta {
        ident: struct_name,
        data,
        id,
        title,
        delegate,
        custom_style,
        fluent,
        load_more,
        eof,
        loading,
        threshold,
        load_more_threshold,
    } = meta;

    let threshold = load_more_threshold.or(threshold);

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

        if let Some(filter_path) = &field.filter {
            let filter_type_ts = get_filter_type_expr(filter_path, &field.ty);
            let resolved_path = resolve_filter_path(filter_path);

            filters_init.push(quote! {
                gpui_table::filter::FilterConfig {
                    column_index: #i,
                    filter_type: #filter_type_ts,
                }
            });

            // Collect filter field metadata for delegate generation
            // The value type is derived from TableFilterComponent::Value
            filter_fields.push(FilterFieldMeta {
                field_ident: ident.clone(),
                filter_type: filter_path.clone(),
                value_type: quote! { <#resolved_path as gpui_table::components::TableFilterComponent>::Value },
            });

            #[cfg(feature = "inventory")]
            {
                let field_name_str = ident.to_string();
                let registry_filter_type = get_registry_filter_type(filter_path);

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

    let table_title_impl = match &fluent {
        Some(Override::Explicit(key)) => {
            let key_cap = key.to_pascal_case();
            let fluent_enum = Ident::new(
                &format!("{}{}{}KvFtl", struct_name, key_cap, ""),
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
            load_more,
            eof,
            loading,
            threshold,
            &filter_fields,
        )
    } else {
        quote! {}
    };

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
                    file!()
                )
            }
        }
    };

    #[cfg(not(feature = "inventory"))]
    let shape_impl = quote! {};

    Ok(quote! {
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
    })
}

#[proc_macro_derive(Filterable, attributes(filter))]
pub fn derive_filterable(input: TokenStream) -> TokenStream {
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
    let meta = FilterableMeta::from_derive_input(&input)?;
    let enum_name = meta.ident;
    let variants = meta.data.take_enum().unwrap();

    let mut options = Vec::new();

    for variant in variants {
        let variant_ident = variant.ident;
        let value = variant_ident.to_string(); // Or snake_case? Using variant name for now.

        let label_expr = if meta.fluent {
            quote! { { use es_fluent::ToFluentString as _; Self::#variant_ident.to_fluent_string() } }
        } else {
            let label = variant
                .label
                .unwrap_or_else(|| value.clone().to_title_case());
            quote! { #label.to_string() }
        };

        let icon = match variant.icon {
            Some(path) => {
                quote! { Some(#path) }
            },
            None => quote! { None },
        };

        options.push(quote! {
            gpui_table::filter::FacetedFilterOption {
                label: #label_expr,
                value: #value.to_string(),
                count: None,
                icon: #icon,
            }
        });
    }

    Ok(quote! {
        impl gpui_table::filter::Filterable for #enum_name {
            fn options() -> Vec<gpui_table::filter::FacetedFilterOption> {
                vec![
                    #(#options),*
                ]
            }
        }
    })
}

// ... existing code ...
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

        quote! { { use es_fluent::ToFluentString as _; #fluent_enum_ident::#fluent_variant_ident.to_fluent_string() } }
    } else {
        let raw_title = ident.to_string().to_title_case();
        quote! { #raw_title }
    }
}

fn generate_delegate(
    struct_name: &Ident,
    column_enum_name: &Ident,
    sort_arms: Vec<proc_macro2::TokenStream>,
    load_more: Option<Path>,
    eof: Option<Ident>,
    loading: Option<Ident>,
    threshold: Option<usize>,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    let delegate_name = Ident::new(&format!("{}TableDelegate", struct_name), struct_name.span());
    let has_load_more = load_more.is_some();
    let _has_filters = !filter_fields.is_empty();

    let load_more_impl = if let Some(load_fn) = load_more {
        quote! {
            fn load_more(&mut self, window: &mut #Window, cx: &mut #Context<#TableState<Self>>) {
                #load_fn(self, window, cx);
            }
        }
    } else {
        quote! {}
    };

    let loading_impl = if let Some(field) = loading {
        quote! {
            fn loading(&self, app: &#App) -> bool {
                self.#field(app)
            }
        }
    } else {
        quote! {
            fn loading(&self, _: &#App) -> bool {
                self.full_loading
            }
        }
    };

    let has_more_impl = if has_load_more {
        if let Some(field) = eof {
            quote! {
                fn has_more(&self, app: &#App) -> bool {
                    if self.loading {
                        return false;
                    }
                    !self.#field(app)
                }
            }
        } else {
            quote! {
                fn has_more(&self, _: &#App) -> bool {
                    if self.loading {
                        return false;
                    }
                    !self.eof
                }
            }
        }
    } else {
        quote! {}
    };

    let threshold_impl = if let Some(val) = threshold {
        quote! {
            fn load_more_threshold(&self) -> usize {
                #val
            }
        }
    } else {
        quote! {}
    };

    let columns_init_expr = quote! { <#struct_name as gpui_table::TableRowMeta>::table_columns() };

    // Generate a separate Filters struct if there are any filters
    let filters_struct_name = Ident::new(&format!("{}Filters", struct_name), struct_name.span());
    let has_filters = !filter_fields.is_empty();

    let filter_field_defs: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let value_type = &f.value_type;
            quote! {
                pub #field_ident: #value_type,
            }
        })
        .collect();

    let filter_field_inits: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            quote! {
                #field_ident: Default::default(),
            }
        })
        .collect();

    // Generate the Filters struct (only if there are filters)
    let filters_struct_def = if has_filters {
        quote! {
            /// Filter values for the #struct_name table.
            #[derive(Clone, Debug, Default)]
            pub struct #filters_struct_name {
                #(#filter_field_defs)*
            }
        }
    } else {
        quote! {}
    };

    // Delegate field for filters
    let delegate_filters_field = if has_filters {
        quote! { pub filters: #filters_struct_name, }
    } else {
        quote! {}
    };

    let delegate_filters_init = if has_filters {
        quote! { filters: #filters_struct_name { #(#filter_field_inits)* }, }
    } else {
        quote! {}
    };

    quote! {
        #filters_struct_def

        pub struct #delegate_name {
            pub rows: Vec<#struct_name>,
            columns: Vec<#Column>,
            pub visible_rows: std::ops::Range<usize>,
            pub visible_cols: std::ops::Range<usize>,
            pub eof: bool,
            pub loading: bool,
            pub full_loading: bool,
            #delegate_filters_field
        }

        impl #delegate_name {
            pub fn new(rows: Vec<#struct_name>) -> Self {
                Self {
                    rows,
                    columns: #columns_init_expr,
                    visible_rows: Default::default(),
                    visible_cols: Default::default(),
                    eof: false,
                    loading: false,
                    full_loading: false,
                    #delegate_filters_init
                }
            }
        }

        impl #TableDelegate for #delegate_name {
            fn columns_count(&self, _: &#App) -> usize {
                self.columns.len()
            }

            fn rows_count(&self, _: &#App) -> usize {
                self.rows.len()
            }

            fn column(&self, col_ix: usize, _: &#App) -> #Column {
                <#struct_name as gpui_table::TableRowMeta>::table_columns()
                    .into_iter()
                    .nth(col_ix)
                    .expect("Invalid column index")
            }

            fn render_td(
                &mut self,
                row_ix: usize,
                col_ix: usize,
                window: &mut #Window,
                cx: &mut #Context<#TableState<Self>>,
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

            #loading_impl
            #has_more_impl
            #load_more_impl
            #threshold_impl

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

#[proc_macro_derive(TableCell)]
pub fn derive_table_cell(input: TokenStream) -> TokenStream {
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
