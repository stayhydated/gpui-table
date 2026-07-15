use gpui::{AnyElement, IntoElement, SharedString};
use gpui_table_schema::{filter::FacetedFilterIcon, registry::RegistryFilterType};

/// One generated filter rendered as sidebar-ready erased GPUI content.
pub struct FilterSidebarItem {
    field_id: SharedString,
    label: SharedString,
    filter_type: RegistryFilterType,
    active: bool,
    element: AnyElement,
}

impl FilterSidebarItem {
    /// Create sidebar metadata and erase the concrete filter element type.
    pub fn new(
        field_id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        filter_type: RegistryFilterType,
        active: bool,
        element: impl IntoElement,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            label: label.into(),
            filter_type,
            active,
            element: element.into_any_element(),
        }
    }

    /// Stable table-and-field identifier for this filter.
    pub fn field_id(&self) -> &SharedString {
        &self.field_id
    }

    /// Localized field label.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Semantic filter group.
    pub fn filter_type(&self) -> RegistryFilterType {
        self.filter_type
    }

    /// Whether the filter currently narrows the row set.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Consume the descriptor and return its erased filter element.
    pub fn into_element(self) -> AnyElement {
        self.element
    }
}

/// A nonempty semantic group of generated sidebar filters.
pub struct FilterSidebarGroup {
    filter_type: RegistryFilterType,
    items: Vec<FilterSidebarItem>,
}

impl FilterSidebarGroup {
    /// Semantic type shared by every item in this group.
    pub fn filter_type(&self) -> RegistryFilterType {
        self.filter_type
    }

    /// Sidebar items in source-field order.
    pub fn items(&self) -> &[FilterSidebarItem] {
        &self.items
    }

    /// Consume the group and return its sidebar items.
    pub fn into_items(self) -> Vec<FilterSidebarItem> {
        self.items
    }
}

/// Grouped render data for one generated filter-entity collection.
pub struct FilterSidebarData {
    groups: Vec<FilterSidebarGroup>,
    active_count: usize,
}

impl FilterSidebarData {
    /// Group generated filters in the stable Text, Faceted, Number, Date order.
    pub fn new(items: Vec<FilterSidebarItem>) -> Self {
        let active_count = items.iter().filter(|item| item.is_active()).count();
        let mut items = items.into_iter().map(Some).collect::<Vec<_>>();
        let groups = [
            RegistryFilterType::Text,
            RegistryFilterType::Faceted,
            RegistryFilterType::NumberRange,
            RegistryFilterType::DateRange,
        ]
        .into_iter()
        .filter_map(|filter_type| {
            let grouped_items = items
                .iter_mut()
                .filter_map(|item| {
                    (item.as_ref()?.filter_type() == filter_type)
                        .then(|| item.take().expect("matching filter remains available"))
                })
                .collect::<Vec<_>>();
            (!grouped_items.is_empty()).then_some(FilterSidebarGroup {
                filter_type,
                items: grouped_items,
            })
        })
        .collect();

        Self {
            groups,
            active_count,
        }
    }

    /// Nonempty semantic groups in stable sidebar order.
    pub fn groups(&self) -> &[FilterSidebarGroup] {
        &self.groups
    }

    /// Number of active generated filters.
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Consume the data and return its groups.
    pub fn into_groups(self) -> Vec<FilterSidebarGroup> {
        self.groups
    }
}

/// Trait for generated filter entity collections that can read and render their current values.
///
/// The derive macro also emits inherent `read_values()`, `apply_values()`,
/// `filter_sidebar_data()`, `active_filter_count()`, and `reset_filters()`
/// methods on each generated `XxxFilterEntities` type. This trait remains
/// useful for generic helper code that wants to work across multiple generated
/// filter collections.
pub trait FilterEntitiesExt {
    /// The filter values type that this entity collection produces.
    type Values: gpui_table_core::filter::FilterValuesExt;

    /// Read all current filter values into the generated filter-values struct.
    ///
    /// Individual wrapper fields can then be serialized with
    /// `gpui_table_component::QueryFilterValue` when their wrapped type
    /// supports query-string conversion.
    fn read_values(&self, cx: &gpui::App) -> Self::Values;

    /// Replace every generated filter from a typed preset and notify once.
    fn apply_values(&self, values: Self::Values, window: &mut gpui::Window, cx: &mut gpui::App);

    /// Build grouped, sidebar-ready render data for all generated filters.
    fn filter_sidebar_data(&self, cx: &gpui::App) -> FilterSidebarData;

    /// Count generated filters that currently narrow the row set.
    fn active_filter_count(&self, cx: &gpui::App) -> usize;

    /// Reset every generated filter and notify the configured change callback once.
    fn reset_filters(&self, window: &mut gpui::Window, cx: &mut gpui::App);
}

/// Convert a `gpui-component` icon token into UI-neutral filter metadata.
pub fn icon_from_name(name: impl gpui_component::IconNamed) -> FacetedFilterIcon {
    FacetedFilterIcon::from_path(name.path().to_string())
}

#[cfg(test)]
mod tests {
    use super::{FilterSidebarData, FilterSidebarItem, icon_from_name};
    use gpui::{SharedString, div};
    use gpui_table_schema::registry::RegistryFilterType;

    struct TestIcon;

    impl gpui_component::IconNamed for TestIcon {
        fn path(self) -> SharedString {
            "icons/test.svg".into()
        }
    }

    #[test]
    fn icon_names_convert_to_ui_neutral_metadata() {
        assert_eq!(icon_from_name(TestIcon).path(), "icons/test.svg");
    }

    #[test]
    fn sidebar_data_groups_in_stable_semantic_order_and_counts_active_items() {
        let data = FilterSidebarData::new(vec![
            FilterSidebarItem::new("date", "Date", RegistryFilterType::DateRange, false, div()),
            FilterSidebarItem::new("status", "Status", RegistryFilterType::Faceted, true, div()),
            FilterSidebarItem::new("name", "Name", RegistryFilterType::Text, true, div()),
            FilterSidebarItem::new(
                "amount",
                "Amount",
                RegistryFilterType::NumberRange,
                false,
                div(),
            ),
        ]);

        assert_eq!(data.active_count(), 2);
        assert_eq!(
            data.groups()
                .iter()
                .map(|group| group.filter_type())
                .collect::<Vec<_>>(),
            vec![
                RegistryFilterType::Text,
                RegistryFilterType::Faceted,
                RegistryFilterType::NumberRange,
                RegistryFilterType::DateRange,
            ]
        );
        assert_eq!(data.groups()[0].items()[0].field_id().as_ref(), "name");
        assert!(data.groups()[0].items()[0].is_active());
    }
}
