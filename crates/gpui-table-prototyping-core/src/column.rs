use gpui_table_schema::registry::{ColumnFixed, ColumnVariant};
use heck::ToPascalCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// A code generator for a single table column.
pub trait ColumnCodeGenerator {
    /// Generate code for accessing this column's value
    fn value_accessor(&self, column: &ColumnVariant) -> TokenStream;

    /// Generate a render child element for this column
    fn render_child(&self, column: &ColumnVariant) -> TokenStream;

    /// Generate any additional imports needed for this column type
    fn additional_imports(&self, column: &ColumnVariant) -> Option<TokenStream>;
}

/// Default column code generator
pub struct DefaultColumnGenerator;

impl ColumnCodeGenerator for DefaultColumnGenerator {
    fn value_accessor(&self, column: &ColumnVariant) -> TokenStream {
        let field_ident = format_ident!("{}", column.field_name);
        quote! { &row.#field_ident }
    }

    fn render_child(&self, column: &ColumnVariant) -> TokenStream {
        let field_ident = format_ident!("{}", column.field_name);
        let title = column.title;

        quote! {
            .child(format!("{}: {:?}", #title, row.#field_ident))
        }
    }

    fn additional_imports(&self, _column: &ColumnVariant) -> Option<TokenStream> {
        None
    }
}

/// Wrapper for ColumnVariant with additional utilities
pub struct ColumnInfo<'a> {
    pub variant: &'a ColumnVariant,
}

impl<'a> ColumnInfo<'a> {
    pub fn new(variant: &'a ColumnVariant) -> Self {
        Self { variant }
    }

    /// Field name as identifier
    pub fn field_ident(&self) -> syn::Ident {
        format_ident!("{}", self.variant.field_name)
    }

    /// Field name in PascalCase for enum variants
    pub fn pascal_case_name(&self) -> String {
        self.variant.field_name.to_pascal_case()
    }

    /// Field name as PascalCase identifier
    pub fn pascal_case_ident(&self) -> syn::Ident {
        format_ident!("{}", self.pascal_case_name())
    }

    /// The column title
    pub fn title(&self) -> &'static str {
        self.variant.title
    }

    /// The field type as a string
    pub fn field_type(&self) -> &'static str {
        self.variant.field_type.as_str()
    }

    /// Parse field type as syn::Type
    pub fn field_type_syn(&self) -> syn::Type {
        self.try_field_type_syn()
            .expect("valid field type in gpui-table shape metadata")
    }

