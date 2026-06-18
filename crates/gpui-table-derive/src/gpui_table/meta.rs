use crate::components::{FilterShapeOptions, ResolvedFilterShape};

use component_shape_codegen::parse_single_shape_path;
use darling::{Error as DarlingError, FromDeriveInput, FromField, FromMeta, util::Override};
use koruma_derive_core::{ParsedDataField, ParsedValidatorUse, ValidatorTargetSelector};
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

    #[darling(default)]
    pub(super) mcp: Option<McpToolOptions>,
}

fn default_delegate() -> bool {
    true
}

#[derive(Clone, Debug, Default)]
pub(super) struct McpToolOptions {
    pub(super) name: Option<String>,
    pub(super) title: Option<String>,
    pub(super) description: Option<String>,
    #[allow(dead_code)]
    pub(super) row_schema: bool,
    #[allow(dead_code)]
    pub(super) read_only: Option<bool>,
    #[allow(dead_code)]
    pub(super) destructive: Option<bool>,
    #[allow(dead_code)]
    pub(super) idempotent: Option<bool>,
    #[allow(dead_code)]
    pub(super) open_world: Option<bool>,
}

#[derive(Clone, Debug, Default, darling::FromMeta)]
struct McpToolOptionsMeta {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    title: Option<String>,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    row_schema: bool,
    #[darling(default)]
    read_only: Option<bool>,
    #[darling(default)]
    destructive: Option<bool>,
    #[darling(default)]
    idempotent: Option<bool>,
    #[darling(default)]
    open_world: Option<bool>,
}

impl FromMeta for McpToolOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let options: Self = McpToolOptionsMeta::from_list(items)?.into();
        options
            .validate(proc_macro2::Span::call_site())
            .map_err(|error| DarlingError::custom(error.to_string()))?;
        Ok(options)
    }
}

impl From<McpToolOptionsMeta> for McpToolOptions {
    fn from(value: McpToolOptionsMeta) -> Self {
        Self {
            name: value.name,
            title: value.title,
            description: value.description,
            row_schema: value.row_schema,
            read_only: value.read_only,
            destructive: value.destructive,
            idempotent: value.idempotent,
            open_world: value.open_world,
        }
    }
}

impl McpToolOptions {
    pub(super) fn validate(&self, span: proc_macro2::Span) -> syn::Result<()> {
        if self.read_only == Some(true) && self.destructive == Some(true) {
            return Err(syn::Error::new(
                span,
                "MCP tool annotation hints cannot be both read-only and destructive",
            ));
        }
        if let Some(name) = self.name.as_deref() {
            component_shape::validate_mcp_tool_name(name)
                .map_err(|error| syn::Error::new(span, error.to_string()))?;
        }
        if let Some(title) = self.title.as_deref() {
            component_shape::validate_mcp_tool_metadata_text("title", title)
                .map_err(|error| syn::Error::new(span, error.to_string()))?;
        }
        if let Some(description) = self.description.as_deref() {
            component_shape::validate_mcp_tool_metadata_text("description", description)
                .map_err(|error| syn::Error::new(span, error.to_string()))?;
        }
        Ok(())
    }
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
    /// Explicit filter shape path.
    /// Example: `filter(gpui_table_component::TextFilter)`
    pub(super) filter: Option<FilterShapeOptions>,
    /// Koruma validators applied to the decoded MCP filter argument.
    pub(super) validation: Option<FilterValidation>,

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
            validation: None,
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

        if column.filter.is_some() {
            column.validation = parse_filter_validation(field)?;
        }

        Ok(column)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "mcp"), allow(dead_code))]
pub(super) struct FilterValidation {
    validators: Vec<ParsedValidatorUse>,
    newtype: bool,
}

#[cfg_attr(not(feature = "mcp"), allow(dead_code))]
impl FilterValidation {
    pub(super) fn new(validators: Vec<ParsedValidatorUse>, newtype: bool) -> Self {
        Self {
            validators,
            newtype,
        }
    }

    pub(super) fn validators(&self) -> &[ParsedValidatorUse] {
        &self.validators
    }

    pub(super) fn is_newtype(&self) -> bool {
        self.newtype
    }

    pub(super) fn is_empty(&self) -> bool {
        self.validators.is_empty() && !self.newtype
    }
}

