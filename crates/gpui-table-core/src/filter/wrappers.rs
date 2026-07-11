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

/// A wrapper around `Option<T>` for exact-match select filter values with helper methods.
#[derive(Clone, Debug, Eq, From, Into, PartialEq)]
pub struct SingleValue<T: Clone + PartialEq>(pub Option<T>);

impl<T: Clone + PartialEq> Default for SingleValue<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Clone + PartialEq> SingleValue<T> {
    /// Create a new empty single-value filter (no restrictions).
    pub fn new() -> Self {
        Self(None)
    }

    /// Check if this filter has an active selection.
    pub fn is_active(&self) -> bool {
        self.0.is_some()
    }

    /// Get the selected value, if any.
    pub fn value(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Iterate over the selected value as zero or one items.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Return the number of selected values.
    pub fn len(&self) -> usize {
        usize::from(self.0.is_some())
    }

    /// Return whether no value is selected.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the given value matches this filter.
    /// Returns true if no value is selected or if the value equals the selection.
    pub fn matches(&self, value: &T) -> bool {
        self.0.as_ref().is_none_or(|selected| selected == value)
    }
}

/// A wrapper around `(Option<T>, Option<T>)` for range filter values with helper methods.
///
/// This type provides convenient methods for checking if a range filter is active
/// and matching values against the range.
#[derive(Clone, Debug, Eq, From, Into, PartialEq)]
pub struct RangeValue<T: Clone + PartialOrd>(pub Option<T>, pub Option<T>);

impl<T: Clone + PartialOrd> Default for RangeValue<T> {
    fn default() -> Self {
        Self(None, None)
    }
}

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
        let min_ok = self.0.as_ref().is_none_or(|min| value >= min);
        let max_ok = self.1.as_ref().is_none_or(|max| value <= max);
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

#[cfg(test)]
mod tests {
    use super::{FacetedValue, RangeValue, SingleValue, TextValue};
    use std::collections::HashSet;

    #[test]
    fn faceted_values_match_everything_until_a_selection_is_active() {
        let mut value = FacetedValue::<bool>::new();
        assert_eq!(value, FacetedValue::default());
        assert!(!value.is_active());
        assert!(value.matches(&true));

        value.insert(true);
        assert!(value.is_active());
        assert!(value.matches(&true));
        assert!(!value.matches(&false));

        let inner: HashSet<_> = value.into();
        assert_eq!(inner, HashSet::from([true]));
    }

    #[test]
    fn single_values_expose_zero_or_one_exact_selection() {
        let empty = SingleValue::<String>::new();
        assert_eq!(empty, SingleValue::default());
        assert!(!empty.is_active());
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.value(), None);
        assert_eq!(empty.iter().count(), 0);
        assert!(empty.matches(&"anything".to_string()));

        let selected = SingleValue(Some("chosen".to_string()));
        assert!(selected.is_active());
        assert!(!selected.is_empty());
        assert_eq!(selected.len(), 1);
        assert_eq!(selected.value().map(String::as_str), Some("chosen"));
        assert_eq!(
            selected.iter().map(String::as_str).collect::<Vec<_>>(),
            ["chosen"]
        );
        assert!(selected.matches(&"chosen".to_string()));
        assert!(!selected.matches(&"other".to_string()));
    }

    #[test]
    fn range_values_apply_each_bound_inclusively() {
        let empty = RangeValue::<i32>::new();
        assert_eq!(empty, RangeValue::default());
        assert!(!empty.is_active());
        assert!(empty.matches(&10));
        assert_eq!(empty.min(), None);
        assert_eq!(empty.max(), None);

        let min_only = RangeValue(Some(10), None);
        assert!(min_only.is_active());
        assert!(min_only.matches(&10));
        assert!(!min_only.matches(&9));
        assert_eq!(min_only.min(), Some(&10));

        let max_only = RangeValue(None, Some(20));
        assert!(max_only.matches(&20));
        assert!(!max_only.matches(&21));
        assert_eq!(max_only.max(), Some(&20));

        let bounded = RangeValue(Some(10), Some(20));
        assert!(bounded.matches(&10));
        assert!(bounded.matches(&15));
        assert!(bounded.matches(&20));
        assert!(!bounded.matches(&9));
        assert!(!bounded.matches(&21));
    }

    #[test]
    fn text_values_match_case_insensitive_substrings() {
        let mut value = TextValue::new();
        assert_eq!(value, TextValue::default());
        assert!(!value.is_active());
        assert!(value.matches("anything"));

        value.push_str("Rust");
        assert!(value.is_active());
        assert!(value.matches("Trustworthy"));
        assert!(!value.matches("tables"));
        assert_eq!(value.as_ref(), "Rust");
        assert_eq!(value.to_string(), "Rust");

        let from_str = TextValue::from("GPUI");
        let from_string = TextValue::from(String::from("GPUI"));
        assert_eq!(from_str, from_string);
        let inner: String = from_string.into();
        assert_eq!(inner, "GPUI");
    }
}
