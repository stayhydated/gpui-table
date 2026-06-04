#[cfg(feature = "inventory")]
use component_shape_codegen::rust_path_metadata_tokens;
use component_shape_codegen::{
    ResolvedComponentShape as SharedResolvedComponentShape, ShapeOptions as SharedShapeOptions,
    shape_type_assertion_tokens_with_suffixes,
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens as _, quote};
use syn::Path;

#[derive(Clone, Debug)]
pub(crate) struct FilterShapeOptions {
    inner: SharedShapeOptions,
}

impl FilterShapeOptions {
    pub(crate) fn from_shape_with_span(shape: Path, span: Span) -> Self {
        Self {
            inner: SharedShapeOptions::from_shape_with_span(shape, span),
        }
    }

    pub(crate) fn resolve(&self, field_name: String, field_type: syn::Type) -> ResolvedFilterShape {
        ResolvedFilterShape {
            inner: self.inner.resolve(field_name, field_type),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedFilterShape {
    inner: SharedResolvedComponentShape,
}

impl ResolvedFilterShape {
    pub(crate) fn shape(&self) -> &syn::Path {
        self.inner.shape()
    }

    pub(crate) fn field_type(&self) -> &syn::Type {
        self.inner.field_type()
    }

    pub(crate) fn field_name(&self) -> &str {
        self.inner.field_name()
    }

    pub(crate) fn span(&self) -> Span {
        self.inner.span()
    }

    pub(crate) fn kind_label(&self) -> &'static str {
        "shape"
    }

    pub(crate) fn component_type_tokens(&self) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::Component
        }
    }

    pub(crate) fn raw_value_type_tokens(&self) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::RawValue
        }
    }

    pub(crate) fn generated_value_type_tokens(&self) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::FilterValue
        }
    }

    #[cfg(feature = "inventory")]
    pub(crate) fn registry_kind_tokens(&self) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::FILTER_TYPE
        }
    }

    pub(crate) fn runtime_filter_type_expr(&self) -> TokenStream {
        let shape = self.shape();
        let field_type = self.field_type();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShapeFor<#field_type>>::filter_type()
        }
    }

    pub(crate) fn read_raw_value_expr(&self, field_ident: &syn::Ident) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::read_value(
                &self.#field_ident,
                cx,
            )
        }
    }

    pub(crate) fn wrap_raw_value_expr(&self, raw_value_expr: TokenStream) -> TokenStream {
        let shape = self.shape();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::wrap_value(#raw_value_expr)
        }
    }

    pub(crate) fn matches_field_expr(&self, field_ident: &syn::Ident) -> TokenStream {
        let shape = self.shape();
        let field_type = self.field_type();
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShapeFor<#field_type>>::matches_field(
                &self.#field_ident,
                &filters.#field_ident,
            )
        }
    }

    pub(crate) fn validate_feature_gate(&self) -> syn::Result<()> {
        if !cfg!(feature = "rust_decimal") && self.is_builtin_shape("NumberRangeFilter") {
            return Err(syn::Error::new(
                self.span(),
                "`gpui_table_component::NumberRangeFilter` requires enabling the `gpui-table/rust_decimal` feature",
            ));
        }

        if !cfg!(feature = "chrono") && self.is_builtin_shape("DateRangeFilter") {
            return Err(syn::Error::new(
                self.span(),
                "`gpui_table_component::DateRangeFilter` requires enabling the `gpui-table/chrono` feature",
            ));
        }

        if !cfg!(feature = "spacetimedb") && self.is_spacetimedb_range_shape() {
            let shape = if self.is_builtin_shape("NumberRangeFilter") {
                "gpui_table_component::NumberRangeFilter"
            } else {
                "gpui_table_component::DateRangeFilter"
            };
            return Err(syn::Error::new(
                self.span(),
                format!(
                    "`{shape}` on `{}` requires enabling the `gpui-table/spacetimedb` feature",
                    self.compact_field_type(),
                ),
            ));
        }

        Ok(())
    }

    pub(crate) fn type_check_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let field_type = self.field_type();
        let span = self.span();
        let runtime_crate: syn::Path = syn::parse_quote!(gpui_table::runtime);

        shape_type_assertion_tokens_with_suffixes(
            "gpui_table",
            self.field_name(),
            shape,
            field_type,
            span,
            [
                quote! { #runtime_crate::shape::DeclaredGpuiTableFilterShape },
                quote! { #runtime_crate::shape::GpuiTableFilterShape },
            ],
            quote! { #runtime_crate::shape::GpuiTableFilterShapeFor },
            "declared_filter_shape",
            "filter_shape_field_support",
        )
    }

    #[cfg(feature = "inventory")]
    pub(crate) fn shape_path_tokens(&self) -> TokenStream {
        rust_path_metadata_tokens(
            quote! { gpui_table::schema::registry::RustPath },
            self.shape(),
        )
    }

    #[cfg(feature = "inventory")]
    pub(crate) fn component_path_tokens(&self) -> TokenStream {
        let component_type = self.component_type_tokens();
        let component_type_string = component_type.to_string();
        quote! {
            gpui_table::schema::registry::RustPath::from_macro_tokens_unchecked(#component_type_string)
        }
    }

    fn is_builtin_shape(&self, component_name: &str) -> bool {
        let shape = self.shape();
        let segments = shape
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();

        matches!(
            segments.as_slice(),
            [root, name] if root == "gpui_table_component" && name == component_name
        )
    }

    fn is_spacetimedb_range_shape(&self) -> bool {
        (self.is_builtin_shape("DateRangeFilter") || self.is_builtin_shape("NumberRangeFilter"))
            && (self
                .compact_field_type()
                .contains("spacetimedb_lib::Timestamp")
                || self
                    .compact_field_type()
                    .contains("spacetimedb_lib::TimeDuration"))
    }

    fn compact_field_type(&self) -> String {
        self.field_type()
            .to_token_stream()
            .to_string()
            .replace(" :: ", "::")
            .replace(" < ", "<")
            .replace(" >", ">")
            .replace(" , ", ", ")
    }
}
