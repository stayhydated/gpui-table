//! Filter configuration types.

/// A single option in a faceted filter.
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

/// Configuration for a column filter.
#[derive(Clone, Debug)]
pub struct FilterConfig {
    pub column_index: usize,
    pub filter_type: FilterType,
}

/// The type of filter to apply to a column.
#[derive(Clone, Debug)]
pub enum FilterType {
    Faceted(Vec<FacetedFilterOption>),
    #[cfg(feature = "chrono")]
    DateRange,
    NumberRange,
    Text,
}
