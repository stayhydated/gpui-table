use gpui_table_schema::registry::GpuiTableShape;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use thiserror::Error;

use crate::{
    identities::{ShapeIdentities, TableIdentities as _, TableIdentitiesExt as _, parse_ident},
    imports::{Alias, ImportItem, ImportSet},
    source_path::source_path_to_use_path,
};

/// Imports every generated table story needs regardless of configuration.
const FRAMEWORK_IMPORTS: &[ImportItem] = &[
    // gpui core
    ImportItem::path("gpui::App"),
    ImportItem::aliased("gpui::AppContext", Alias::Anonymous),
    ImportItem::path("gpui::Context"),
    ImportItem::path("gpui::Entity"),
    ImportItem::path("gpui::Focusable"),
    ImportItem::path("gpui::IntoElement"),
    ImportItem::path("gpui::ParentElement"),
    ImportItem::path("gpui::Render"),
    ImportItem::path("gpui::Styled"),
    ImportItem::path("gpui::Subscription"),
    ImportItem::path("gpui::Window"),
    // gpui_component table
    ImportItem::path("gpui_component::table::DataTable"),
    ImportItem::path("gpui_component::table::TableState"),
    ImportItem::aliased("gpui_component::table::TableDelegate", Alias::Anonymous),
    ImportItem::path("gpui_component::v_flex"),
    // i18n / fluent
    ImportItem::aliased("es_fluent::ThisFtl", Alias::Anonymous),
];

/// Extra imports needed when the table has filters.
const FILTER_IMPORTS: &[ImportItem] = &[ImportItem::path("gpui_component::h_flex")];

#[derive(Debug, Error)]
pub enum TableCodegenError {
    #[error("invalid {kind} `{value}` derived from table shape `{struct_name}`")]
    InvalidIdentifier {
        kind: &'static str,
        struct_name: &'static str,
        value: String,
        #[source]
        source: syn::Error,
    },
    #[error("failed to derive a Rust module path from source path `{source_path}`")]
    InvalidSourcePath { source_path: String },
    #[error("failed to render generated imports")]
    InvalidImports {
        #[source]
        source: syn::Error,
    },
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

    /// Collect all imports needed by this table's generated file.
    ///
    /// Starts with the universal [`FRAMEWORK_IMPORTS`] base, then conditionally
    /// adds filter imports. The result can be rendered as grouped `use`
    /// statements via [`ImportSet::to_token_stream`].
    pub fn required_imports(&self) -> ImportSet {
        let mut set = ImportSet::default();
        set.extend_items(FRAMEWORK_IMPORTS);
        if self.identities.has_filters() {
            set.extend_items(FILTER_IMPORTS);
        }
        set
    }

    /// Compute all token-stream fragments and identifiers for this table.
    ///
    /// Prefer this when you want to assemble a fully custom `quote!{}` template.
    /// All conditional / derived fragments are pre-evaluated so you only need
    /// to splice them in.
    pub fn parts(&self) -> TableParts {
        self.try_parts()
            .expect("valid gpui-table shape metadata for TableShapeAdapter::parts")
    }

    /// Fallible version of [`TableShapeAdapter::parts`] for user-facing tooling.
    pub fn try_parts(&self) -> Result<TableParts, TableCodegenError> {
        let struct_name_ident = self.identities.try_struct_name_ident()?;
        let story_struct_ident = Self::story_struct_ident(&struct_name_ident);
        let delegate_struct_ident = Self::delegate_struct_ident(&struct_name_ident);

        let source_module_path =
            source_path_to_use_path(self.shape.source_path).ok_or_else(|| {
                TableCodegenError::InvalidSourcePath {
                    source_path: self.shape.source_path.to_string(),
                }
            })?;

        let collected_imports = self
            .required_imports()
            .try_to_token_stream()
            .map_err(|source| TableCodegenError::InvalidImports { source })?;
        let imports = quote! {
            use #source_module_path::*;
            #collected_imports
        };

        let delegate_creation = Self::delegate_creation_tokens(&delegate_struct_ident);
        let table_state_creation = self.table_state_creation_tokens(&struct_name_ident);
        let field_initializers = self.field_initializers();
        let struct_fields = self.struct_fields_tokens(&struct_name_ident, &delegate_struct_ident);
        let render_children = self.try_render_children()?;
        let title_expr = Self::title_expr_tokens(&struct_name_ident);

        Ok(TableParts {
            struct_name_ident,
            story_struct_ident,
            delegate_struct_ident,
            source_module_path,
            has_filters: self.identities.has_filters(),
            load_more: self.shape.load_more,
            imports,
            delegate_creation,
            table_state_creation,
            field_initializers,
            struct_fields,
            render_children,
            title_expr,
        })
    }

