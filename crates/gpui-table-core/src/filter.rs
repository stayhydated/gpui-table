#[derive(Clone)]
pub struct FacetedFilterOption {
    pub label: String,
    pub value: String,
    pub count: Option<usize>,
    pub icon: Option<gpui_component::IconName>,
}

impl std::fmt::Debug for FacetedFilterOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FacetedFilterOption")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("count", &self.count)
            .field(
                "icon",
                &if self.icon.is_some() {
                    "Some(IconName)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

impl PartialEq for FacetedFilterOption {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.value == other.value && self.count == other.count
        // Ignore icon for equality as IconName doesn't implement PartialEq
    }
}

impl Eq for FacetedFilterOption {}

#[derive(Clone, Debug)]
pub struct FilterConfig {
    pub column_index: usize,
    pub filter_type: FilterType,
}

#[derive(Clone, Debug)]
pub enum FilterType {
    Faceted(Vec<FacetedFilterOption>),
    #[cfg(feature = "chrono")]
    DateRange,
    NumberRange,
    Text,
}

/// Trait for types that can be used as filter values in a `HashSet<T>`.
///
/// This trait enables storing typed values in the faceted filter instead of strings.
/// Implementors must provide conversion to/from the string representation used
/// in `FacetedFilterOption::value`.
pub trait FilterValue: Clone + Eq + std::hash::Hash + Send + 'static {
    /// Convert the value to its string representation for matching with options.
    fn to_filter_string(&self) -> String;

    /// Parse a string back into the typed value.
    /// Returns `None` if the string doesn't represent a valid value.
    fn from_filter_string(s: &str) -> Option<Self>;
}

pub trait Filterable: FilterValue {
    fn options() -> Vec<FacetedFilterOption>;
}

impl FilterValue for bool {
    fn to_filter_string(&self) -> String {
        if *self { "true" } else { "false" }.to_string()
    }

    fn from_filter_string(s: &str) -> Option<Self> {
        match s {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}

impl Filterable for bool {
    fn options() -> Vec<FacetedFilterOption> {
        vec![
            FacetedFilterOption {
                label: "True".to_string(),
                value: "true".to_string(),
                count: None,
                icon: None,
            },
            FacetedFilterOption {
                label: "False".to_string(),
                value: "false".to_string(),
                count: None,
                icon: None,
            },
        ]
    }
}

// ============================================================================
// Filter Value Wrapper Types
// ============================================================================

use derive_more::{Deref, DerefMut, From, Into};

/// A wrapper around `HashSet<T>` for faceted filter values with helper methods.
///
/// This type provides convenient methods for checking if a filter is active
/// and matching values against the filter.
#[derive(Clone, Debug, Deref, DerefMut, Eq, From, Into, PartialEq)]
pub struct FacetedValue<T: FilterValue>(pub std::collections::HashSet<T>);

impl<T: FilterValue> Default for FacetedValue<T> {
    fn default() -> Self {
        Self(std::collections::HashSet::new())
    }
}

impl<T: FilterValue> FacetedValue<T> {
    /// Create a new empty faceted value.
    pub fn new() -> Self {
        Self(std::collections::HashSet::new())
    }

    /// Check if this filter has any active selections.
    pub fn is_active(&self) -> bool {
        !self.0.is_empty()
    }

    /// Check if the given value matches this filter.
    /// Returns true if the filter is empty (no restrictions) or if the value is in the set.
    pub fn matches(&self, value: &T) -> bool {
        self.0.is_empty() || self.0.contains(value)
    }
}

/// A wrapper around `(Option<T>, Option<T>)` for range filter values with helper methods.
///
/// This type provides convenient methods for checking if a range filter is active
/// and matching values against the range.
#[derive(Clone, Debug, Default, From, Into, PartialEq)]
pub struct RangeValue<T: Clone + PartialOrd>(pub Option<T>, pub Option<T>);

impl<T: Clone + PartialOrd> RangeValue<T> {
    /// Create a new empty range (no restrictions).
    pub fn new() -> Self {
        Self(None, None)
    }

    /// Check if this range filter has any active bounds.
    pub fn is_active(&self) -> bool {
        self.0.is_some() || self.1.is_some()
    }

    /// Check if the given value is within this range.
    /// Returns true if no bounds are set (no restrictions) or if the value is within bounds.
    pub fn matches(&self, value: &T) -> bool {
        let min_ok = self.0.as_ref().map_or(true, |min| value >= min);
        let max_ok = self.1.as_ref().map_or(true, |max| value <= max);
        min_ok && max_ok
    }

    /// Get the minimum bound.
    pub fn min(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Get the maximum bound.
    pub fn max(&self) -> Option<&T> {
        self.1.as_ref()
    }
}

/// A wrapper around `String` for text filter values with helper methods.
///
/// This type provides convenient methods for checking if a text filter is active
/// and matching values against the filter.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Deref,
    DerefMut,
    From,
    Into,
    derive_more::Display,
    derive_more::AsRef,
)]
#[as_ref(str)]
pub struct TextValue(pub String);

impl TextValue {
    /// Create a new empty text value.
    pub fn new() -> Self {
        Self(String::new())
    }

    /// Check if this text filter is active (non-empty).
    pub fn is_active(&self) -> bool {
        !self.0.is_empty()
    }

    /// Check if the given value contains this filter text (case-insensitive).
    /// Returns true if the filter is empty (no restrictions) or if the value contains the filter.
    pub fn matches(&self, value: &str) -> bool {
        self.0.is_empty() || value.to_lowercase().contains(&self.0.to_lowercase())
    }
}

impl From<&str> for TextValue {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ============================================================================
// Conversion Traits for Filter Matching
// ============================================================================

/// Trait for converting numeric types to Decimal for range filter matching.
#[cfg(feature = "rust_decimal")]
pub trait IntoDecimal {
    fn into_decimal(&self) -> rust_decimal::Decimal;
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for rust_decimal::Decimal {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        *self
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for f64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f64_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for f32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from_f32_retain(*self).unwrap_or(rust_decimal::Decimal::ZERO)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i8 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as i32)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i16 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as i32)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for i64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u8 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as u32)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u16 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as u32)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u32 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for u64 {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for usize {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as u64)
    }
}

#[cfg(feature = "rust_decimal")]
impl IntoDecimal for isize {
    fn into_decimal(&self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(*self as i64)
    }
}

/// Trait for converting date/time types to NaiveDate for range filter matching.
#[cfg(feature = "chrono")]
pub trait IntoNaiveDate {
    fn into_naive_date(&self) -> chrono::NaiveDate;
}

#[cfg(feature = "chrono")]
impl IntoNaiveDate for chrono::NaiveDate {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        *self
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> IntoNaiveDate for chrono::DateTime<Tz> {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        self.date_naive()
    }
}

#[cfg(feature = "chrono")]
impl IntoNaiveDate for chrono::NaiveDateTime {
    fn into_naive_date(&self) -> chrono::NaiveDate {
        self.date()
    }
}
