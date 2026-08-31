#[cfg(feature = "chrono")]
use crate::DateRangeFilter;
use crate::{FacetedFilter, FacetedFilterExt as _, TextFilter, TextFilterExt as _};
#[cfg(feature = "rust_decimal")]
use crate::{NumberRangeFilter, NumberRangeFilterExt as _};
use gpui::{App, Entity, Window};
use gpui_table_core::filter::{FacetedValue, FilterType, Filterable, TextValue};
use gpui_table_runtime::shape::{
    ComponentShapeFor, ComponentShapeMetadata, DeclaredComponentShape,
    DeclaredGpuiTableFilterShape, GpuiTableFilterShape, GpuiTableFilterShapeBuilder,
    GpuiTableFilterShapeFor,
};
use gpui_table_schema::registry::RegistryFilterType;
use std::collections::HashSet;
use std::marker::PhantomData;

#[cfg(feature = "chrono")]
mod date;
mod faceted;
#[cfg(feature = "rust_decimal")]
mod number;
mod shared;
mod text;

#[cfg(feature = "chrono")]
pub use date::*;
pub use faceted::*;
#[cfg(feature = "rust_decimal")]
pub use number::*;
pub use text::*;