    /// Fallible version of [`ColumnInfo::field_type_syn`] for user-facing tooling.
    pub fn try_field_type_syn(&self) -> syn::Result<syn::Type> {
        self.variant.field_type.parse().map_err(|err| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "failed to parse field type `{}`: {err}",
                    self.variant.field_type.as_str()
                ),
            )
        })
    }

    /// Column width
    pub fn width(&self) -> f32 {
        self.variant.width
    }

    /// Whether column is sortable
    pub fn sortable(&self) -> bool {
        self.variant.sortable
    }

    /// Get the fixed position
    pub fn fixed(&self) -> &ColumnFixed {
        &self.variant.fixed
    }

    /// Generate value accessor code
    pub fn generate_value_accessor(&self) -> TokenStream {
        let field_ident = self.field_ident();
        quote! { row.#field_ident.clone() }
    }

    /// Generate a simple display child
    pub fn generate_display_child(&self) -> TokenStream {
        let field_ident = self.field_ident();
        let title = self.title();
        quote! {
            .child(format!("{}: {:?}", #title, row.#field_ident))
        }
    }
}

/// Iterator over columns with utilities
pub struct ColumnIterator<'a> {
    columns: std::slice::Iter<'a, ColumnVariant>,
    index: usize,
}

impl<'a> ColumnIterator<'a> {
    pub fn new(columns: &'a [ColumnVariant]) -> Self {
        Self {
            columns: columns.iter(),
            index: 0,
        }
    }
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = (usize, ColumnInfo<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.columns.next().map(|v| {
            let index = self.index;
            self.index += 1;
            (index, ColumnInfo::new(v))
        })
    }
}

/// Extension trait for slices of ColumnVariant
pub trait ColumnSliceExt {
    fn column_iter(&self) -> ColumnIterator<'_>;
    fn sortable_columns(&self) -> Vec<&ColumnVariant>;
}

impl ColumnSliceExt for [ColumnVariant] {
    fn column_iter(&self) -> ColumnIterator<'_> {
        ColumnIterator::new(self)
    }

    fn sortable_columns(&self) -> Vec<&ColumnVariant> {
        self.iter().filter(|c| c.sortable).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ColumnCodeGenerator as _, ColumnInfo, ColumnIterator, ColumnSliceExt as _,
        DefaultColumnGenerator,
    };
    use gpui_table_schema::registry::{ColumnFixed, ColumnVariant, RustType};

    static COLUMNS: [ColumnVariant; 2] = [
        ColumnVariant::new(
            "display_name",
            RustType::from_macro_tokens_unchecked("String"),
            "Display name",
            240.0,
            true,
            ColumnFixed::Left,
        ),
        ColumnVariant::new(
            "score",
            RustType::from_macro_tokens_unchecked("Option < i64 >"),
            "Score",
            80.0,
            false,
            ColumnFixed::Right,
        ),
    ];

    #[test]
    fn default_generator_emits_accessor_and_debug_child_contracts() {
        let generator = DefaultColumnGenerator;

        assert_eq!(
            generator.value_accessor(&COLUMNS[0]).to_string(),
            "& row . display_name"
        );
        assert_eq!(
            generator.render_child(&COLUMNS[0]).to_string(),
            ". child (format ! (\"{}: {:?}\" , \"Display name\" , row . display_name))"
        );
        assert!(generator.additional_imports(&COLUMNS[0]).is_none());
    }

    #[test]
    fn column_info_exposes_metadata_and_generated_identifiers() {
        let info = ColumnInfo::new(&COLUMNS[0]);

        assert_eq!(info.field_ident().to_string(), "display_name");
        assert_eq!(info.pascal_case_name(), "DisplayName");
        assert_eq!(info.pascal_case_ident().to_string(), "DisplayName");
        assert_eq!(info.title(), "Display name");
        assert_eq!(info.field_type(), "String");
        let field_type = info.field_type_syn();
        assert_eq!(quote::quote!(#field_type).to_string(), "String");
        assert!(info.try_field_type_syn().is_ok());
        assert_eq!(info.width(), 240.0);
        assert!(info.sortable());
        assert_eq!(info.fixed(), &ColumnFixed::Left);
        assert_eq!(
            info.generate_value_accessor().to_string(),
            "row . display_name . clone ()"
        );
        assert_eq!(
            info.generate_display_child().to_string(),
            ". child (format ! (\"{}: {:?}\" , \"Display name\" , row . display_name))"
        );
    }

    #[test]
    fn invalid_field_types_return_contextual_parse_errors() {
        let invalid = ColumnVariant::new(
            "bad",
            RustType::from_macro_tokens_unchecked("Vec<"),
            "Bad",
            1.0,
            false,
            ColumnFixed::None,
        );
        let error = ColumnInfo::new(&invalid).try_field_type_syn().unwrap_err();

        assert!(
            error
                .to_string()
                .contains("failed to parse field type `Vec<`")
        );
    }

    #[test]
    fn column_iterators_preserve_indexes_and_select_sortable_columns() {
        let iterated = ColumnIterator::new(&COLUMNS)
            .map(|(index, info)| (index, info.field_ident().to_string()))
            .collect::<Vec<_>>();
        assert_eq!(
            iterated,
            [(0, "display_name".to_string()), (1, "score".to_string())]
        );

        let via_extension = COLUMNS
            .column_iter()
            .map(|(index, info)| (index, info.title()))
            .collect::<Vec<_>>();
        assert_eq!(via_extension, [(0, "Display name"), (1, "Score")]);
        let sortable = COLUMNS.sortable_columns();
        assert_eq!(sortable.len(), 1);
        assert_eq!(sortable[0].field_name, "display_name");
    }
}
