//! Filter value traits and implementations.

use super::config::FacetedFilterOption;
#[cfg(feature = "fluent")]
use es_fluent::{EsFluent, ToFluentString as _};

#[cfg_attr(feature = "fluent", derive(EsFluent))]
#[derive(Clone, Copy)]
enum BoolFilterOption {
    True,
    False,
}

impl BoolFilterOption {
    fn from_bool(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    fn value(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
        }
    }

    fn label(self) -> String {
        #[cfg(feature = "fluent")]
        {
            return self.to_fluent_string();
        }

        #[cfg(not(feature = "fluent"))]
        {
            match self {
                Self::True => "True".to_string(),
                Self::False => "False".to_string(),
            }
        }
    }
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

/// Trait for types that can provide their own filter options.
pub trait Filterable: FilterValue {
    fn options() -> Vec<FacetedFilterOption>;
}

impl FilterValue for bool {
    fn to_filter_string(&self) -> String {
        BoolFilterOption::from_bool(*self).value().to_string()
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
                label: BoolFilterOption::True.label(),
                value: BoolFilterOption::True.value().to_string(),
                count: None,
                icon: None,
            },
            FacetedFilterOption {
                label: BoolFilterOption::False.label(),
                value: BoolFilterOption::False.value().to_string(),
                count: None,
                icon: None,
            },
        ]
    }
}