fn parse_filter_validation(field: &syn::Field) -> darling::Result<Option<FilterValidation>> {
    match koruma_derive_core::parse_field(field, 0).map_err(DarlingError::from)? {
        ParsedDataField::Participating(info) => {
            if info.is_nested() {
                let span = info.marker_span().unwrap_or_else(|| field.span());
                return Err(DarlingError::from(syn::Error::new(
                    span,
                    "`#[koruma(nested)]` is not supported on table MCP filters; validate filter raw values with direct validators",
                )));
            }
            if !info.element_validators().is_empty() {
                let span = info
                    .element_validators()
                    .first()
                    .map(ParsedValidatorUse::source_span)
                    .unwrap_or_else(|| field.span());
                return Err(DarlingError::from(syn::Error::new(
                    span,
                    "`#[koruma(each(...))]` is not supported on table MCP filters; attach a collection validator to the filter raw value instead",
                )));
            }

            let mut validators = Vec::new();
            for validator in info.field_validators() {
                if matches!(
                    validator.target(),
                    ValidatorTargetSelector::Unwrapped { .. }
                ) {
                    return Err(DarlingError::from(syn::Error::new(
                        validator.source_span(),
                        "`#[koruma(unwrapped(...))]` is not supported on table MCP filters; validators run against the decoded filter raw value",
                    )));
                }
                validators.push(validator.clone());
            }

            let newtype = info.is_newtype();
            if validators.is_empty() && !newtype {
                Ok(None)
            } else {
                Ok(Some(FilterValidation::new(validators, newtype)))
            }
        },
        ParsedDataField::Unannotated(_) | ParsedDataField::Skipped { .. } => Ok(None),
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
    if meta.input.is_empty() {
        return Err(meta.error(
            "expected explicit filter shape, e.g. `filter(gpui_table_component::TextFilter)`",
        ));
    }

    let content;
    parenthesized!(content in meta.input);
    let shape = parse_single_shape_path(&content, "expected exactly one filter shape path")?;
    let span = shape.span();
    Ok(FilterShapeOptions::from_shape_with_span(shape, span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_options_accept_word_form() {
        let options = McpToolOptions::from_word().expect("mcp word form should parse");

        assert!(options.name.is_none());
        assert!(options.title.is_none());
        assert!(options.description.is_none());
        assert!(!options.row_schema);
        assert!(options.read_only.is_none());
        assert!(options.destructive.is_none());
        assert!(options.idempotent.is_none());
        assert!(options.open_world.is_none());
    }

    #[test]
    fn mcp_tool_options_accept_metadata_and_annotation_hints() {
        let options = McpToolOptions::from_list(&[
            darling::ast::NestedMeta::Meta(syn::parse_quote!(name = "query_users")),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(title = "Query users")),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(description = "Query users.")),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(row_schema)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(read_only = true)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(destructive = false)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(idempotent = true)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(open_world = true)),
        ])
        .expect("mcp list form should parse");

        assert_eq!(options.name.as_deref(), Some("query_users"));
        assert_eq!(options.title.as_deref(), Some("Query users"));
        assert_eq!(options.description.as_deref(), Some("Query users."));
        assert!(options.row_schema);
        assert_eq!(options.read_only, Some(true));
        assert_eq!(options.destructive, Some(false));
        assert_eq!(options.idempotent, Some(true));
        assert_eq!(options.open_world, Some(true));
    }

    #[test]
    fn mcp_tool_options_parses_row_schema() {
        let options = McpToolOptions::from_list(&[darling::ast::NestedMeta::Meta(
            syn::parse_quote!(row_schema),
        )])
        .expect("row_schema option should parse");

        assert!(options.row_schema);
    }

    #[test]
    fn mcp_tool_options_rejects_duplicate_row_schema() {
        let error = McpToolOptions::from_list(&[
            darling::ast::NestedMeta::Meta(syn::parse_quote!(row_schema)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(row_schema)),
        ])
        .expect_err("duplicate row_schema option should fail");

        assert!(
            error.to_string().contains("row_schema"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn mcp_tool_options_reject_invalid_name() {
        let error = McpToolOptions::from_list(&[darling::ast::NestedMeta::Meta(
            syn::parse_quote!(name = "bad name"),
        )])
        .expect_err("invalid name should fail");

        assert!(
            error.to_string().contains("tool name may only contain"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn mcp_tool_options_reject_empty_title_and_description() {
        let error = McpToolOptions::from_list(&[
            darling::ast::NestedMeta::Meta(syn::parse_quote!(title = "  ")),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(name = "users")),
        ])
        .expect_err("blank title should fail");
        assert!(
            error.to_string().contains("tool title cannot be empty"),
            "unexpected error: {error}"
        );

        let error = McpToolOptions::from_list(&[
            darling::ast::NestedMeta::Meta(syn::parse_quote!(description = "")),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(name = "users")),
        ])
        .expect_err("blank description should fail");
        assert!(
            error
                .to_string()
                .contains("tool description cannot be empty"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn mcp_tool_options_reject_conflicting_annotation_hints() {
        let error = McpToolOptions::from_list(&[
            darling::ast::NestedMeta::Meta(syn::parse_quote!(read_only = true)),
            darling::ast::NestedMeta::Meta(syn::parse_quote!(destructive = true)),
        ])
        .expect_err("conflicting annotation hints should fail");

        assert!(
            error
                .to_string()
                .contains("cannot be both read-only and destructive"),
            "unexpected error: {error}"
        );
    }
}

/// Filter field metadata for delegate generation.
#[derive(Clone)]
pub(super) struct FilterFieldMeta {
    /// The field name identifier
    pub(super) field_ident: Ident,
    /// The resolved filter shape configuration.
    pub(super) filter_config: ResolvedFilterShape,
    /// Koruma validators applied to decoded MCP filter arguments.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub(super) validation: Option<FilterValidation>,
}
