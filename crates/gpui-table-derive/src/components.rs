use darling::FromMeta;
use syn::Path;

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

/// Options for number range filter
#[derive(Clone, Debug, Default, FromMeta)]
#[darling(default)]
pub struct NumberRangeFilterOptions {
    /// Minimum value for the range
    #[darling(default)]
    pub min: Option<f64>,
    /// Maximum value for the range
    #[darling(default)]
    pub max: Option<f64>,
    /// Step size for increment/decrement
    #[darling(default)]
    pub step: Option<f64>,
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
}

impl FilterComponents {
    /// Check if this is a faceted filter
    pub fn is_faceted(&self) -> bool {
        matches!(self, FilterComponents::Faceted(_))
    }
    
    /// Get text filter options if this is a text filter
    pub fn text_options(&self) -> Option<&TextFilterOptions> {
        match self {
            FilterComponents::Text(opts) => Some(opts),
            _ => None,
        }
    }
    
    /// Get faceted filter options if this is a faceted filter
    pub fn faceted_options(&self) -> Option<&FacetedFilterOptions> {
        match self {
            FilterComponents::Faceted(opts) => Some(opts),
            _ => None,
        }
    }
}
