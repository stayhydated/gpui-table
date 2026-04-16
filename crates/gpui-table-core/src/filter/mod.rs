//! Filter types and traits for table filtering.

mod convert;
mod traits;
mod value;
mod wrappers;

#[cfg(feature = "rust_decimal")]
pub use convert::ToDecimal;
#[cfg(feature = "chrono")]
pub use convert::ToNaiveDate;
pub use gpui_table_schema::filter::{
    FacetedFilterIcon, FacetedFilterOption, FilterConfig, FilterType,
};
pub use traits::{FilterValuesExt, Matchable};
pub use value::{FilterValue, Filterable};
pub use wrappers::{FacetedValue, RangeValue, SingleValue, TextValue};
