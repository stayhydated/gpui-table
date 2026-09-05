use gpui_kit::Entity;
use gpui_table::{
    GpuiTable,
    core::filter::{FilterType, TextValue},
    runtime::shape::{
        ComponentShapeMetadata, DeclaredComponentShape, DeclaredGpuiTableFilterShape,
        GpuiTableFilterShape, GpuiTableFilterShapeFor,
    },
    schema::registry::RegistryFilterType,
};

struct LocalTextFilter;

impl ComponentShapeMetadata for LocalTextFilter {}
impl DeclaredComponentShape for LocalTextFilter {}

impl GpuiTableFilterShape for LocalTextFilter {
    type Component = gpui_table_component::TextFilter;
    type RawValue = String;
    type FilterValue = TextValue;

    const FILTER_TYPE: RegistryFilterType = RegistryFilterType::Text;

    fn new_for(
        _title: impl Fn(&gpui_kit::App) -> String + 'static,
        _value: Self::RawValue,
        _on_change: impl Fn(Self::RawValue, &mut gpui_kit::Window, &mut gpui_kit::App) + 'static,
        _cx: &mut gpui_kit::App,
    ) -> Entity<Self::Component> {
        unimplemented!()
    }

    fn read_value(_entity: &Entity<Self::Component>, _cx: &gpui_kit::App) -> Self::RawValue {
        String::new()
    }

    fn wrap_value(value: Self::RawValue) -> Self::FilterValue {
        TextValue::from(value)
    }

    fn reset_silent(
        _entity: &Entity<Self::Component>,
        _window: &mut gpui_kit::Window,
        _cx: &mut gpui_kit::App,
    ) {
    }
}

impl DeclaredGpuiTableFilterShape for LocalTextFilter {}

impl GpuiTableFilterShapeFor<String> for LocalTextFilter {
    fn filter_type() -> FilterType {
        FilterType::Text
    }

    fn matches_field(field: &String, value: &Self::FilterValue) -> bool {
        value.matches(field.as_str())
    }
}

#[derive(Clone, GpuiTable)]
#[gpui_table(filters, mcp)]
struct Row {
    #[gpui_table(filter(LocalTextFilter))]
    name: String,
}

fn main() {}
