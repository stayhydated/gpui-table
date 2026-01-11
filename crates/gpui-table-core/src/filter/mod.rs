//! Filter types and traits for table filtering.

mod config;
mod convert;
mod traits;
mod value;
mod wrappers;

pub use config::{FacetedFilterOption, FilterConfig, FilterType};
#[cfg(feature = "rust_decimal")]
pub use convert::ToDecimal;
#[cfg(feature = "chrono")]
pub use convert::ToNaiveDate;
pub use traits::{FilterEntitiesExt, FilterValuesExt, Matchable};
pub use value::{FilterValue, Filterable};
pub use wrappers::{FacetedValue, RangeValue, TextValue};
