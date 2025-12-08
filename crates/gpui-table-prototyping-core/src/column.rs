//! Column-level code generation utilities.

use gpui_table_core::registry::ColumnVariant;
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
        self.variant.field_type
    }

    /// Parse field type as syn::Type
    pub fn field_type_syn(&self) -> syn::Type {
        syn::parse_str(self.variant.field_type).unwrap_or_else(|_| {
            // If parsing fails, wrap in angle brackets to handle generics
            syn::parse_str(&format!("{}", self.variant.field_type))
                .unwrap_or_else(|_| panic!("Failed to parse type: {}", self.variant.field_type))
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
    pub fn fixed(&self) -> &gpui_table_core::registry::ColumnFixed {
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