    /// Generate a `syn::File` from a [`TableLayout`] implementation.
    ///
    /// ```rust,ignore
    /// struct MyLayout;
    /// impl TableLayout for MyLayout {
    ///     fn generate_file(&self, parts: &TableParts) -> syn::File {
    ///         let TableParts { imports, story_struct_ident, .. } = parts;
    ///         syn::parse2(quote! {
    ///             #imports
    ///             pub struct #story_struct_ident { /* ... */ }
    ///         })
    ///         .expect("static layout template should parse")
    ///     }
    /// }
    /// let file = TableShapeAdapter::new(shape, true).try_generate_file(&MyLayout)?;
    /// ```
    pub fn generate_file(&self, layout: &impl TableLayout) -> syn::File {
        self.try_generate_file(layout)
            .expect("valid gpui-table shape metadata for TableShapeAdapter::generate_file")
    }

    /// Fallible version of [`TableShapeAdapter::generate_file`] for user-facing tooling.
    pub fn try_generate_file(
        &self,
        layout: &impl TableLayout,
    ) -> Result<syn::File, TableCodegenError> {
        let parts = self.try_parts()?;
        Ok(layout.generate_file(&parts))
    }

    fn story_struct_ident(struct_name_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}TableStory", struct_name_ident)
    }

    fn delegate_struct_ident(struct_name_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}TableDelegate", struct_name_ident)
    }

    fn filter_entities_ident(struct_name_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}FilterEntities", struct_name_ident)
    }

    fn delegate_creation_tokens(delegate_struct_ident: &syn::Ident) -> TokenStream {
        quote! {
            let delegate = #delegate_struct_ident::new(vec![]);
        }
    }

    fn table_state_creation_tokens(&self, struct_name_ident: &syn::Ident) -> TokenStream {
        let has_filters = self.identities.has_filters();
        let load_more = self.shape.load_more;

        if has_filters {
            let filter_entities_ident = Self::filter_entities_ident(struct_name_ident);

            if load_more {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    let filters =
                        #filter_entities_ident::build_for_table_loader(table.clone(), window, cx);

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            } else {
                quote! {
                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    let filters = #filter_entities_ident::build_for_table(table.clone(), cx);

                    let _subscription = cx.observe(&table, |_, _, cx| cx.notify());
                }
            }
        } else if load_more {
            quote! {
                let table = cx.new(|cx| TableState::new(delegate, window, cx));

                // Trigger initial data load
                table.update(cx, |table, cx| {
                    use gpui_table::runtime::TableDataLoader as _;
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

    fn struct_fields_tokens(
        &self,
        struct_name_ident: &syn::Ident,
        delegate_struct_ident: &syn::Ident,
    ) -> TokenStream {
        if self.identities.has_filters() {
            let filter_entities_ident = Self::filter_entities_ident(struct_name_ident);

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

    fn try_render_children(&self) -> Result<TokenStream, TableCodegenError> {
        let has_filters = self.identities.has_filters();

        let filter_views = if has_filters {
            if self.use_filter_helpers {
                quote! {
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(self.filters.all_filters())
                    )
                }
            } else {
                let filter_field_idents = self
                    .shape
                    .filters
                    .iter()
                    .map(|filter| {
                        parse_ident(
                            "filter field identifier",
                            self.shape.struct_name,
                            filter.field_name.to_string(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let views: Vec<_> = filter_field_idents
                    .iter()
                    .map(|field_ident| quote! { .child(self.filters.#field_ident.clone()) })
                    .collect();
                quote! {
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            #(#views)*
                    )
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            #filter_views
            .child(gpui_table::runtime::generated_filters::TableStatusBar::new(
                delegate.rows.len(),
                delegate.loading,
                delegate.eof,
            ))
            .child(
                DataTable::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true)
            )
        })
    }

    fn title_expr_tokens(struct_name_ident: &syn::Ident) -> TokenStream {
        quote! {
            #struct_name_ident::this_ftl()
        }
    }
}

// ── TableParts ────────────────────────────────────────────────────────────────

/// All pre-computed token-stream fragments and identifiers for one table scaffold.
///
/// Obtained via [`TableShapeAdapter::parts`] and consumed by [`TableLayout::generate_file`].
/// Every field is `pub` so custom layouts can freely destructure and splice whichever
/// pieces they need.
pub struct TableParts {
    // ── Identifiers ──────────────────────────────────────────────────────────
    /// The original struct ident, e.g. `User`.
    pub struct_name_ident: syn::Ident,
    /// Generated story struct ident, e.g. `UserTableStory`.
    pub story_struct_ident: syn::Ident,
    /// Generated delegate struct ident, e.g. `UserTableDelegate`.
    pub delegate_struct_ident: syn::Ident,
    /// Glob import path for the source module, e.g. `some_lib::structs::user`.
    pub source_module_path: syn::Path,

    // ── Flags ─────────────────────────────────────────────────────────────────
    /// True when the table has filter fields.
    pub has_filters: bool,
    /// True when the table has load_more enabled.
    pub load_more: bool,

    // ── Raw generated fragments ───────────────────────────────────────────────
    /// Grouped `use` statements (source module glob + framework base + conditional items).
    pub imports: TokenStream,
    /// `let delegate = ...;` creation.
    pub delegate_creation: TokenStream,
    /// `let table = cx.new(...);` + filter + subscription setup.
    pub table_state_creation: TokenStream,
    /// Field name tokens for the `Self { ... }` struct literal.
    pub field_initializers: TokenStream,
    /// Struct field definitions for the story struct.
    pub struct_fields: TokenStream,
    /// `.child(...)` chains for the render body.
    pub render_children: TokenStream,
    /// Expression for the story title.
    pub title_expr: TokenStream,
}

// ── TableLayout ───────────────────────────────────────────────────────────────

/// Template strategy for [`TableShapeAdapter::generate_file`].
///
/// Implement this to fully control the shape of the generated file while
/// reusing all the pre-computed [`TableParts`] fragments.
pub trait TableLayout {
    fn generate_file(&self, parts: &TableParts) -> syn::File;
}

// ── TableShape impl ──────────────────────────────────────────────────────────

impl TableShape for TableShapeAdapter<'_> {
    fn delegate_creation(&self) -> TokenStream {
        Self::delegate_creation_tokens(&self.identities.delegate_struct_ident())
    }

    fn table_state_creation(&self) -> TokenStream {
        Self::table_state_creation_tokens(self, &self.identities.struct_name_ident())
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
        Self::struct_fields_tokens(
            self,
            &self.identities.struct_name_ident(),
            &self.identities.delegate_struct_ident(),
        )
    }

    fn render_children(&self) -> TokenStream {
        self.try_render_children()
            .expect("valid filter field identifiers in gpui-table shape metadata")
    }

    fn title_expr(&self) -> TokenStream {
        Self::title_expr_tokens(&self.identities.struct_name_ident())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_schema::registry::{FilterVariant, RegistryFilterType};

    #[test]
    fn try_parts_rejects_invalid_manual_filter_field_identifiers() {
        static FILTERS: [FilterVariant; 1] = [FilterVariant::new(
            "invalid field",
            RegistryFilterType::Text,
        )];

        let shape = GpuiTableShape::new(
            "User",
            "users",
            "Users",
            &[],
            &FILTERS,
            false,
            "demo-crate/src/user.rs",
        );

        match TableShapeAdapter::new(&shape, false).try_parts() {
            Err(TableCodegenError::InvalidIdentifier {
                kind: "filter field identifier",
                ..
            }) => {},
            Err(other) => panic!("unexpected error: {other}"),
            Ok(_) => panic!("invalid filter field name should be rejected"),
        }
    }
}
