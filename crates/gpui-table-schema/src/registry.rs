use std::fmt;
use strum::{Display, EnumString, IntoStaticStr};

inventory::collect!(GpuiTableShape);

pub use component_shape::{ComponentFieldName, ComponentShapeUse, RustPath, RustType};

/// Stable borrowed identifier for a table row or registry entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TableId<'a> {
    value: &'a str,
}

impl<'a> TableId<'a> {
    /// Creates a validated table id from a borrowed identifier.
    ///
    /// Table ids are non-empty and contain only lowercase ASCII letters,
    /// digits, `_`, or `-`.
    ///
    /// # Panics
    ///
    /// Panics when `value` is not a valid stable table identifier.
    pub const fn new(value: &'a str) -> Self {
        assert_valid_table_id(value);
        Self { value }
    }

    /// Returns the table id string.
    pub const fn as_str(&self) -> &'a str {
        self.value
    }
}

const fn assert_valid_table_id(value: &str) {
    let bytes = value.as_bytes();
    assert!(!bytes.is_empty(), "table ids must not be empty");

    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        assert!(
            matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'),
            "table ids must use lowercase ASCII letters, digits, `_`, or `-`"
        );
        index += 1;
    }
}

impl fmt::Display for TableId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value)
    }
}

impl From<TableId<'_>> for String {
    fn from(table_id: TableId<'_>) -> Self {
        table_id.to_string()
    }
}

/// Metadata for a table row type, collected via inventory.
#[derive(Debug)]
pub struct GpuiTableShape {
    pub struct_name: &'static str,
    pub table_id: TableId<'static>,
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
            table_id: TableId::new(table_id),
            table_title,
            fluent,
            columns,
            filters,
            load_more,
            source_path,
        }
    }

    pub const fn table_id(&self) -> TableId<'static> {
        self.table_id
    }
}

/// Metadata for a single filter in a table.
#[derive(Debug)]
pub struct FilterVariant {
    pub shape_use: ComponentShapeUse,
    pub filter_type: RegistryFilterType,
    pub component_path: RustPath,
}

impl FilterVariant {
    pub const fn new(
        shape_use: ComponentShapeUse,
        filter_type: RegistryFilterType,
        component_path: RustPath,
    ) -> Self {
        Self {
            shape_use,
            filter_type,
            component_path,
        }
    }
}

/// Type of filter for registry (metadata only).
#[derive(Clone, Copy, Debug, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case", const_into_str)]
pub enum RegistryFilterType {
    Faceted,
    DateRange,
    NumberRange,
    Text,
}

impl RegistryFilterType {
    pub const fn as_str(self) -> &'static str {
        self.into_str()
    }
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
#[strum(serialize_all = "snake_case", const_into_str)]
pub enum ColumnFixed {
    #[default]
    None,
    Left,
    Right,
}

pub use inventory;

#[cfg(test)]
mod tests {
    use super::{
        ColumnFixed, ColumnVariant, ComponentFieldName, ComponentShapeUse, FilterVariant,
        GpuiTableShape, RegistryFilterType, RustPath, RustType, TableId,
    };

    #[test]
    fn table_id_wrapper_exposes_borrowed_identifier() {
        let table_id = TableId::new("purchase_order");

        assert_eq!(table_id.as_str(), "purchase_order");
        assert_eq!(table_id.to_string(), "purchase_order");
        assert_eq!(String::from(table_id), "purchase_order");
    }

    #[test]
    #[should_panic(expected = "table ids must not be empty")]
    fn table_ids_reject_empty_values() {
        _ = TableId::new("");
    }

    #[test]
    #[should_panic(expected = "table ids must use lowercase ASCII")]
    fn table_ids_reject_unstable_text() {
        _ = TableId::new("Purchase Order");
    }

    #[test]
    fn table_shape_exposes_typed_table_id() {
        let shape = GpuiTableShape::new(
            "PurchaseOrder",
            "purchase_order",
            "Purchase Orders",
            false,
            &[],
            &[],
            false,
            "src/purchase_order.rs",
        );

        assert_eq!(shape.table_id().as_str(), "purchase_order");
    }

    #[test]
    fn filter_variant_records_neutral_shape_use_metadata() {
        let variant = FilterVariant::new(
            ComponentShapeUse::new(
                ComponentFieldName::new("status"),
                RustPath::from_macro_tokens_unchecked("crate::StatusFilterShape"),
            ),
            RegistryFilterType::Faceted,
            RustPath::from_macro_tokens_unchecked("crate::StatusFilter"),
        );

        assert_eq!(variant.shape_use.field_name().as_str(), "status");
        assert_eq!(
            variant.shape_use.shape_path().as_str(),
            "crate::StatusFilterShape"
        );
        assert_eq!(variant.component_path.as_str(), "crate::StatusFilter");
    }

    #[test]
    fn filter_variant_records_field_type_in_neutral_shape_use() {
        let variant = FilterVariant::new(
            ComponentShapeUse::new(
                ComponentFieldName::new("status"),
                RustPath::from_macro_tokens_unchecked("crate::StatusFilterShape"),
            )
            .with_field_type(RustType::from_macro_tokens_unchecked("crate::Status")),
            RegistryFilterType::Faceted,
            RustPath::from_macro_tokens_unchecked("crate::StatusFilter"),
        );

        assert_eq!(
            variant.shape_use.field_type().map(RustType::as_str),
            Some("crate::Status")
        );
    }

    #[test]
    fn registry_filter_type_names_are_stable_schema_metadata() {
        assert_eq!(RegistryFilterType::Faceted.as_str(), "faceted");
        assert_eq!(RegistryFilterType::DateRange.as_str(), "date_range");
        assert_eq!(RegistryFilterType::NumberRange.as_str(), "number_range");
        assert_eq!(RegistryFilterType::Text.as_str(), "text");
    }

    #[test]
    fn column_metadata_and_fixed_position_names_are_stable() {
        let column = ColumnVariant::new(
            "name",
            RustType::from_macro_tokens_unchecked("String"),
            "Name",
            160.0,
            true,
            ColumnFixed::Left,
        );

        assert_eq!(column.field_name, "name");
        assert_eq!(column.field_type.as_str(), "String");
        assert_eq!(column.title, "Name");
        assert_eq!(column.width, 160.0);
        assert!(column.sortable);
        assert_eq!(column.fixed, ColumnFixed::Left);

        assert_eq!(ColumnFixed::default(), ColumnFixed::None);
        assert_eq!(ColumnFixed::None.to_string(), "none");
        assert_eq!(ColumnFixed::Left.to_string(), "left");
        assert_eq!(ColumnFixed::Right.to_string(), "right");
        assert_eq!("none".parse(), Ok(ColumnFixed::None));
        assert_eq!("left".parse(), Ok(ColumnFixed::Left));
        assert_eq!("right".parse(), Ok(ColumnFixed::Right));
        assert!("center".parse::<ColumnFixed>().is_err());
    }
}
