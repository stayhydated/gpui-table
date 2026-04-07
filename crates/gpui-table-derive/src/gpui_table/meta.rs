use crate::components::FilterComponents;

use darling::{FromDeriveInput, FromField, util::Override};
use syn::Ident;

#[derive(FromDeriveInput)]
#[darling(attributes(gpui_table), supports(struct_named))]
pub(super) struct TableMeta {
    pub(super) ident: Ident,
    pub(super) data: darling::ast::Data<darling::util::Ignored, TableColumn>,

    #[darling(default)]
    pub(super) id: Option<String>,
    #[darling(default)]
    pub(super) title: Option<String>,

    #[darling(default = "default_delegate")]
    pub(super) delegate: bool,

    #[darling(default)]
    pub(super) custom_style: Option<Override<bool>>,

    #[darling(default)]
    pub(super) custom_context_menu: Option<Override<bool>>,

    /// Generates a default context-menu link entry using this field as row id.
    /// Must be used together with `context_menu_route`.
    #[darling(default)]
    pub(super) context_menu_row_id: Option<String>,

    /// Route template used for generated row context-menu link.
    /// Must contain `{id}` placeholder and be used with `context_menu_row_id`.
    #[darling(default)]
    pub(super) context_menu_route: Option<String>,

    /// Optional menu label for generated context-menu route item.
    #[darling(default)]
    pub(super) context_menu_label: Option<String>,

    /// Function path used to build a route string at runtime.
    /// Signature should be compatible with `fn(&T) -> impl ToString`.
    #[darling(default)]
    pub(super) context_menu_route_fn: Option<syn::Path>,

    /// Function path used to build a label string at runtime.
    /// Signature should be compatible with `fn(&T) -> impl ToString`.
    #[darling(default)]
    pub(super) context_menu_label_fn: Option<syn::Path>,

    #[darling(default)]
    pub(super) fluent: Option<Override<String>>,

    #[darling(default)]
    pub(super) loading: Option<Ident>,

    /// Enable load_more wiring. When set, the generated delegate delegates
    /// has_more/load_more/threshold to #[gpui_table_impl].
    #[darling(default)]
    pub(super) load_more: bool,

    /// Enable filter support. When set, generates FilterEntities, FilterValues,
    /// and matches_filters() method. Field-level `filter(...)` attributes are
    /// only processed when this is enabled.
    #[darling(default)]
    pub(super) filters: bool,
}

fn default_delegate() -> bool {
    true
}

#[derive(FromField)]
#[darling(attributes(gpui_table))]
pub(super) struct TableColumn {
    pub(super) ident: Option<Ident>,
    pub(super) ty: syn::Type,

    #[darling(default)]
    pub(super) col: Option<String>,
    #[darling(default)]
    pub(super) title: Option<String>,
    #[darling(default)]
    pub(super) width: Option<f32>,
    #[darling(default)]
    pub(super) fixed: Option<String>,
    #[darling(default)]
    pub(super) sortable: bool,
    #[darling(default)]
    pub(super) ascending: bool,
    #[darling(default)]
    pub(super) descending: bool,
    #[darling(default)]
    pub(super) text_right: bool,
    #[darling(default)]
    pub(super) resizable: Option<bool>,
    #[darling(default)]
    pub(super) movable: Option<bool>,
    #[darling(default)]
    pub(super) skip: bool,
    /// Filter component configuration using function-style syntax.
    /// Examples: `filter = text()`, `filter = number_range(min = 0, max = 100)`
    #[darling(default)]
    pub(super) filter: Option<FilterComponents>,

    /// Marks this field as the value source for generated row context-menu route/label.
    #[darling(default)]
    pub(super) context_menu_id: bool,
}

/// Filter field metadata for delegate generation.
#[derive(Clone)]
pub(super) struct FilterFieldMeta {
    /// The field name identifier
    pub(super) field_ident: Ident,
    /// The filter component configuration
    pub(super) filter_config: FilterComponents,
    /// The field type (e.g., String, bool, Priority enum, chrono::DateTime)
    pub(super) field_type: syn::Type,
}
