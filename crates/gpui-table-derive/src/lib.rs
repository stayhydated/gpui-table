#[doc(hidden)]
mod __crate_paths;
mod components;
mod impl_attr;

use __crate_paths::gpui::{AnyElement, App, Context, Entity, IntoElement, Window};
use __crate_paths::gpui_component::table::{
    Column, ColumnFixed, ColumnSort, TableDelegate, TableState,
};
use components::FilterComponents;

use darling::{FromDeriveInput, FromField, FromVariant, util::Override};
use heck::{ToPascalCase as _, ToTitleCase as _};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Path};

/// Attribute macro for table delegate impl blocks.
///
/// This macro processes an `impl` block for a table delegate and generates
/// the appropriate `TableDelegate` trait method implementations based on
/// inner attributes.
///
/// # Supported Attributes
///
/// - `#[load_more]` - Marks a method as the load_more handler
/// - `#[threshold]` - Marks a const as the load_more threshold value
/// - `#[eof]` - Marks a const containing the eof field name (defaults to "eof")
///
/// # Example
///
/// ```ignore
/// #[gpui_table_impl]
/// impl ProductTableDelegate {
///     #[threshold]
///     const LOAD_MORE_THRESHOLD: usize = 20;
///
///     #[load_more]
///     pub fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
///         // Load more data...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn gpui_table_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_attr::gpui_table_impl(attr.into(), item.into()).into()
}

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
    loading: Option<Ident>,

    /// Enable filter support. When set, generates FilterEntities, FilterValues,
    /// and matches_filters() method. Field-level `filter(...)` attributes are
    /// only processed when this is enabled.
    #[darling(default)]
    filters: bool,
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
    /// Filter component configuration using function-style syntax.
    /// Examples: `filter = text()`, `filter = number_range(min = 0, max = 100)`
    #[darling(default)]
    filter: Option<FilterComponents>,
}

/// Filter field metadata for delegate generation.
#[derive(Clone)]
struct FilterFieldMeta {
    /// The field name identifier
    field_ident: Ident,
    /// The filter component configuration
    filter_config: FilterComponents,
    /// The value type for this filter
    value_type: proc_macro2::TokenStream,
    /// The field type (e.g., String, bool, Priority enum, chrono::DateTime)
    field_type: syn::Type,
    /// Column index for this filter
    column_index: usize,
}

/// Get the filter component type tokens for code generation.
/// For FacetedFilter, the field_ty is required to generate the generic parameter.
///
/// Returns a tuple of (type_tokens, type_with_turbofish) where:
/// - type_tokens: For use in type position (e.g., `Entity<FacetedFilter<T>>`)
/// - type_with_turbofish: For use in expression position (e.g., `FacetedFilter::<T>::new_for()`)
fn get_filter_type_tokens(
    filter: &FilterComponents,
    field_ty: Option<&syn::Type>,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => quote! { gpui_table::components::TextFilter },
        FilterComponents::NumberRange(_) => quote! { gpui_table::components::NumberRangeFilter },
        FilterComponents::DateRange(_) => quote! { gpui_table::components::DateRangeFilter },
        FilterComponents::Faceted(_) => {
            if let Some(ty) = field_ty {
                quote! { gpui_table::components::FacetedFilter::<#ty> }
            } else {
                // Fallback for cases where field_ty is not available (shouldn't happen in practice)
                quote! { gpui_table::components::FacetedFilter::<String> }
            }
        },
    }
}

/// Get the registry filter type for a given filter component.
#[cfg(feature = "inventory")]
fn get_registry_filter_type(filter: &FilterComponents) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => {
            quote! { gpui_table::registry::RegistryFilterType::Text }
        },
        FilterComponents::NumberRange(_) => {
            quote! { gpui_table::registry::RegistryFilterType::NumberRange }
        },
        FilterComponents::DateRange(_) => {
            quote! { gpui_table::registry::RegistryFilterType::DateRange }
        },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::registry::RegistryFilterType::Faceted }
        },
    }
}

