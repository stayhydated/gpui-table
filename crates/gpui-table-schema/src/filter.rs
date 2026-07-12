//! Filter configuration metadata shared across the workspace.

/// UI-neutral icon metadata for faceted filter options.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FacetedFilterIcon {
    path: String,
}

impl FacetedFilterIcon {
    /// Build an icon descriptor from an asset path such as `icons/check.svg`.
    pub fn from_path(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Return the asset path used by runtime filter components.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// A single option in a faceted filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacetedFilterOption {
    pub group: Option<String>,
    pub label: String,
    pub value: String,
    pub count: Option<usize>,
    pub icon: Option<FacetedFilterIcon>,
}

/// Configuration for a column filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterConfig {
    pub column_index: usize,
    pub filter_type: FilterType,
}

/// The type of filter to apply to a column.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilterType {
    Faceted(Vec<FacetedFilterOption>),
    DateRange,
    NumberRange,
    Text,
}

#[cfg(test)]
mod tests {
    use super::{FacetedFilterIcon, FacetedFilterOption, FilterConfig, FilterType};

    #[test]
    fn faceted_filter_metadata_preserves_icons_and_option_details() {
        let icon = FacetedFilterIcon::from_path("icons/check.svg");
        assert_eq!(icon.path(), "icons/check.svg");

        let option = FacetedFilterOption {
            group: Some("State".into()),
            label: "Active".into(),
            value: "active".into(),
            count: Some(3),
            icon: Some(icon),
        };
        let config = FilterConfig {
            column_index: 2,
            filter_type: FilterType::Faceted(vec![option.clone()]),
        };

        assert_eq!(config.column_index, 2);
        assert_eq!(config.filter_type, FilterType::Faceted(vec![option]));
        assert_ne!(FilterType::DateRange, FilterType::NumberRange);
        assert_ne!(FilterType::Text, FilterType::NumberRange);
    }
}
