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