/// Get the FilterType enum for runtime filter config.
fn get_filter_type_expr(
    filter: &FilterComponents,
    field_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(_) => quote! { gpui_table::filter::FilterType::Text },
        FilterComponents::NumberRange(_) => quote! { gpui_table::filter::FilterType::NumberRange },
        FilterComponents::DateRange(_) => quote! { gpui_table::filter::FilterType::DateRange },
        FilterComponents::Faceted(_) => {
            quote! { gpui_table::filter::FilterType::Faceted(<#field_ty as gpui_table::filter::Filterable>::options()) }
        },
    }
}

/// Generate chain method calls for filter options.
fn generate_filter_chain_methods(filter: &FilterComponents) -> proc_macro2::TokenStream {
    use components::TextValidation;

    match filter {
        FilterComponents::Text(opts) => {
            let mut chain = quote! {};

            // Generate validation method if specified
            if let Some(ref validation) = opts.validate {
                let validation_chain = match validation {
                    TextValidation::Alphabetic => quote! {
                        use gpui_table::components::TextFilterExt as _;
                        let filter = filter.alphabetic_only(cx);
                    },
                    TextValidation::Numeric => quote! {
                        use gpui_table::components::TextFilterExt as _;
                        let filter = filter.numeric_only(cx);
                    },
                    TextValidation::Alphanumeric => quote! {
                        use gpui_table::components::TextFilterExt as _;
                        let filter = filter.alphanumeric_only(cx);
                    },
                    TextValidation::Custom(path) => quote! {
                        use gpui_table::components::TextFilterExt as _;
                        let filter = filter.validate(#path, cx);
                    },
                };
                chain = quote! { #chain #validation_chain };
            }

            chain
        },
        FilterComponents::NumberRange(opts) => {
            let mut chain = quote! {};

            // Generate .range() call if min or max is specified
            if opts.min.is_some() || opts.max.is_some() {
                let min_val = opts.min.unwrap_or(0.0);
                let max_val = opts.max.unwrap_or(100.0);
                // Convert f64 to string for Decimal parsing at compile time
                let min_str = min_val.to_string();
                let max_str = max_val.to_string();
                chain = quote! {
                    #chain
                    use gpui_table::components::NumberRangeFilterExt as _;
                    let filter = filter.range(
                        rust_decimal::Decimal::from_str_exact(#min_str).unwrap(),
                        rust_decimal::Decimal::from_str_exact(#max_str).unwrap(),
                        cx,
                    );
                };
            }

            // Generate .step() call if step is specified
            if let Some(step_val) = opts.step {
                let step_str = step_val.to_string();
                chain = quote! {
                    #chain
                    let filter = filter.step(rust_decimal::Decimal::from_str_exact(#step_str).unwrap(), cx);
                };
            }

            chain
        },
        FilterComponents::DateRange(_opts) => {
            // Date range filter has no configurable options yet
            quote! {}
        },
        FilterComponents::Faceted(opts) => {
            let mut chain = quote! {};

            // Generate .searchable() call if enabled
            if opts.searchable {
                chain = quote! {
                    #chain
                    use gpui_table::components::FacetedFilterExt as _;
                    let filter = filter.searchable(cx);
                };
            }

            chain
        },
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
        loading,
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
            let filter_type_ts = get_filter_type_expr(filter_config, &field.ty);
            let filter_type_tokens = get_filter_type_tokens(filter_config, Some(&field.ty));

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
                    filter_config: filter_config.clone(),
                    value_type: quote! { <#filter_type_tokens as gpui_table::components::TableFilterComponent>::Value },
                    field_type: field.ty.clone(),
                    column_index: i,
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
            loading,
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
        #filter_entities_impl
        #matches_filters_impl
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
    let mut variant_name_arms = Vec::new();
    let mut from_filter_string_arms = Vec::new();

    for variant in &variants {
        let variant_ident = &variant.ident;
        let value = variant_ident.to_string(); // Or snake_case? Using variant name for now.

        let label_expr = if meta.fluent {
            quote! { { use es_fluent::ToFluentString as _; Self::#variant_ident.to_fluent_string() } }
        } else {
            let label = variant
                .label
                .clone()
                .unwrap_or_else(|| value.clone().to_title_case());
            quote! { #label.to_string() }
        };

        let icon = match &variant.icon {
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
        impl gpui_table::filter::FilterValue for #enum_name {
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

        impl gpui_table::filter::Filterable for #enum_name {
            fn options() -> Vec<gpui_table::filter::FacetedFilterOption> {
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
    loading: Option<Ident>,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    let delegate_name = Ident::new(&format!("{}TableDelegate", struct_name), struct_name.span());
    let _has_filters = !filter_fields.is_empty();

    // Generate load_more related implementations.
    // Always delegate to LoadMoreDelegate trait (implemented by #[gpui_table_impl] with #[load_more]).
    let load_more_impl = quote! {
        fn load_more(&mut self, window: &mut #Window, cx: &mut #Context<#TableState<Self>>) {
            gpui_table::__private::LoadMoreDelegate::load_more(self, window, cx);
        }
    };

    let has_more_impl = quote! {
        fn has_more(&self, app: &#App) -> bool {
            gpui_table::__private::LoadMoreDelegate::has_more(self, app)
        }
    };

    let threshold_impl = quote! {
        fn load_more_threshold(&self) -> usize {
            gpui_table::__private::LoadMoreDelegate::load_more_threshold(self)
        }
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

    let columns_init_expr = quote! { <#struct_name as gpui_table::TableRowMeta>::table_columns() };

    quote! {

        pub struct #delegate_name {
            pub rows: Vec<#struct_name>,
            columns: Vec<#Column>,
            pub visible_rows: std::ops::Range<usize>,
            pub visible_cols: std::ops::Range<usize>,
            pub eof: bool,
            pub loading: bool,
            pub full_loading: bool,
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

/// Generate the FilterEntities struct that holds all filter Entity<T> fields
/// and provides builder methods for creating them.
fn generate_filter_entities(
    struct_name: &Ident,
    filter_fields: &[FilterFieldMeta],
    fluent_config: &Option<Override<String>>,
) -> proc_macro2::TokenStream {
    if filter_fields.is_empty() {
        return quote! {};
    }

    let filter_entities_name = Ident::new(
        &format!("{}FilterEntities", struct_name),
        struct_name.span(),
    );

    // Generate Entity fields for each filter
    let entity_field_defs: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let filter_type_tokens = get_filter_type_tokens(&f.filter_config, Some(&f.field_type));
            quote! {
                pub #field_ident: #Entity<#filter_type_tokens>,
            }
        })
        .collect();

    // Generate the build method that creates all filter entities
    let filter_builders: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let filter_type_tokens = get_filter_type_tokens(&f.filter_config, Some(&f.field_type));

            // Determine the title expression based on fluent config
            let title_expr =
                determine_filter_title_expr(&f.field_ident, fluent_config, struct_name);

            // Check if this is a FacetedFilter using the enum method
            if f.filter_config.is_faceted() {
                // Generate chain methods for options
                let chain_methods = generate_filter_chain_methods(&f.filter_config);

                // For FacetedFilter<T>, use new_for (type is already in the generic parameter)
                quote! {
                    let #field_ident = {
                        let on_filter_change = on_filter_change.clone();
                        let filter = #filter_type_tokens::new_for(
                            || #title_expr,
                            Default::default(),
                            move |_value, window, cx| {
                                // Notify callback for server-side filtering
                                if let Some(ref on_change) = on_filter_change {
                                    let on_change = on_change.clone();
                                    window.defer(cx, move |window, cx| {
                                        on_change(window, cx);
                                    });
                                }
                            },
                            cx,
                        );
                        #chain_methods
                        filter
                    };
                }
            } else {
                // Generate chain methods for options
                let chain_methods = generate_filter_chain_methods(&f.filter_config);

                // For other filters (TextFilter, NumberRangeFilter, DateRangeFilter)
                quote! {
                    let #field_ident = {
                        let on_filter_change = on_filter_change.clone();
                        let filter = #filter_type_tokens::new(
                            #title_expr,
                            Default::default(),
                            move |_value, window, cx| {
                                // Notify callback for server-side filtering
                                if let Some(ref on_change) = on_filter_change {
                                    let on_change = on_change.clone();
                                    window.defer(cx, move |window, cx| {
                                        on_change(window, cx);
                                    });
                                }
                            },
                            cx,
                        );
                        #chain_methods
                        filter
                    };
                }
            }
        })
        .collect();

    // Field names for struct construction
    let field_names: Vec<&Ident> = filter_fields.iter().map(|f| &f.field_ident).collect();

    // Generate clone implementations for each entity
    let clone_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            quote! { #field_ident: self.#field_ident.clone(), }
        })
        .collect();

    // Generate value getter methods for each filter
    let value_getters: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let getter_name = Ident::new(&format!("{}_value", field_ident), field_ident.span());

            match &f.filter_config {
                FilterComponents::Text(_) => {
                    quote! {
                        /// Get the current value of the #field_ident text filter.
                        pub fn #getter_name(&self, cx: &#App) -> String {
                            self.#field_ident.read(cx).value().to_string()
                        }
                    }
                }
                FilterComponents::NumberRange(_) => {
                    quote! {
                        /// Get the current value of the #field_ident number range filter.
                        pub fn #getter_name(&self, cx: &#App) -> (Option<rust_decimal::Decimal>, Option<rust_decimal::Decimal>) {
                            self.#field_ident.read(cx).value()
                        }
                    }
                }
                FilterComponents::Faceted(_) => {
                    let field_type = &f.field_type;
                    quote! {
                        /// Get the current value of the #field_ident faceted filter.
                        pub fn #getter_name(&self, cx: &#App) -> std::collections::HashSet<#field_type> {
                            self.#field_ident.read(cx).value().clone()
                        }
                    }
                }
                FilterComponents::DateRange(_) => {
                    quote! {
                        /// Get the current value of the #field_ident date range filter.
                        pub fn #getter_name(&self, cx: &#App) -> (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
                            self.#field_ident.read(cx).value()
                        }
                    }
                }
            }
        })
        .collect();

    // Generate render helpers that group filters by type
    let (text_filters, number_filters, faceted_filters, date_filters) =
        categorize_filters(filter_fields);

    let text_filter_render = if !text_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = text_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all text filters as children (returns impl IntoElement).
            pub fn text_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let number_filter_render = if !number_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = number_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all number range filters as children (returns impl IntoElement).
            pub fn number_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let faceted_filter_render = if !faceted_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = faceted_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all faceted filters as children (returns impl IntoElement).
            pub fn faceted_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let date_filter_render = if !date_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = date_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all date range filters as children (returns impl IntoElement).
            pub fn date_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    // Generate all_filters render method
    let all_filter_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let ident = &f.field_ident;
            quote! { .child(self.#ident.clone()) }
        })
        .collect();

    // Generate FilterValues struct for client-side filtering
    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());

    let filter_values_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let value_type = match &f.filter_config {
                FilterComponents::Text(_) => quote! { gpui_table::filter::TextValue },
                FilterComponents::NumberRange(_) => {
                    quote! { gpui_table::filter::RangeValue<rust_decimal::Decimal> }
                },
                FilterComponents::Faceted(_) => {
                    let ty = &f.field_type;
                    quote! { gpui_table::filter::FacetedValue<#ty> }
                },
                FilterComponents::DateRange(_) => {
                    quote! { gpui_table::filter::RangeValue<chrono::NaiveDate> }
                },
            };
            quote! {
                pub #field_ident: #value_type,
            }
        })
        .collect();

    // Generate read_values method that populates FilterValues from FilterEntities
    let read_values_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let getter_name = Ident::new(&format!("{}_value", field_ident), field_ident.span());
            match &f.filter_config {
                FilterComponents::Text(_) => quote! {
                    #field_ident: gpui_table::filter::TextValue::from(self.#getter_name(cx)),
                },
                FilterComponents::NumberRange(_) => quote! {
                    #field_ident: gpui_table::filter::RangeValue::from(self.#getter_name(cx)),
                },
                FilterComponents::Faceted(_) => quote! {
                    #field_ident: gpui_table::filter::FacetedValue::from(self.#getter_name(cx)),
                },
                FilterComponents::DateRange(_) => quote! {
                    #field_ident: gpui_table::filter::RangeValue::from(self.#getter_name(cx)),
                },
            }
        })
        .collect();

    // Generate has_active_filters check expressions
    let has_active_checks: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            quote! { self.#field_ident.is_active() }
        })
        .collect();

    quote! {
        /// Entity handles for all filter UI components.
        /// Generated by the `#[derive(GpuiTable)]` macro.
        pub struct #filter_entities_name {
            #(#entity_field_defs)*
        }

        impl Clone for #filter_entities_name {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields)*
                }
            }
        }

        impl #filter_entities_name {
            /// Build all filter entities for server-side filtering.
            ///
            /// # Arguments
            /// * `on_filter_change` - Optional callback invoked after any filter changes.
            ///   Use this for triggering data reload with new filter parameters.
            /// * `cx` - The application context
            pub fn build(
                on_filter_change: Option<std::rc::Rc<dyn Fn(&mut #Window, &mut #App) + 'static>>,
                cx: &mut #App,
            ) -> Self {
                use gpui_table::components::TableFilterComponent as _;

                #(#filter_builders)*

                Self {
                    #(#field_names,)*
                }
            }

            #text_filter_render
            #number_filter_render
            #faceted_filter_render
            #date_filter_render

            // Value getters for server-side filtering
            #(#value_getters)*
        }

        impl gpui_table::filter::FilterEntitiesExt for #filter_entities_name {
            type Values = #filter_values_name;

            fn read_values(&self, cx: &#App) -> Self::Values {
                #filter_values_name {
                    #(#read_values_fields)*
                }
            }

            fn all_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().flex_wrap().items_center().gap_2()
                    #(#all_filter_fields)*
            }
        }

        /// Plain data struct holding all filter values.
        /// Generated by the `#[derive(GpuiTable)]` macro for client-side filtering.
        #[derive(Clone, Debug, Default)]
        pub struct #filter_values_name {
            #(#filter_values_fields)*
        }

        impl gpui_table::filter::FilterValuesExt for #filter_values_name {
            fn has_active_filters(&self) -> bool {
                #(#has_active_checks)||*
            }
        }

    }
}

