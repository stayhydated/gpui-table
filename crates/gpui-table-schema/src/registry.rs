use strum::{Display, EnumString, IntoStaticStr};

inventory::collect!(GpuiTableShape);

pub use component_shape::{ComponentShapeUse, RustPath, RustType};

/// Metadata for a table row type, collected via inventory.
#[derive(Debug)]
pub struct GpuiTableShape {
    pub struct_name: &'static str,
    pub table_id: &'static str,
    pub table_title: &'static str,
    pub fluent: bool,
    pub columns: &'static [ColumnVariant],
    pub filters: &'static [FilterVariant],
    /// Whether load_more is enabled on the table via #[gpui_table(load_more)].
    pub load_more: bool,
    /// The source file path where the struct with #[derive(GpuiTable)] is declared.
    /// This is the full path from file!() macro, useful for generating imports.
    pub source_path: &'static str,
}

impl GpuiTableShape {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        struct_name: &'static str,
        table_id: &'static str,
        table_title: &'static str,
        fluent: bool,
        columns: &'static [ColumnVariant],
        filters: &'static [FilterVariant],
        load_more: bool,
        source_path: &'static str,
    ) -> Self {
        Self {
            struct_name,
            table_id,
            table_title,
            fluent,
            columns,
            filters,
            load_more,
            source_path,
        }
    }
}

/// Metadata for a single filter in a table.
#[derive(Debug)]
pub struct FilterVariant {
    pub shape_use: ComponentShapeUse,
    pub field_name: &'static str,
    pub filter_type: RegistryFilterType,
    pub shape_path: RustPath,
    pub component_path: RustPath,
}

impl FilterVariant {
    pub const fn new(
        field_name: &'static str,
        filter_type: RegistryFilterType,
        shape_path: RustPath,
        component_path: RustPath,
    ) -> Self {
        let shape_use = ComponentShapeUse::new(field_name, shape_path);
        Self {
            shape_use,
            field_name,
            filter_type,
            shape_path,
            component_path,
        }
    }

    pub const fn with_field_type(mut self, field_type: RustType) -> Self {
        self.shape_use = self.shape_use.with_field_type(field_type);
        self
    }
}

/// Type of filter for registry (metadata only).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub field_type: RustType,
    pub title: &'static str,
    pub width: f32,
    pub sortable: bool,
    pub fixed: ColumnFixed,
}

impl ColumnVariant {
    pub const fn new(
        field_name: &'static str,
        field_type: RustType,
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

#[cfg(test)]
mod tests {
    use super::{FilterVariant, RegistryFilterType, RustPath, RustType};

    #[test]
    fn filter_variant_records_neutral_shape_use_metadata() {
        let variant = FilterVariant::new(
            "status",
            RegistryFilterType::Faceted,
            RustPath::from_macro_tokens_unchecked("crate::StatusFilterShape"),
            RustPath::from_macro_tokens_unchecked("crate::StatusFilter"),
        );

        assert_eq!(variant.shape_use.field_name(), "status");
        assert_eq!(
            variant.shape_use.shape_path().as_str(),
            "crate::StatusFilterShape"
        );
        assert_eq!(variant.field_name, "status");
        assert_eq!(variant.shape_path.as_str(), "crate::StatusFilterShape");
        assert_eq!(variant.component_path.as_str(), "crate::StatusFilter");
    }

    #[test]
    fn filter_variant_records_field_type_in_neutral_shape_use() {
        let variant = FilterVariant::new(
            "status",
            RegistryFilterType::Faceted,
            RustPath::from_macro_tokens_unchecked("crate::StatusFilterShape"),
            RustPath::from_macro_tokens_unchecked("crate::StatusFilter"),
        )
        .with_field_type(RustType::from_macro_tokens_unchecked("crate::Status"));

        assert_eq!(
            variant.shape_use.field_type().map(RustType::as_str),
            Some("crate::Status")
        );
    }
}
