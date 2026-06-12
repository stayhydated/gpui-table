#[cfg(feature = "inventory")]
use component_shape_codegen::rust_path_metadata_tokens;
use component_shape_codegen::{
    ResolvedComponentShape as SharedResolvedComponentShape, ShapeOptions as SharedShapeOptions,
    shape_type_assertion_tokens_with_suffixes,
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens as _, quote};
use syn::{Path, Type, parse_quote};

#[derive(Clone, Debug)]
pub(crate) struct FilterShapeOptions {
    kind: FilterShapeOptionsKind,
}

#[derive(Clone, Debug)]
enum FilterShapeOptionsKind {
    Explicit(SharedShapeOptions),
    Inferred { span: Span },
}

impl FilterShapeOptions {
    pub(crate) fn from_shape_with_span(shape: Path, span: Span) -> Self {
        Self {
            kind: FilterShapeOptionsKind::Explicit(SharedShapeOptions::from_shape_with_span(
                shape, span,
            )),
        }
    }

    pub(crate) fn inferred(span: Span) -> Self {
        Self {
            kind: FilterShapeOptionsKind::Inferred { span },
        }
    }

    pub(crate) fn resolve(&self, field_name: String, field_type: syn::Type) -> ResolvedFilterShape {
        let inner = match &self.kind {
            FilterShapeOptionsKind::Explicit(inner) => inner.clone(),
            FilterShapeOptionsKind::Inferred { span } => {
                SharedShapeOptions::from_shape_with_span(infer_filter_shape(&field_type), *span)
            },
        };

        ResolvedFilterShape {
            inner: inner.resolve(field_name, field_type),
        }
    }
}

fn infer_filter_shape(field_type: &Type) -> Path {
    let scalar_type = peel_option(peel_type_wrappers(field_type))
        .map(peel_type_wrappers)
        .unwrap_or_else(|| peel_type_wrappers(field_type));

    if is_string_type(scalar_type) {
        return parse_quote!(gpui_table::runtime::shape::TextFilter);
    }

    if is_number_type(scalar_type) {
        return parse_quote!(gpui_table::runtime::shape::NumberRangeFilter);
    }

    if is_date_type(scalar_type) {
        return parse_quote!(gpui_table::runtime::shape::DateRangeFilter);
    }

    let facet_type = faceted_value_type(field_type).unwrap_or_else(|| field_type.clone());
    parse_quote!(gpui_table::runtime::shape::FacetedFilter::<#facet_type>)
}

fn faceted_value_type(field_type: &Type) -> Option<Type> {
    let ty = peel_option(peel_type_wrappers(field_type))
        .map(peel_type_wrappers)
        .unwrap_or_else(|| peel_type_wrappers(field_type));

    if let Some(item) = single_type_argument_for_ident(ty, "Vec") {
        return Some(item.clone());
    }

    Some(ty.clone())
}

fn is_string_type(ty: &Type) -> bool {
    path_last_ident(ty).is_some_and(|ident| matches!(ident.as_str(), "String" | "str"))
}

fn is_number_type(ty: &Type) -> bool {
    path_last_ident(ty).is_some_and(|ident| {
        matches!(
            ident.as_str(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "Decimal"
                | "TimeDuration"
        )
    })
}

fn is_date_type(ty: &Type) -> bool {
    path_last_ident(ty).is_some_and(|ident| {
        matches!(
            ident.as_str(),
            "NaiveDate" | "NaiveDateTime" | "DateTime" | "Date" | "Timestamp" | "Zoned"
        )
    })
}

fn path_last_ident(ty: &Type) -> Option<String> {
    let Type::Path(path) = peel_type_wrappers(ty) else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn peel_option(ty: &Type) -> Option<&Type> {
    single_type_argument_for_ident(ty, "Option")
}

fn single_type_argument_for_ident<'a>(ty: &'a Type, ident: &str) -> Option<&'a Type> {
    let Type::Path(path) = peel_type_wrappers(ty) else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != ident {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut arguments = arguments.args.iter();
    let syn::GenericArgument::Type(ty) = arguments.next()? else {
        return None;
    };
    if arguments.next().is_some() {
        return None;
    }
    Some(ty)
}

fn peel_type_wrappers(mut ty: &Type) -> &Type {
    loop {
        match ty {
            Type::Group(group) => ty = &group.elem,
            Type::Paren(paren) => ty = &paren.elem,
            Type::Reference(reference) => ty = &reference.elem,
            _ => return ty,
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
        ) || matches!(
            segments.as_slice(),
            [root, runtime, shape, name]
                if root == "gpui_table"
                    && runtime == "runtime"
                    && shape == "shape"
                    && name == component_name
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    fn inferred_shape_tokens(field_type: Type) -> String {
        infer_filter_shape(&field_type)
            .to_token_stream()
            .to_string()
    }

    #[test]
    fn infer_filter_shape_uses_text_for_strings() {
        assert_eq!(
            inferred_shape_tokens(parse_quote!(Option<String>)),
            "gpui_table :: runtime :: shape :: TextFilter"
        );
    }

    #[test]
    fn infer_filter_shape_uses_number_ranges_for_numeric_values() {
        assert_eq!(
            inferred_shape_tokens(parse_quote!(u64)),
            "gpui_table :: runtime :: shape :: NumberRangeFilter"
        );
        assert_eq!(
            inferred_shape_tokens(parse_quote!(Option<rust_decimal::Decimal>)),
            "gpui_table :: runtime :: shape :: NumberRangeFilter"
        );
    }

    #[test]
    fn infer_filter_shape_uses_date_ranges_for_date_values() {
        assert_eq!(
            inferred_shape_tokens(parse_quote!(chrono::NaiveDate)),
            "gpui_table :: runtime :: shape :: DateRangeFilter"
        );
        assert_eq!(
            inferred_shape_tokens(parse_quote!(Option<spacetimedb_lib::Timestamp>)),
            "gpui_table :: runtime :: shape :: DateRangeFilter"
        );
    }

    #[test]
    fn infer_filter_shape_uses_facets_for_enum_like_values() {
        assert_eq!(
            inferred_shape_tokens(parse_quote!(Option<Vec<Role>>)),
            "gpui_table :: runtime :: shape :: FacetedFilter :: < Role >"
        );
        assert_eq!(
            inferred_shape_tokens(parse_quote!(Status)),
            "gpui_table :: runtime :: shape :: FacetedFilter :: < Status >"
        );
    }
}
