use darling::{Error as DarlingError, FromMeta};
use quote::ToTokens as _;
#[cfg(feature = "rust_decimal")]
use quote::quote;
use syn::{Expr, Lit, Path, UnOp, spanned::Spanned as _};

/// Built-in text validation modes
#[derive(Clone, Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum TextValidation {
    /// Only allow alphabetic characters (a-z, A-Z)
    Alphabetic,
    /// Only allow numeric characters (0-9)
    Numeric,
    /// Only allow alphanumeric characters
    Alphanumeric,
    /// Custom validation function path
    #[darling(rename = "fn")]
    Custom(Path),
}

/// Options for text filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct TextFilterOptions {
    /// Validation mode for the text input
    #[darling(default)]
    pub validate: Option<TextValidation>,
}

/// A decimal literal captured from `number_range(...)` metadata.
///
/// The raw string is preserved so macro validation can point at the original
/// literal while codegen can lower the parsed value directly into a
/// `rust_decimal::Decimal` constructor.
#[derive(Clone, Debug)]
pub struct DecimalLiteral {
    raw: String,
    span: proc_macro2::Span,
}

impl DecimalLiteral {
    #[cfg(feature = "rust_decimal")]
    pub fn span(&self) -> proc_macro2::Span {
        self.span
    }

    #[cfg(feature = "rust_decimal")]
    pub fn parse_decimal(&self, key: &str) -> syn::Result<rust_decimal::Decimal> {
        rust_decimal::Decimal::from_str_exact(&self.raw).map_err(|_| {
            syn::Error::new(
                self.span,
                format!(
                    "invalid decimal literal `{}` for `number_range({key} = ...)`; use a plain decimal like `0.25` or a quoted decimal string like \"0.25\"",
                    self.raw
                ),
            )
        })
    }

    #[cfg(feature = "rust_decimal")]
    pub fn decimal_tokens(&self, key: &str) -> proc_macro2::TokenStream {
        let decimal = self
            .parse_decimal(key)
            .expect("number_range decimal literal should be validated before code generation");
        let mantissa = decimal.mantissa();
        let scale = decimal.scale();
        quote! {
            gpui_table::__deps::rust_decimal::Decimal::from_i128_with_scale(#mantissa, #scale)
        }
    }
}

impl FromMeta for DecimalLiteral {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        parse_decimal_literal(expr).map_err(|message| DarlingError::custom(message).with_span(expr))
    }
}

/// Options for number range filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct NumberRangeFilterOptions {
    /// Minimum value for the range
    #[darling(default)]
    pub min: Option<DecimalLiteral>,
    /// Maximum value for the range
    #[darling(default)]
    pub max: Option<DecimalLiteral>,
    /// Step size for increment/decrement
    #[darling(default)]
    pub step: Option<DecimalLiteral>,
}

/// Options for date range filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct DateRangeFilterOptions {}

/// Options for faceted filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct FacetedFilterOptions {
    /// Whether the filter should show a search input
    #[darling(default)]
    pub searchable: bool,
}

/// Options for infinite faceted filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct InfiniteFacetedFilterOptions {}

/// Filter component enum parsed from attributes.
/// Supports syntax like: `filter(text())` or `filter(number_range(min = 0, max = 100))`
#[derive(Clone, Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum FilterComponents {
    /// Text search filter with optional validation
    Text(TextFilterOptions),
    /// Numeric range filter with min/max bounds
    NumberRange(NumberRangeFilterOptions),
    /// Date range filter with start/end dates
    DateRange(DateRangeFilterOptions),
    /// Faceted filter with enumerated options
    Faceted(FacetedFilterOptions),
    /// Hierarchical faceted filter backed by InfiniteSelect
    #[darling(rename = "infinite_faceted_filter")]
    InfiniteFaceted(InfiniteFacetedFilterOptions),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FilterKind {
    Text,
    NumberRange,
    DateRange,
    Faceted,
    InfiniteFaceted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FilterRenderGroup {
    Text,
    NumberRange,
    Faceted,
    DateRange,
}

impl FilterKind {
    #[cfg(any(
        not(feature = "chrono"),
        not(feature = "rust_decimal"),
        not(feature = "spacetimedb")
    ))]
    pub(crate) fn attribute_syntax(self) -> &'static str {
        match self {
            Self::Text => "text()",
            Self::NumberRange => "number_range(...)",
            Self::DateRange => "date_range()",
            Self::Faceted => "faceted(...)",
            Self::InfiniteFaceted => "infinite_faceted_filter()",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::NumberRange => "number range",
            Self::DateRange => "date range",
            Self::Faceted => "faceted",
            Self::InfiniteFaceted => "infinite faceted",
        }
    }

    pub(crate) fn render_group(self) -> FilterRenderGroup {
        match self {
            Self::Text => FilterRenderGroup::Text,
            Self::NumberRange => FilterRenderGroup::NumberRange,
            Self::DateRange => FilterRenderGroup::DateRange,
            Self::Faceted | Self::InfiniteFaceted => FilterRenderGroup::Faceted,
        }
    }
}

