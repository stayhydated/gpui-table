use gpui_table_core::registry::GpuiTableShape;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::Path;
use thiserror::Error;

use crate::imports::{Alias, ImportItem, ImportSet};

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
const FILTER_IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui_component::h_flex"),
    ImportItem::aliased("gpui_table::filter::FilterEntitiesExt", Alias::Anonymous),
    ImportItem::aliased("gpui_table::filter::Matchable", Alias::Anonymous),
];

/// Trait for deriving various identifier names from a table shape.
pub trait TableIdentities {
    /// The original struct name (e.g., "User")
    fn struct_name(&self) -> &'static str;

    /// The struct name as an identifier.
    ///
    /// For user-facing tooling, prefer [`TableIdentitiesExt::try_struct_name_ident`].
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

    /// Snake case name as identifier (for import paths).
    ///
    /// For user-facing tooling, prefer [`TableIdentitiesExt::try_snake_case_ident`].
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

/// Fallible identifier helpers for user-facing tooling.
pub trait TableIdentitiesExt: TableIdentities {
    fn try_struct_name_ident(&self) -> Result<syn::Ident, TableCodegenError> {
        parse_ident(
            "struct identifier",
            self.struct_name(),
            self.struct_name().to_string(),
        )
    }

    fn try_snake_case_ident(&self) -> Result<syn::Ident, TableCodegenError> {
        let snake_case_name = self.snake_case_name();
        parse_ident("snake_case identifier", self.struct_name(), snake_case_name)
    }
}

impl<T: TableIdentities + ?Sized> TableIdentitiesExt for T {}

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
    pub fn columns(&self) -> &'static [gpui_table_core::registry::ColumnVariant] {
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
        let story_struct_ident = self.identities.story_struct_ident();
        let delegate_struct_ident = self.identities.delegate_struct_ident();

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

        let delegate_creation = self.delegate_creation();
        let table_state_creation = self.table_state_creation();
        let field_initializers = self.field_initializers();
        let struct_fields = self.struct_fields();
        let render_children = self.render_children();
        let title_expr = self.title_expr();

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

fn parse_ident(
    kind: &'static str,
    struct_name: &'static str,
    value: String,
) -> Result<syn::Ident, TableCodegenError> {
    syn::parse_str(&value).map_err(|source| TableCodegenError::InvalidIdentifier {
        kind,
        struct_name,
        value,
        source,
    })
}

// ── source_path_to_use_path ───────────────────────────────────────────────────

/// Converts a `file!()` source path like
/// `examples/some-lib/src/structs/user.rs` into a use-path like
/// `some_lib::structs::user` for the glob import at the top of each generated file.
pub fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let path = Path::new(source_path);
    let components: Vec<_> = path.components().collect();

    let src_index = components
        .iter()
        .position(|c| matches!(c, std::path::Component::Normal(s) if s.to_str() == Some("src")))?;

    if src_index == 0 {
        return None;
    }
    let crate_name = match &components[src_index - 1] {
        std::path::Component::Normal(s) => s.to_str()?.replace('-', "_"),
        _ => return None,
    };

    let mut path_segments = vec![crate_name];
    for component in &components[src_index + 1..] {
        if let std::path::Component::Normal(s) = component {
            let segment = s.to_str()?;
            if segment == "mod.rs" {
                continue;
            }
            path_segments.push(
                segment
                    .strip_suffix(".rs")
                    .unwrap_or(segment)
                    .replace('-', "_"),
            );
        }
    }

    syn::parse_str(&path_segments.join("::")).ok()
}

// ── TableShape impl ──────────────────────────────────────────────────────────

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
                DataTable::new(&self.table)
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
