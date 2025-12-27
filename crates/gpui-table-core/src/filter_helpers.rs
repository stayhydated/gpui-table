//! Helper functions for client-side filtering.
//!
//! These functions are used by the generated `matches_filters` method
//! to check if a row matches the current filter values.

use std::collections::HashSet;

/// Trait for types that can be checked against a number range filter.
pub trait NumberRangeMatch {
    /// Check if the value is within the given range.
    fn matches_range(&self, range: &(Option<f64>, Option<f64>)) -> bool;
}

// Implement for integer types
macro_rules! impl_number_range_match_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl NumberRangeMatch for $t {
                fn matches_range(&self, range: &(Option<f64>, Option<f64>)) -> bool {
                    let v = *self as f64;
                    match range {
                        (None, None) => true,
                        (Some(min), None) => v >= *min,
                        (None, Some(max)) => v <= *max,
                        (Some(min), Some(max)) => v >= *min && v <= *max,
                    }
                }
            }
        )*
    };
}

impl_number_range_match_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

// Implement for float types
impl NumberRangeMatch for f32 {
    fn matches_range(&self, range: &(Option<f64>, Option<f64>)) -> bool {
        let v = *self as f64;
        match range {
            (None, None) => true,
            (Some(min), None) => v >= *min,
            (None, Some(max)) => v <= *max,
            (Some(min), Some(max)) => v >= *min && v <= *max,
        }
    }
}

impl NumberRangeMatch for f64 {
    fn matches_range(&self, range: &(Option<f64>, Option<f64>)) -> bool {
        match range {
            (None, None) => true,
            (Some(min), None) => *self >= *min,
            (None, Some(max)) => *self <= *max,
            (Some(min), Some(max)) => *self >= *min && *self <= *max,
        }
    }
}

/// Check if a Decimal is within a range filter.
#[cfg(feature = "rust_decimal")]
impl NumberRangeMatch for rust_decimal::Decimal {
    fn matches_range(&self, range: &(Option<f64>, Option<f64>)) -> bool {
        use rust_decimal::prelude::ToPrimitive;
        let v: f64 = self.to_f64().unwrap_or(0.0);
        match range {
            (None, None) => true,
            (Some(min), None) => v >= *min,
            (None, Some(max)) => v <= *max,
            (Some(min), Some(max)) => v >= *min && v <= *max,
        }
    }
}

/// Check if a number is within a range filter.
///
/// The range is (min, max) where either can be None.
/// Returns true if the value is within the range or if the range is empty.
pub fn number_in_range<T: NumberRangeMatch>(value: &T, range: &(Option<f64>, Option<f64>)) -> bool {
    value.matches_range(range)
}

/// Check if a faceted value matches the filter set.
///
/// For enums and other types that implement Debug, the variant name is compared.
/// Returns true if the filter set is empty (no filter) or if the value is in the set.
pub fn facet_matches<T: std::fmt::Debug>(value: &T, filter: &HashSet<String>) -> bool {
    if filter.is_empty() {
        return true;
    }
    let value_str = format!("{:?}", value);
    filter.contains(&value_str)
}

/// Check if a boolean matches the faceted filter.
pub fn facet_matches_bool(value: &bool, filter: &HashSet<String>) -> bool {
    if filter.is_empty() {
        return true;
    }
    filter.contains(&value.to_string())
}

/// Check if a date is within a date range filter.
#[cfg(feature = "chrono")]
pub fn date_in_range<Tz: chrono::TimeZone>(
    value: &chrono::DateTime<Tz>,
    range: &(Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
) -> bool {
    let date = value.date_naive();
    match range {
        (None, None) => true,
        (Some(start), None) => date >= *start,
        (None, Some(end)) => date <= *end,
        (Some(start), Some(end)) => date >= *start && date <= *end,
    }
}

/// Check if a NaiveDate is within a date range filter.
#[cfg(feature = "chrono")]
pub fn naive_date_in_range(
    value: &chrono::NaiveDate,
    range: &(Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
) -> bool {
    match range {
        (None, None) => true,
        (Some(start), None) => *value >= *start,
        (None, Some(end)) => *value <= *end,
        (Some(start), Some(end)) => *value >= *start && *value <= *end,
    }
}

/// Check if a NaiveDateTime is within a date range filter.
#[cfg(feature = "chrono")]
pub fn naive_datetime_in_range(
    value: &chrono::NaiveDateTime,
    range: &(Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
) -> bool {
    let date = value.date();
    match range {
        (None, None) => true,
        (Some(start), None) => date >= *start,
        (None, Some(end)) => date <= *end,
        (Some(start), Some(end)) => date >= *start && date <= *end,
    }
}