impl FilterRenderGroup {
    pub(crate) const ALL: [Self; 4] = [
        Self::Text,
        Self::NumberRange,
        Self::Faceted,
        Self::DateRange,
    ];

    pub(crate) fn method_name(self) -> &'static str {
        match self {
            Self::Text => "text_filters",
            Self::NumberRange => "number_filters",
            Self::Faceted => "faceted_filters",
            Self::DateRange => "date_filters",
        }
    }

    pub(crate) fn doc_label(self) -> &'static str {
        match self {
            Self::Text => "text filters",
            Self::NumberRange => "number range filters",
            Self::Faceted => "faceted filters",
            Self::DateRange => "date range filters",
        }
    }
}

impl FilterComponents {
    pub(crate) fn kind(&self) -> FilterKind {
        match self {
            Self::Text(_) => FilterKind::Text,
            Self::NumberRange(_) => FilterKind::NumberRange,
            Self::DateRange(_) => FilterKind::DateRange,
            Self::Faceted(_) => FilterKind::Faceted,
            Self::InfiniteFaceted(_) => FilterKind::InfiniteFaceted,
        }
    }

    /// The attribute syntax users write for this built-in filter.
    #[cfg(any(
        not(feature = "chrono"),
        not(feature = "rust_decimal"),
        not(feature = "spacetimedb")
    ))]
    pub(crate) fn attribute_syntax(&self) -> &'static str {
        self.kind().attribute_syntax()
    }

    /// A short human-readable label for generated docs and diagnostics.
    pub(crate) fn kind_label(&self) -> &'static str {
        self.kind().label()
    }

    pub(crate) fn render_group(&self) -> FilterRenderGroup {
        self.kind().render_group()
    }
}

fn parse_decimal_literal(expr: &Expr) -> Result<DecimalLiteral, String> {
    match expr {
        Expr::Lit(expr_lit) => parse_decimal_lit(&expr_lit.lit),
        Expr::Group(group) => parse_decimal_literal(&group.expr),
        Expr::Unary(unary) => parse_signed_decimal_literal(unary),
        _ => {
            Err("expected an integer, float, or string literal in `number_range(...)`".to_string())
        },
    }
}

fn parse_signed_decimal_literal(unary: &syn::ExprUnary) -> Result<DecimalLiteral, String> {
    let prefix = match unary.op {
        UnOp::Neg(_) => "-",
        _ => {
            return Err(
                "expected an integer, float, or string literal in `number_range(...)`".to_string(),
            );
        },
    };

    match unary.expr.as_ref() {
        Expr::Lit(expr_lit) => {
            let mut literal = parse_decimal_lit(&expr_lit.lit)?;
            literal.raw = format!("{prefix}{}", literal.raw);
            literal.span = unary.span();
            Ok(literal)
        },
        Expr::Group(group) => {
            let mut literal = parse_decimal_literal(&group.expr)?;
            literal.raw = format!("{prefix}{}", literal.raw);
            literal.span = unary.span();
            Ok(literal)
        },
        _ => {
            Err("expected an integer, float, or string literal in `number_range(...)`".to_string())
        },
    }
}

fn parse_decimal_lit(lit: &Lit) -> Result<DecimalLiteral, String> {
    match lit {
        Lit::Int(lit_int) => {
            if !lit_int.suffix().is_empty() {
                return Err(
                    "number_range values cannot use numeric suffixes; use `1.25` or \"1.25\""
                        .to_string(),
                );
            }

            Ok(DecimalLiteral {
                raw: lit_int.base10_digits().to_string(),
                span: lit_int.span(),
            })
        },
        Lit::Float(lit_float) => {
            if !lit_float.suffix().is_empty() {
                return Err(
                    "number_range values cannot use numeric suffixes; use `1.25` or \"1.25\""
                        .to_string(),
                );
            }

            Ok(DecimalLiteral {
                raw: lit_float.to_token_stream().to_string().replace('_', ""),
                span: lit_float.span(),
            })
        },
        Lit::Str(lit_str) => Ok(DecimalLiteral {
            raw: lit_str.value(),
            span: lit_str.span(),
        }),
        _ => {
            Err("expected an integer, float, or string literal in `number_range(...)`".to_string())
        },
    }
}
