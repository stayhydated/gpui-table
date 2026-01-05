//! Filter value wrapper types.

use derive_more::{Deref, DerefMut, From, Into};

use super::value::FilterValue;

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
