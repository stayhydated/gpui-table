use gpui_table_schema::registry::GpuiTableShape;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use thiserror::Error;

use crate::{
    identities::{ShapeIdentities, TableIdentities as _, TableIdentitiesExt as _, parse_ident},
    source_path::source_path_to_use_path,
};
use component_shape_codegen::imports::{Alias, ImportItem, ImportSet};

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

    /// Generate story title expression.
    ///
    /// Fluent-backed titles may reference a `cx: &gpui::App` parameter from
    /// the generated `gpui_storybook::Story::title` implementation.
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
    /// Starts with the universal `FRAMEWORK_IMPORTS` base, then conditionally
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
        let title_expr = self.title_expr_tokens();

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
                            filter.shape_use.field_name().to_string(),
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
            .child(gpui_table_component::TableStatusBar::new(
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

    fn title_expr_tokens(&self) -> TokenStream {
        if self.identities.uses_fluent_labels() {
            let struct_name_ident = self.identities.struct_name_ident();
            quote! {
                gpui_table_component::i18n::localize_label::<#struct_name_ident>(cx)
            }
        } else {
            let title = self.identities.table_title();
            quote! {
                #title.to_string()
            }
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
    ///
    /// For fluent-backed tables this expression expects to be emitted inside a
    /// scope that provides `cx: &gpui::App`.
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
        self.title_expr_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_schema::registry::{
        ColumnFixed, ColumnVariant, ComponentFieldName, ComponentShapeUse, FilterVariant,
        RegistryFilterType, RustPath, RustType,
    };

    static COLUMNS: [ColumnVariant; 1] = [ColumnVariant::new(
        "name",
        RustType::from_macro_tokens_unchecked("String"),
        "Name",
        120.0,
        true,
        ColumnFixed::None,
    )];
    static FILTERS: [FilterVariant; 1] = [FilterVariant::new(
        ComponentShapeUse::new(
            ComponentFieldName::new("name"),
            RustPath::from_macro_tokens_unchecked("gpui_table_component::TextFilter"),
        ),
        RegistryFilterType::Text,
        RustPath::from_macro_tokens_unchecked(
            "gpui_table::runtime::generated_filters::text_filter::TextFilter",
        ),
    )];

    fn shape(
        filters: bool,
        load_more: bool,
        fluent: bool,
        source_path: &'static str,
    ) -> GpuiTableShape {
        GpuiTableShape::new(
            "User",
            "users",
            "Users",
            fluent,
            &COLUMNS,
            if filters { &FILTERS } else { &[] },
            load_more,
            source_path,
        )
    }

    struct MinimalLayout;

    impl TableLayout for MinimalLayout {
        fn generate_file(&self, parts: &TableParts) -> syn::File {
            let story_ident = &parts.story_struct_ident;
            syn::parse2(quote! { pub struct #story_ident; }).unwrap()
        }
    }

    #[test]
    fn try_parts_rejects_invalid_manual_filter_field_identifiers() {
        static FILTERS: [FilterVariant; 1] = [FilterVariant::new(
            ComponentShapeUse::new(
                ComponentFieldName::new("invalid field"),
                RustPath::from_macro_tokens_unchecked("gpui_table_component::TextFilter"),
            ),
            RegistryFilterType::Text,
            RustPath::from_macro_tokens_unchecked(
                "gpui_table::runtime::generated_filters::text_filter::TextFilter",
            ),
        )];

        let shape = GpuiTableShape::new(
            "User",
            "users",
            "Users",
            false,
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

    #[test]
    fn parts_cover_plain_table_generation_contract() {
        let shape = shape(false, false, false, "demo-crate/src/user.rs");
        let adapter = TableShapeAdapter::new(&shape, false);
        let parts = adapter.try_parts().unwrap();

        assert_eq!(parts.struct_name_ident.to_string(), "User");
        assert_eq!(parts.story_struct_ident.to_string(), "UserTableStory");
        assert_eq!(parts.delegate_struct_ident.to_string(), "UserTableDelegate");
        let source_module_path = &parts.source_module_path;
        assert_eq!(
            quote!(#source_module_path).to_string(),
            "demo_crate :: user"
        );
        assert!(!parts.has_filters);
        assert!(!parts.load_more);
        assert!(
            parts
                .imports
                .to_string()
                .contains("use demo_crate :: user :: *")
        );
        assert!(
            !adapter
                .required_imports()
                .try_to_token_stream()
                .unwrap()
                .to_string()
                .contains("h_flex")
        );
        assert!(
            parts
                .delegate_creation
                .to_string()
                .contains("UserTableDelegate :: new")
        );
        assert!(
            parts
                .table_state_creation
                .to_string()
                .contains("TableState :: new")
        );
        assert!(!parts.table_state_creation.to_string().contains("load_data"));
        assert!(!parts.field_initializers.to_string().contains("filters"));
        assert!(!parts.struct_fields.to_string().contains("filters"));
        assert!(
            parts
                .render_children
                .to_string()
                .contains("TableStatusBar :: new")
        );
        assert!(
            parts
                .render_children
                .to_string()
                .contains("DataTable :: new")
        );
        assert_eq!(parts.title_expr.to_string(), "\"Users\" . to_string ()");
    }

    #[test]
    fn table_state_generation_covers_load_and_filter_combinations() {
        let plain_loading = shape(false, true, false, "demo/src/user.rs");
        let plain_tokens = TableShapeAdapter::new(&plain_loading, false)
            .table_state_creation()
            .to_string();
        assert!(plain_tokens.contains("load_data"));

        let filtered = shape(true, false, false, "demo/src/user.rs");
        let filtered_adapter = TableShapeAdapter::new(&filtered, false);
        let filtered_tokens = filtered_adapter.table_state_creation().to_string();
        assert!(filtered_tokens.contains("build_for_table"));
        assert!(!filtered_tokens.contains("build_for_table_loader"));
        assert!(
            filtered_adapter
                .required_imports()
                .try_to_token_stream()
                .unwrap()
                .to_string()
                .contains("h_flex")
        );

        let filtered_loading = shape(true, true, false, "demo/src/user.rs");
        let loading_tokens = TableShapeAdapter::new(&filtered_loading, false)
            .table_state_creation()
            .to_string();
        assert!(loading_tokens.contains("build_for_table_loader"));
    }

    #[test]
    fn render_generation_supports_helpers_and_explicit_filter_entities() {
        let shape = shape(true, false, false, "demo/src/user.rs");
        let helper_adapter = TableShapeAdapter::new(&shape, true);
        let explicit_adapter = TableShapeAdapter::new(&shape, false);

        assert!(
            helper_adapter
                .render_children()
                .to_string()
                .contains("all_filters")
        );
        let explicit = explicit_adapter.render_children().to_string();
        assert!(explicit.contains("self . filters . name . clone"));
        assert!(!explicit.contains("all_filters"));

        assert!(
            helper_adapter
                .field_initializers()
                .to_string()
                .contains("filters")
        );
        assert!(
            helper_adapter
                .struct_fields()
                .to_string()
                .contains("UserFilterEntities")
        );
        assert!(
            helper_adapter
                .delegate_creation()
                .to_string()
                .contains("UserTableDelegate")
        );
    }

    #[test]
    fn fluent_titles_and_layout_entry_points_are_generated() {
        let shape = shape(true, false, true, "demo/src/user.rs");
        let adapter = TableShapeAdapter::new(&shape, true);

        assert!(
            adapter
                .title_expr()
                .to_string()
                .contains("localize_label :: < User >")
        );

        let direct = adapter.try_generate_file(&MinimalLayout).unwrap();
        assert_eq!(quote!(#direct).to_string(), "pub struct UserTableStory ;");
        let infallible = adapter.generate_file(&MinimalLayout);
        assert_eq!(
            quote!(#infallible).to_string(),
            "pub struct UserTableStory ;"
        );

        let parts = adapter.parts();
        assert!(parts.has_filters);
    }

    #[test]
    fn invalid_struct_and_source_metadata_return_typed_errors() {
        let invalid_struct = GpuiTableShape::new(
            "invalid name",
            "invalid",
            "Invalid",
            false,
            &[],
            &[],
            false,
            "demo/src/invalid.rs",
        );
        assert!(matches!(
            TableShapeAdapter::new(&invalid_struct, false).try_parts(),
            Err(TableCodegenError::InvalidIdentifier {
                kind: "struct identifier",
                ..
            })
        ));

        let invalid_source = shape(false, false, false, "src/user.rs");
        assert!(matches!(
            TableShapeAdapter::new(&invalid_source, false).try_parts(),
            Err(TableCodegenError::InvalidSourcePath { source_path })
                if source_path == "src/user.rs"
        ));
    }
}
