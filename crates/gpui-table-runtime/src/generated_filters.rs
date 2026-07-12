use gpui_table_schema::filter::FacetedFilterIcon;

/// Trait for generated filter entity collections that can read and render their current values.
///
/// The derive macro also emits inherent `read_values()` and `all_filters()`
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

    /// Render all filters in a single row.
    fn all_filters(&self) -> impl gpui::IntoElement;
}

/// Convert a `gpui-component` icon token into UI-neutral filter metadata.
pub fn icon_from_name(name: impl gpui_component::IconNamed) -> FacetedFilterIcon {
    FacetedFilterIcon::from_path(name.path().to_string())
}

#[cfg(test)]
mod tests {
    use super::icon_from_name;
    use gpui::SharedString;

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
}
