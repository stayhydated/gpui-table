#[cfg(feature = "derive")]
pub use gpui_table_derive::*;

pub use gpui_table_core::*;

pub use derive_new;

/// Re-export filter components for use in derive macros.
/// These are used by the `#[gpui_table(filter = ...)]` attribute.
pub mod components {
    pub use gpui_table_components::TableFilterComponent;
    pub use gpui_table_components::date_range_filter::DateRangeFilter;
    pub use gpui_table_components::faceted_filter::FacetedFilter;
    pub use gpui_table_components::number_range_filter::NumberRangeFilter;
    pub use gpui_table_components::text_filter::TextFilter;
}
