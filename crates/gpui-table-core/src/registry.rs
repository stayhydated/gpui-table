use strum::{Display, EnumString, IntoStaticStr};

inventory::collect!(GpuiTableShape);

/// Metadata for a table row type, collected via inventory.
#[derive(Debug)]
pub struct GpuiTableShape {
    pub struct_name: &'static str,
    pub table_id: &'static str,
    pub table_title: &'static str,
    pub columns: &'static [ColumnVariant],
    pub filters: &'static [FilterVariant],
}

impl GpuiTableShape {
    pub const fn new(
        struct_name: &'static str,
        table_id: &'static str,
        table_title: &'static str,
        columns: &'static [ColumnVariant],
        filters: &'static [FilterVariant],
    ) -> Self {
        Self {
            struct_name,
            table_id,
            table_title,
            columns,
            filters,
        }
    }
}

/// Metadata for a single filter in a table.
#[derive(Debug)]
pub struct FilterVariant {
    pub field_name: &'static str,
    pub filter_type: RegistryFilterType,
}

impl FilterVariant {
    pub const fn new(field_name: &'static str, filter_type: RegistryFilterType) -> Self {
        Self {
            field_name,
            filter_type,
        }
    }
}

/// Type of filter for registry (metadata only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegistryFilterType {
    Faceted,
    DateRange,
    NumberRange,
    Text,
}

/// Metadata for a single column in a table.
#[derive(Debug)]
pub struct ColumnVariant {
    pub field_name: &'static str,
    pub field_type: &'static str,
    pub title: &'static str,
    pub width: f32,
    pub sortable: bool,
    pub fixed: ColumnFixed,
}

impl ColumnVariant {
    pub const fn new(
        field_name: &'static str,
        field_type: &'static str,
        title: &'static str,
        width: f32,
        sortable: bool,
        fixed: ColumnFixed,
    ) -> Self {
        Self {
            field_name,
            field_type,
            title,
            width,
            sortable,
            fixed,
        }
    }
}

/// Column fixed position.
#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ColumnFixed {
    #[default]
    None,
    Left,
    Right,
}

pub use inventory;