/// Generate the matches_filters() method on the struct.
/// This method checks if all filter values match the struct's fields.
fn generate_matches_filters_method(
    struct_name: &Ident,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    if filter_fields.is_empty() {
        return quote! {};
    }

    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());

    // Generate match expressions for each filter field
    let match_exprs: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;

            match &f.filter_config {
                FilterComponents::Text(_) => {
                    // TextValue::matches takes &str
                    quote! { filters.#field_ident.matches(&self.#field_ident) }
                }
                FilterComponents::NumberRange(_) => {
                    // RangeValue<Decimal>::matches takes &Decimal
                    // Convert numeric types to Decimal
                    quote! { filters.#field_ident.matches(&gpui_table::filter::IntoDecimal::into_decimal(&self.#field_ident)) }
                }
                FilterComponents::DateRange(_) => {
                    // RangeValue<NaiveDate>::matches takes &NaiveDate
                    // Convert DateTime to NaiveDate if needed
                    quote! { filters.#field_ident.matches(&gpui_table::filter::IntoNaiveDate::into_naive_date(&self.#field_ident)) }
                }
                FilterComponents::Faceted(_) => {
                    // FacetedValue<T>::matches takes &T
                    quote! { filters.#field_ident.matches(&self.#field_ident) }
                }
            }
        })
        .collect();

    quote! {
        impl gpui_table::filter::Matchable<#filter_values_name> for #struct_name {
            fn matches_filters(&self, filters: &#filter_values_name) -> bool {
                #(#match_exprs)&&*
            }
        }
    }
}

