use crate::components::{FilterShapeOptions, ResolvedFilterShape};

use component_shape_codegen::parse_single_shape_path;
use darling::{Error as DarlingError, FromDeriveInput, FromField, util::Override};
use syn::{Ident, LitBool, LitFloat, LitInt, LitStr, Token, parenthesized, spanned::Spanned as _};

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
    /// Must be paired with `context_menu_route`.
    #[darling(default)]
    pub(super) context_menu_row_id: Option<String>,

    /// Route template for generated row context-menu link.
    /// Must contain `{id}` placeholder and be paired with `context_menu_row_id`.
    #[darling(default)]
    pub(super) context_menu_route: Option<String>,

    /// Optional menu label for generated context-menu route item.
    #[darling(default)]
    pub(super) context_menu_label: Option<String>,

    /// Function path that builds a route string at runtime.
    /// Signature should match `fn(&T) -> impl ToString`.
    #[darling(default)]
    pub(super) context_menu_route_fn: Option<syn::Path>,

    /// Function path that builds a label string at runtime.
    /// Signature should match `fn(&T) -> impl ToString`.
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

pub(super) struct TableColumn {
    pub(super) ident: Option<Ident>,
    pub(super) ty: syn::Type,

    pub(super) col: Option<String>,
    pub(super) title: Option<String>,
    pub(super) width: Option<f32>,
    pub(super) fixed: Option<String>,
    pub(super) sortable: bool,
    pub(super) ascending: bool,
    pub(super) descending: bool,
    pub(super) text_right: bool,
    pub(super) resizable: Option<bool>,
    pub(super) movable: Option<bool>,
    pub(super) skip: bool,
    /// Filter shape path.
    /// Example: `filter(gpui_table_component::TextFilter)`
    pub(super) filter: Option<FilterShapeOptions>,

    /// Marks this field as the value source for generated row context-menu route/label.
    pub(super) context_menu_id: bool,
}

impl FromField for TableColumn {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut column = Self {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            col: None,
            title: None,
            width: None,
            fixed: None,
            sortable: false,
            ascending: false,
            descending: false,
            text_right: false,
            resizable: None,
            movable: None,
            skip: false,
            filter: None,
            context_menu_id: false,
        };

        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("gpui_table"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("col") {
                    set_option(
                        &mut column.col,
                        parse_string_value(&meta)?,
                        "col",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("title") {
                    set_option(
                        &mut column.title,
                        parse_string_value(&meta)?,
                        "title",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("width") {
                    set_option(
                        &mut column.width,
                        parse_f32_value(&meta)?,
                        "width",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("fixed") {
                    set_option(
                        &mut column.fixed,
                        parse_string_value(&meta)?,
                        "fixed",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("sortable") {
                    set_flag(
                        &mut column.sortable,
                        parse_bool_flag_or_value(&meta)?,
                        "sortable",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("ascending") {
                    set_flag(
                        &mut column.ascending,
                        parse_bool_flag_or_value(&meta)?,
                        "ascending",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("descending") {
                    set_flag(
                        &mut column.descending,
                        parse_bool_flag_or_value(&meta)?,
                        "descending",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("text_right") {
                    set_flag(
                        &mut column.text_right,
                        parse_bool_flag_or_value(&meta)?,
                        "text_right",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("resizable") {
                    set_option(
                        &mut column.resizable,
                        parse_bool_flag_or_value(&meta)?,
                        "resizable",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("movable") {
                    set_option(
                        &mut column.movable,
                        parse_bool_flag_or_value(&meta)?,
                        "movable",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("skip") {
                    set_flag(
                        &mut column.skip,
                        parse_bool_flag_or_value(&meta)?,
                        "skip",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("filter") {
                    set_option(
                        &mut column.filter,
                        parse_filter_shape(&meta)?,
                        "filter",
                        meta.path.span(),
                    )
                } else if meta.path.is_ident("context_menu_id") {
                    set_flag(
                        &mut column.context_menu_id,
                        parse_bool_flag_or_value(&meta)?,
                        "context_menu_id",
                        meta.path.span(),
                    )
                } else {
                    Err(meta.error("unknown `gpui_table` field option"))
                }
            })
            .map_err(DarlingError::from)?;
        }

        Ok(column)
    }
}

fn set_option<T>(
    slot: &mut Option<T>,
    value: T,
    option_name: &'static str,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(
            span,
            format!("duplicate `{option_name}` option"),
        ));
    }
    *slot = Some(value);
    Ok(())
}

fn set_flag(
    slot: &mut bool,
    value: bool,
    option_name: &'static str,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    if *slot {
        return Err(syn::Error::new(
            span,
            format!("duplicate `{option_name}` option"),
        ));
    }
    *slot = value;
    Ok(())
}

fn parse_string_value(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    Ok(meta.value()?.parse::<LitStr>()?.value())
}

fn parse_f32_value(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<f32> {
    let value = meta.value()?;
    if value.peek(LitFloat) {
        value.parse::<LitFloat>()?.base10_parse::<f32>()
    } else if value.peek(LitInt) {
        value.parse::<LitInt>()?.base10_parse::<f32>()
    } else {
        Err(meta.error("expected a numeric literal"))
    }
}

fn parse_bool_flag_or_value(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.input.peek(Token![=]) {
        Ok(meta.value()?.parse::<LitBool>()?.value)
    } else {
        Ok(true)
    }
}

fn parse_filter_shape(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<FilterShapeOptions> {
    let content;
    parenthesized!(content in meta.input);
    let shape = parse_single_shape_path(&content, "expected exactly one filter shape path")?;
    let span = shape.span();
    Ok(FilterShapeOptions::from_shape_with_span(shape, span))
}

/// Filter field metadata for delegate generation.
#[derive(Clone)]
pub(super) struct FilterFieldMeta {
    /// The field name identifier
    pub(super) field_ident: Ident,
    /// The resolved filter shape configuration.
    pub(super) filter_config: ResolvedFilterShape,
}
