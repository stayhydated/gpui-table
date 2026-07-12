//! Filter value traits and implementations.

#[cfg(feature = "fluent")]
use es_fluent::EsFluent;
use gpui_table_schema::filter::FacetedFilterOption;

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

    fn value(self) -> String {
        match self {
            Self::True => "true",
            Self::False => "false",
        }
        .to_string()
    }

    fn label(self) -> String {
        #[cfg(feature = "fluent")]
        {
            crate::i18n::localize_message(&self)
        }

        #[cfg(not(feature = "fluent"))]
        {
            match self {
                Self::True => "True",
                Self::False => "False",
            }
            .to_string()
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
        BoolFilterOption::from_bool(*self).value()
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
                group: None,
                label: BoolFilterOption::True.label(),
                value: BoolFilterOption::True.value(),
                count: None,
                icon: None,
            },
            FacetedFilterOption {
                group: None,
                label: BoolFilterOption::False.label(),
                value: BoolFilterOption::False.value(),
                count: None,
                icon: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{FilterValue as _, Filterable as _};

    #[test]
    fn bool_filter_values_round_trip_and_reject_other_spellings() {
        assert_eq!(true.to_filter_string(), "true");
        assert_eq!(false.to_filter_string(), "false");
        assert_eq!(bool::from_filter_string("true"), Some(true));
        assert_eq!(bool::from_filter_string("false"), Some(false));
        assert_eq!(bool::from_filter_string("True"), None);
        assert_eq!(bool::from_filter_string(""), None);
    }

    #[test]
    fn bool_filter_options_are_complete_and_stable() {
        let options = bool::options();

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].value, "true");
        assert_eq!(options[1].value, "false");
        assert!(!options[0].label.is_empty());
        assert!(!options[1].label.is_empty());
        assert!(options.iter().all(|option| {
            option.group.is_none() && option.count.is_none() && option.icon.is_none()
        }));
    }
}
