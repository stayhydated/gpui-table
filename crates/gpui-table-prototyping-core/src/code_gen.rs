use gpui_table_core::registry::{ColumnVariant, GpuiTableShape, RegistryFilterType};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Trait for deriving various identifier names from a table shape.
pub trait TableIdentities {
    /// The original struct name (e.g., "User")
    fn struct_name(&self) -> &'static str;

    /// The struct name as an identifier
    fn struct_name_ident(&self) -> syn::Ident {
        syn::parse_str(self.struct_name()).unwrap()
    }

    /// The table story struct name (e.g., "UserTableStory")
    fn story_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableStory", self.struct_name())
    }

    /// The table delegate struct name (e.g., "UserTableDelegate")
    fn delegate_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableDelegate", self.struct_name())
    }

    /// The table ID
    fn table_id(&self) -> &'static str;

    /// The table title
    fn table_title(&self) -> &'static str;

    /// The snake_case version of struct name for file paths
    fn snake_case_name(&self) -> String {
        self.struct_name().to_snake_case()
    }

    /// Snake case name as identifier (for import paths)
    fn snake_case_ident(&self) -> syn::Ident {
        syn::parse_str(&self.snake_case_name()).unwrap()
    }

    /// Fluent label enum identifier (e.g., "UserLabelKvFtl")
    fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelKvFtl", self.struct_name())
    }

    /// Fluent description enum identifier (e.g., "UserDescriptionKvFtl")
    fn ftl_description_ident(&self) -> syn::Ident {
        format_ident!("{}DescriptionKvFtl", self.struct_name())
    }

    /// The story ID literal (e.g., "user-table-story")
    fn story_id_literal(&self) -> String {
        format!("{}-table-story", self.snake_case_name().replace('_', "-"))
    }
}

/// Trait for generating different parts of the table story code.
pub trait TableShape {
    /// Generate delegate state creation (e.g., `let delegate = ...;`)
    fn delegate_creation(&self) -> TokenStream;

    /// Generate table state creation (e.g., `let table = cx.new(...);`)
    fn table_state_creation(&self) -> TokenStream;

    /// Generate struct field initializers (for the Self { ... } block)
    fn field_initializers(&self) -> TokenStream;

    /// Generate struct field definitions (for the struct definition)
    fn struct_fields(&self) -> TokenStream;

    /// Generate render children (the .child(...) calls)
    fn render_children(&self) -> TokenStream;

    /// Generate story title expression
    fn title_expr(&self) -> TokenStream;
}

/// Identities wrapper for GpuiTableShape
pub struct ShapeIdentities<'a>(&'a GpuiTableShape);

impl<'a> ShapeIdentities<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self {
        Self(shape)
    }

    /// Get the underlying shape
    pub fn shape(&self) -> &'a GpuiTableShape {
        self.0
    }

    /// Get columns
    pub fn columns(&self) -> &'static [ColumnVariant] {
        self.0.columns
    }
}

impl TableIdentities for ShapeIdentities<'_> {
    fn struct_name(&self) -> &'static str {
        self.0.struct_name
    }

    fn table_id(&self) -> &'static str {
        self.0.table_id
    }

    fn table_title(&self) -> &'static str {
        self.0.table_title
    }
}

/// Adapter for generating code from a table shape.
pub struct TableShapeAdapter<'a> {
    pub shape: &'a GpuiTableShape,
    pub identities: ShapeIdentities<'a>,
}

impl<'a> TableShapeAdapter<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self {
        Self {
            shape,
            identities: ShapeIdentities::new(shape),
        }
    }

    fn get_column_title(&self, field_name: &str) -> String {
        self.shape
            .columns
            .iter()
            .find(|c| c.field_name == field_name)
            .map(|c| c.title.to_string())
            .unwrap_or_else(|| field_name.to_string())
    }
}