/// Categorize filters by their type for grouped rendering.
fn categorize_filters(
    filter_fields: &[FilterFieldMeta],
) -> (
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
) {
    let mut text = Vec::new();
    let mut number = Vec::new();
    let mut faceted = Vec::new();
    let mut date = Vec::new();

    for f in filter_fields {
        match &f.filter_config {
            FilterComponents::Text(_) => text.push(f),
            FilterComponents::NumberRange(_) => number.push(f),
            FilterComponents::Faceted(_) => faceted.push(f),
            FilterComponents::DateRange(_) => date.push(f),
        }
    }

    (text, number, faceted, date)
}

/// Determine the title expression for a filter based on fluent config.
fn determine_filter_title_expr(
    field_ident: &Ident,
    fluent_config: &Option<Override<String>>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    if let Some(fluent) = fluent_config {
        let fluent_enum_ident = match fluent {
            Override::Explicit(key) => {
                let key_cap = key.to_pascal_case();
                Ident::new(
                    &format!("{}{}KvFtl", struct_name, key_cap),
                    struct_name.span(),
                )
            },
            Override::Inherit => Ident::new(&format!("{}KvFtl", struct_name), struct_name.span()),
        };

        let field_name = field_ident.to_string().to_pascal_case();
        let fluent_variant_ident = Ident::new(&field_name, field_ident.span());

        quote! { { use es_fluent::ToFluentString as _; #fluent_enum_ident::#fluent_variant_ident.to_fluent_string() } }
    } else {
        let raw_title = field_ident.to_string().to_title_case();
        quote! { #raw_title.to_string() }
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
