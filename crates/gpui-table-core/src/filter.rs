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
    #[cfg(feature = "chrono")]
    DateRange,
    NumberRange,
    Text,
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

pub trait Filterable: FilterValue {
    fn options() -> Vec<FacetedFilterOption>;
}

impl FilterValue for bool {
    fn to_filter_string(&self) -> String {
        if *self { "true" } else { "false" }.to_string()
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
