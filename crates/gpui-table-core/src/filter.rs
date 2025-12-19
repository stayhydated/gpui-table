#[cfg(feature = "chrono")]
use chrono::NaiveDateTime;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum TableFilter {
    Faceted(HashSet<String>),
    #[cfg(feature = "chrono")]
    DateRange(Option<NaiveDateTime>, Option<NaiveDateTime>),
    NumberRange(Option<f64>, Option<f64>),
    Text(String),
}

impl TableFilter {
    pub fn is_empty(&self) -> bool {
        match self {
            TableFilter::Faceted(set) => set.is_empty(),
            #[cfg(feature = "chrono")]
            TableFilter::DateRange(start, end) => start.is_none() && end.is_none(),
            TableFilter::NumberRange(min, max) => min.is_none() && max.is_none(),
            TableFilter::Text(s) => s.is_empty(),
        }
    }
}

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
    DateRange,
    NumberRange,
    Text,
}

pub trait Filterable {
    fn options() -> Vec<FacetedFilterOption>;
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
