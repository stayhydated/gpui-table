use super::shared::{delegate_filter_shape, matches_optional_field};
use super::*;

/// Adapter shape for text filters over application-owned field types.
///
/// Use this when a table field is a transparent or domain-specific value type
/// that should be matched by its text representation while reusing the built-in
/// [`TextFilter`] component and MCP schema.
pub struct TextFilterAdapter;

/// Field conversion contract used by [`TextFilterAdapter`].
pub trait TextFilterField {
    /// Converts the field value into the text matched by [`TextFilter`].
    fn to_filter_text(&self) -> String;
}

impl TextFilterField for String {
    fn to_filter_text(&self) -> String {
        self.clone()
    }
}

/// Configured construction options for [`TextFilter`].
///
/// Use `TextFilter.matching_regex(...)`, `TextFilter.numeric_only()`,
/// `TextFilter.alphabetic_only()`, or `TextFilter.alphanumeric_only()` in
/// `#[gpui_table(filter(...))]` when a generated table field should build the
/// text filter with the matching input validator enabled. Regex patterns are
/// matched against the complete candidate input value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextFilterArgs {
    validation: TextFilterValidation,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TextFilterValidation {
    #[default]
    None,
    Alphabetic,
    Numeric,
    Alphanumeric,
    MatchingRegex(&'static str),
}

impl TextFilter {
    pub const fn matching_regex(pattern: &'static str) -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::MatchingRegex(pattern),
        }
    }

    pub const fn alphabetic_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Alphabetic,
        }
    }

    pub const fn numeric_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Numeric,
        }
    }

    pub const fn alphanumeric_only() -> TextFilterArgs {
        TextFilterArgs {
            validation: TextFilterValidation::Alphanumeric,
        }
    }
}

impl GpuiTableFilterShapeBuilder<TextFilter> for TextFilterArgs {
    fn build(
        self,
        title: impl Fn(&App) -> String + 'static,
        value: <TextFilter as GpuiTableFilterShape>::RawValue,
        on_change: impl Fn(<TextFilter as GpuiTableFilterShape>::RawValue, &mut Window, &mut App)
        + 'static,
        cx: &mut App,
    ) -> Entity<<TextFilter as GpuiTableFilterShape>::Component> {
        let entity = TextFilter::new_for(title, value, on_change, cx);
        match self.validation {
            TextFilterValidation::None => entity,
            TextFilterValidation::Alphabetic => entity.alphabetic_only(cx),
            TextFilterValidation::Numeric => entity.numeric_only(cx),
            TextFilterValidation::Alphanumeric => entity.alphanumeric_only(cx),
            TextFilterValidation::MatchingRegex(pattern) => entity.matching_regex(pattern, cx),
        }
    }
}

delegate_filter_shape!(TextFilterAdapter, TextFilter);

impl<T> ComponentShapeFor<T> for TextFilterAdapter where T: TextFilterField {}

impl<T> ComponentShapeFor<Option<T>> for TextFilterAdapter where T: TextFilterField {}

impl<T> GpuiTableFilterShapeFor<T> for TextFilterAdapter
where
    T: TextFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &T, value: &Self::FilterValue) -> bool {
        value.matches(&field.to_filter_text())
    }
}

impl<T> GpuiTableFilterShapeFor<Option<T>> for TextFilterAdapter
where
    T: TextFilterField,
{
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &Option<T>, value: &Self::FilterValue) -> bool {
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(&field.to_filter_text())
        })
    }
}

impl GpuiTableFilterShape for TextFilter {
    type Component = TextFilter;
    type RawValue = String;
    type FilterValue = TextValue;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        title: impl Fn(&App) -> String + 'static,
        value: Self::RawValue,
        on_change: impl Fn(Self::RawValue, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self::Component> {
        TextFilter::new_for(title, value, on_change, cx)
    }

    fn read_value(entity: &Entity<Self::Component>, cx: &App) -> Self::RawValue {
        entity.read(cx).value().to_string()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        TextValue::from(value)
    }

    fn unwrap_value(value: Self::FilterValue) -> Self::RawValue {
        value.0
    }

    fn set_silent(
        entity: &Entity<Self::Component>,
        value: Self::RawValue,
        window: &mut Window,
        cx: &mut App,
    ) {
        entity.update(cx, |filter, cx| filter.set_silent(value, window, cx));
    }

    fn reset_silent(entity: &Entity<Self::Component>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |filter, cx| filter.reset_silent(window, cx));
    }
}

impl DeclaredGpuiTableFilterShape for TextFilter {}

impl GpuiTableFilterShapeFor<String> for TextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        value.matches(field.as_ref())
    }
}

impl GpuiTableFilterShapeFor<Option<String>> for TextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &Option<String>, value: &Self::FilterValue) -> bool {
        matches_optional_field(field, value.is_active(), |field| {
            value.matches(field.as_ref())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_table_core::filter::{FilterType, TextValue};
    use gpui_table_runtime::shape::{GpuiTableFilterShape, GpuiTableFilterShapeFor};

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    struct Label(String);

    impl TextFilterField for Label {
        fn to_filter_text(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn text_shapes_match_owned_optional_and_adapter_fields() {
        let active = TextValue::from("rust");
        let inactive = TextValue::new();
        let owned = "Trustworthy".to_string();

        assert!(<TextFilter as GpuiTableFilterShapeFor<String>>::matches_field(&owned, &active));
        assert!(
            !<TextFilter as GpuiTableFilterShapeFor<String>>::matches_field(
                &"tables".to_string(),
                &active,
            )
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(
                &None, &inactive,
            )
        );
        assert!(
            <TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(
                &Some(owned),
                &active,
            )
        );
        assert!(
            !<TextFilter as GpuiTableFilterShapeFor<Option<String>>>::matches_field(&None, &active,)
        );

        assert!(
            <TextFilterAdapter as GpuiTableFilterShapeFor<Label>>::matches_field(
                &Label("Rust language".into()),
                &active,
            )
        );
        assert!(<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<Label>,
        >>::matches_field(&None, &inactive,));
        assert!(!<TextFilterAdapter as GpuiTableFilterShapeFor<
            Option<Label>,
        >>::matches_field(&None, &active,));

        assert!(matches!(
            <TextFilter as GpuiTableFilterShapeFor<String>>::filter_type(),
            FilterType::Text
        ));
        assert!(matches!(
            <TextFilterAdapter as GpuiTableFilterShapeFor<Label>>::filter_type(),
            FilterType::Text
        ));
        assert_eq!(
            <TextFilter as GpuiTableFilterShape>::wrap_value("query".into()),
            TextValue::from("query")
        );

        assert_eq!(
            TextFilter::alphabetic_only().validation,
            super::TextFilterValidation::Alphabetic
        );
        assert_eq!(
            TextFilter::numeric_only().validation,
            super::TextFilterValidation::Numeric
        );
        assert_eq!(
            TextFilter::alphanumeric_only().validation,
            super::TextFilterValidation::Alphanumeric
        );
        assert_eq!(
            TextFilter::matching_regex("[a-z]+").validation,
            super::TextFilterValidation::MatchingRegex("[a-z]+")
        );
        assert_eq!(
            TextFilterArgs::default().validation,
            super::TextFilterValidation::None
        );
    }
}
