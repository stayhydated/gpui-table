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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacetedFilterOption {
    pub label: String,
    pub value: String,
    pub count: Option<usize>,
}
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