impl TableShape for TableShapeAdapter<'_> {
    fn delegate_creation(&self) -> TokenStream {
        let delegate_struct_ident = self.identities.delegate_struct_ident();

        quote! {
            let mut delegate = #delegate_struct_ident::new(vec![]);
            for _ in 0..100 {
                delegate.rows.push(fake::Faker.fake());
            }
        }
    }

    fn table_state_creation(&self) -> TokenStream {
        quote! {
            let table = cx.new(|cx| TableState::new(delegate, window, cx));
        }
    }

    fn field_initializers(&self) -> TokenStream {
        let mut initializers = quote! {
            table,
        };

        for filter in self.shape.filters {
            let field_ident = format_ident!("filter_{}", filter.field_name);
            let init = match filter.filter_type {
                RegistryFilterType::Faceted => quote! { Default::default() },
                RegistryFilterType::DateRange => quote! { (None, None) },
                RegistryFilterType::NumberRange => quote! { (None, None) },
                RegistryFilterType::Text => quote! { String::new() },
            };
            initializers.extend(quote! { #field_ident: #init, });
        }
        initializers
    }

    fn struct_fields(&self) -> TokenStream {
        let delegate_struct_ident = self.identities.delegate_struct_ident();

        let mut fields = quote! {
            table: Entity<TableState<#delegate_struct_ident>>,
        };

        for filter in self.shape.filters {
            let field_ident = format_ident!("filter_{}", filter.field_name);
            let ty = match filter.filter_type {
                RegistryFilterType::Faceted => quote! { std::collections::HashSet<String> },
                RegistryFilterType::DateRange => {
                    quote! { (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) }
                },
                RegistryFilterType::NumberRange => quote! { (Option<f64>, Option<f64>) },
                RegistryFilterType::Text => quote! { String },
            };
            fields.extend(quote! { #field_ident: #ty, });
        }
        fields
    }

    fn render_children(&self) -> TokenStream {
        let mut filter_views = quote! {};

        for filter in self.shape.filters {
            let field_name = filter.field_name;
            let field_ident = format_ident!("filter_{}", field_name);
            let title = self.get_column_title(field_name);

            // Look up column for type if needed
            let col = self
                .shape
                .columns
                .iter()
                .find(|c| c.field_name == field_name)
                .expect("Filter field not found in columns");

            // Handle generic types or complex types by using syn::parse_str
            // If it fails, it might be a simple identifier.
            let field_type_str = col.field_type;
            let field_type: syn::Type = syn::parse_str(field_type_str).unwrap_or_else(|_| {
                syn::parse_str(&format!("crate::{}", field_type_str)).unwrap_or_else(|_| {
                    panic!("Could not parse type for filter: {}", field_type_str)
                })
            });

            let component = match filter.filter_type {
                RegistryFilterType::Faceted => {
                    quote! {
                         gpui_table_components::faceted_filter::FacetedFilter::build_with_options(
                            #title,
                            <#field_type as gpui_table::filter::Filterable>::options(),
                            self.#field_ident.clone(),
                            move |new_val, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.#field_ident = new_val;
                                    cx.notify();
                                });
                            },
                            cx
                        )
                    }
                },
                RegistryFilterType::DateRange => {
                    quote! {
                        gpui_table_components::date_range_filter::DateRangeFilter::build(
                            #title,
                            self.#field_ident,
                            move |new_val, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.#field_ident = new_val;
                                    cx.notify();
                                });
                            },
                            cx
                        )
                    }
                },
                RegistryFilterType::NumberRange => {
                    quote! {
                        gpui_table_components::number_range_filter::NumberRangeFilter::build(
                            #title,
                            self.#field_ident,
                            move |new_val, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.#field_ident = new_val;
                                    cx.notify();
                                });
                            },
                            cx
                        )
                    }
                },
                RegistryFilterType::Text => {
                    quote! {
                        gpui_table_components::text_filter::TextFilter::build(
                            #title,
                            self.#field_ident.clone(),
                            move |new_val, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.#field_ident = new_val;
                                    cx.notify();
                                });
                            },
                            cx
                        )
                    }
                },
            };

            filter_views.extend(quote! { .child(#component) });
        }

        quote! {
            .child(
                gpui_component::h_flex()
                    .gap_2()
                    .flex_wrap()
                    #filter_views
            )
            .child(format!("Total Rows: {}", rows_count))
            .child(Table::new(&self.table))
        }
    }

    fn title_expr(&self) -> TokenStream {
        let struct_name_ident = self.identities.struct_name_ident();

        quote! {
            #struct_name_ident::this_ftl()
        }
    }
}
