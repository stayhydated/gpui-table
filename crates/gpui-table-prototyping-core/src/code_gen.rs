use gpui_table_core::registry::{ColumnVariant, GpuiTableShape};
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

    /// The table story struct name (e.g., "UserStory")
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

    /// Fluent label enum identifier (e.g., "UserLabelVariants")
    fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelVariants", self.struct_name())
    }

    /// Fluent description enum identifier (e.g., "UserDescriptionVariants")
    fn ftl_description_ident(&self) -> syn::Ident {
        format_ident!("{}DescriptionVariants", self.struct_name())
    }

    /// The story ID literal (e.g., "user-table-story")
    fn story_id_literal(&self) -> String {
        format!("{}-table-story", self.snake_case_name().replace('_', "-"))
    }

    /// Whether this table has filters defined
    fn has_filters(&self) -> bool;
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

    fn has_filters(&self) -> bool {
        !self.0.filters.is_empty()
    }
}

/// Adapter for generating code from a table shape.
pub struct TableShapeAdapter<'a> {
    pub shape: &'a GpuiTableShape,
    pub identities: ShapeIdentities<'a>,
    pub use_filter_helpers: bool,
}

impl<'a> TableShapeAdapter<'a> {
    pub fn new(shape: &'a GpuiTableShape, use_filter_helpers: bool) -> Self {
        Self {
            shape,
            identities: ShapeIdentities::new(shape),
            use_filter_helpers,
        }
    }
}

impl TableShape for TableShapeAdapter<'_> {
    fn delegate_creation(&self) -> TokenStream {
        let delegate_struct_ident = self.identities.delegate_struct_ident();

        quote! {
            let delegate = #delegate_struct_ident::new(vec![]);
        }
    }

    fn table_state_creation(&self) -> TokenStream {
        let has_filters = self.identities.has_filters();
        let load_more = self.shape.load_more;

        if has_filters {
            let struct_name_ident = self.identities.struct_name_ident();
            let filter_entities_ident = format_ident!("{}FilterEntities", struct_name_ident);

            if load_more {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    // Trigger initial data load
                    table.update(cx, |table, cx| {
                        use gpui_table::TableDataLoader as _;
                        table.delegate_mut().load_data(window, cx);
                    });

                    // Build filter entities with reload callback
                    let table_for_reload = table.clone();
                    let filters = #filter_entities_ident::build(
                        Some(std::rc::Rc::new(move |window, cx| {
                            table_for_reload.update(cx, |table, cx| {
                                table.delegate_mut().rows.clear();
                                table.delegate_mut().eof = false;
                                use gpui_table::TableDataLoader as _;
                                table.delegate_mut().load_data(window, cx);
                            });
                        })),
                        cx,
                    );

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            } else {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    let filters = #filter_entities_ident::build(None, cx);

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            }
        } else {
            if load_more {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    // Trigger initial data load
                    table.update(cx, |table, cx| {
                        use gpui_table::TableDataLoader as _;
                        table.delegate_mut().load_data(window, cx);
                    });

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            } else {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            }
        }
    }

    fn field_initializers(&self) -> TokenStream {
        if self.identities.has_filters() {
            quote! {
                table,
                filters,
                _subscription,
            }
        } else {
            quote! {
                table,
                _subscription,
            }
        }
    }

    fn struct_fields(&self) -> TokenStream {
        let delegate_struct_ident = self.identities.delegate_struct_ident();

        if self.identities.has_filters() {
            let struct_name_ident = self.identities.struct_name_ident();
            let filter_entities_ident = format_ident!("{}FilterEntities", struct_name_ident);

            quote! {
                table: Entity<TableState<#delegate_struct_ident>>,
                filters: #filter_entities_ident,
                _subscription: Subscription,
            }
        } else {
            quote! {
                table: Entity<TableState<#delegate_struct_ident>>,
                _subscription: Subscription,
            }
        }
    }

    fn render_children(&self) -> TokenStream {
        let has_filters = self.identities.has_filters();

        let filter_views = if has_filters {
            if self.use_filter_helpers {
                // Use all_filters() helper method for cleaner code
                quote! {
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(self.filters.all_filters())
                    )
                }
            } else {
                // Manually list each filter entity
                let mut views = quote! {};
                for filter in self.shape.filters {
                    let field_ident = format_ident!("{}", filter.field_name);
                    views.extend(quote! { .child(self.filters.#field_ident.clone()) });
                }
                quote! {
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            #views
                    )
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #filter_views
            .child(gpui_table_component::TableStatusBar::new(
                delegate.rows.len(),
                delegate.loading,
                delegate.eof,
            ))
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true)
            )
        }
    }

    fn title_expr(&self) -> TokenStream {
        let struct_name_ident = self.identities.struct_name_ident();

        quote! {
            #struct_name_ident::this_ftl()
        }
    }
}
