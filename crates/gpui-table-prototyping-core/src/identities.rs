use gpui_table_schema::registry::{ColumnVariant, GpuiTableShape};
use heck::ToSnakeCase as _;
use quote::format_ident;

use crate::code_gen::TableCodegenError;

/// Trait for deriving various identifier names from a table shape.
pub trait TableIdentities {
    /// The original struct name (e.g., "User")
    fn struct_name(&self) -> &'static str;

    /// The struct name as an identifier.
    ///
    /// For user-facing tooling, prefer [`TableIdentitiesExt::try_struct_name_ident`].
    fn struct_name_ident(&self) -> syn::Ident {
        syn::parse_str(self.struct_name()).unwrap()
    }

    /// The table story struct name (e.g., "UserTableStory")
    fn story_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableStory", self.struct_name())
    }

    /// The table delegate struct name (e.g., "UserTableDelegate")
    fn delegate_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableDelegate", self.struct_name())
    }

    /// The table ID.
    fn table_id(&self) -> &'static str;

    /// The table title.
    fn table_title(&self) -> &'static str;

    /// Whether the source table uses Fluent-derived labels.
    fn uses_fluent_labels(&self) -> bool;

    /// The snake_case version of struct name for file paths.
    fn snake_case_name(&self) -> String {
        self.struct_name().to_snake_case()
    }

    /// Snake case name as identifier (for import paths).
    ///
    /// For user-facing tooling, prefer [`TableIdentitiesExt::try_snake_case_ident`].
    fn snake_case_ident(&self) -> syn::Ident {
        syn::parse_str(&self.snake_case_name()).unwrap()
    }

    /// Fluent label enum identifier (e.g., "UserLabelVariants").
    fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelVariants", self.struct_name())
    }

    /// Fluent description enum identifier (e.g., "UserDescriptionVariants").
    fn ftl_description_ident(&self) -> syn::Ident {
        format_ident!("{}DescriptionVariants", self.struct_name())
    }

    /// The story ID literal (e.g., "user-table-story").
    fn story_id_literal(&self) -> String {
        format!("{}-table-story", self.snake_case_name().replace('_', "-"))
    }

    /// Whether this table has filters defined.
    fn has_filters(&self) -> bool;
}

/// Fallible identifier helpers for user-facing tooling.
pub trait TableIdentitiesExt: TableIdentities {
    fn try_struct_name_ident(&self) -> Result<syn::Ident, TableCodegenError> {
        parse_ident(
            "struct identifier",
            self.struct_name(),
            self.struct_name().to_string(),
        )
    }

    fn try_snake_case_ident(&self) -> Result<syn::Ident, TableCodegenError> {
        let snake_case_name = self.snake_case_name();
        parse_ident("snake_case identifier", self.struct_name(), snake_case_name)
    }
}

impl<T: TableIdentities + ?Sized> TableIdentitiesExt for T {}

/// Identities wrapper for `GpuiTableShape`.
pub struct ShapeIdentities<'a>(&'a GpuiTableShape);

impl<'a> ShapeIdentities<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self {
        Self(shape)
    }

    /// Get the underlying shape.
    pub fn shape(&self) -> &'a GpuiTableShape {
        self.0
    }

    /// Get columns.
    pub fn columns(&self) -> &'static [ColumnVariant] {
        self.0.columns
    }
}

impl TableIdentities for ShapeIdentities<'_> {
    fn struct_name(&self) -> &'static str {
        self.0.struct_name
    }

    fn table_id(&self) -> &'static str {
        self.0.table_id
    }

    fn table_title(&self) -> &'static str {
        self.0.table_title
    }

    fn uses_fluent_labels(&self) -> bool {
        self.0.fluent
    }

    fn has_filters(&self) -> bool {
        !self.0.filters.is_empty()
    }
}

pub(crate) fn parse_ident(
    kind: &'static str,
    struct_name: &'static str,
    value: String,
) -> Result<syn::Ident, TableCodegenError> {
    syn::parse_str(&value).map_err(|source| TableCodegenError::InvalidIdentifier {
        kind,
        struct_name,
        value,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::{ShapeIdentities, TableIdentities as _, TableIdentitiesExt as _, parse_ident};
    use crate::code_gen::TableCodegenError;
    use gpui_table_schema::registry::{
        ColumnFixed, ColumnVariant, ComponentFieldName, ComponentShapeUse, FilterVariant,
        GpuiTableShape, RegistryFilterType, RustPath, RustType,
    };

    static COLUMNS: [ColumnVariant; 1] = [ColumnVariant::new(
        "display_name",
        RustType::from_macro_tokens_unchecked("String"),
        "Display name",
        100.0,
        true,
        ColumnFixed::None,
    )];
    static FILTERS: [FilterVariant; 1] = [FilterVariant::new(
        ComponentShapeUse::new(
            ComponentFieldName::new("display_name"),
            RustPath::from_macro_tokens_unchecked("crate::TextFilter"),
        ),
        RegistryFilterType::Text,
        RustPath::from_macro_tokens_unchecked("crate::TextFilter"),
    )];
    static SHAPE: GpuiTableShape = GpuiTableShape::new(
        "PurchaseOrder",
        "purchase_order",
        "Purchase orders",
        true,
        &COLUMNS,
        &FILTERS,
        true,
        "demo/src/purchase_order.rs",
    );

    #[test]
    fn shape_identities_derive_every_public_name_and_flag() {
        let identities = ShapeIdentities::new(&SHAPE);

        assert_eq!(identities.shape().table_id, "purchase_order");
        assert_eq!(identities.columns().len(), 1);
        assert_eq!(identities.struct_name(), "PurchaseOrder");
        assert_eq!(identities.struct_name_ident().to_string(), "PurchaseOrder");
        assert_eq!(
            identities.story_struct_ident().to_string(),
            "PurchaseOrderTableStory"
        );
        assert_eq!(
            identities.delegate_struct_ident().to_string(),
            "PurchaseOrderTableDelegate"
        );
        assert_eq!(identities.table_id(), "purchase_order");
        assert_eq!(identities.table_title(), "Purchase orders");
        assert!(identities.uses_fluent_labels());
        assert_eq!(identities.snake_case_name(), "purchase_order");
        assert_eq!(identities.snake_case_ident().to_string(), "purchase_order");
        assert_eq!(
            identities.ftl_label_ident().to_string(),
            "PurchaseOrderLabelVariants"
        );
        assert_eq!(
            identities.ftl_description_ident().to_string(),
            "PurchaseOrderDescriptionVariants"
        );
        assert_eq!(identities.story_id_literal(), "purchase-order-table-story");
        assert!(identities.has_filters());
        assert_eq!(
            identities.try_struct_name_ident().unwrap().to_string(),
            "PurchaseOrder"
        );
        assert_eq!(
            identities.try_snake_case_ident().unwrap().to_string(),
            "purchase_order"
        );
    }

    #[test]
    fn parse_ident_reports_the_derived_value_and_shape() {
        assert_eq!(
            parse_ident("field", "User", "valid_name".into())
                .unwrap()
                .to_string(),
            "valid_name"
        );

        let error = parse_ident("field", "User", "invalid field".into()).unwrap_err();
        match error {
            TableCodegenError::InvalidIdentifier {
                kind,
                struct_name,
                value,
                source,
            } => {
                assert_eq!(kind, "field");
                assert_eq!(struct_name, "User");
                assert_eq!(value, "invalid field");
                assert!(!source.to_string().is_empty());
            },
            other => panic!("unexpected error: {other}"),
        }
    }
}
