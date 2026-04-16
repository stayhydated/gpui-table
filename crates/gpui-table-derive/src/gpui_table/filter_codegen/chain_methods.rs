use crate::components::{FilterComponents, TextValidation};

use quote::quote;

/// Generate chain method calls for filter options.
pub(in crate::gpui_table) fn generate_filter_chain_methods(
    filter: &FilterComponents,
) -> proc_macro2::TokenStream {
    match filter {
        FilterComponents::Text(opts) => {
            let mut chain = quote! {};

            // Generate validation method if specified
            if let Some(ref validation) = opts.validate {
                let validation_chain = match validation {
                    TextValidation::Alphabetic => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.alphabetic_only(cx);
                    },
                    TextValidation::Numeric => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.numeric_only(cx);
                    },
                    TextValidation::Alphanumeric => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.alphanumeric_only(cx);
                    },
                    TextValidation::Custom(path) => quote! {
                        use gpui_table::runtime::generated_filters::text_filter::TextFilterExt as _;
                        let filter = filter.validate(#path, cx);
                    },
                };
                chain = quote! { #chain #validation_chain };
            }

            chain
        },
        FilterComponents::NumberRange(opts) => {
            let mut chain = quote! {};

            // Generate .range() call if min or max is specified
            if opts.min.is_some() || opts.max.is_some() {
                #[cfg(feature = "rust_decimal")]
                let min_expr = opts
                    .min
                    .as_ref()
                    .map(|value| value.decimal_tokens("min"))
                    .unwrap_or_else(|| {
                        quote! {
                            gpui_table::__deps::rust_decimal::Decimal::from_i128_with_scale(0, 0)
                        }
                    });
                #[cfg(feature = "rust_decimal")]
                let max_expr = opts
                    .max
                    .as_ref()
                    .map(|value| value.decimal_tokens("max"))
                    .unwrap_or_else(|| {
                        quote! {
                            gpui_table::__deps::rust_decimal::Decimal::from_i128_with_scale(100, 0)
                        }
                    });

                #[cfg(not(feature = "rust_decimal"))]
                let (min_expr, max_expr) = (quote! {}, quote! {});

                chain = quote! {
                    #chain
                    use gpui_table::runtime::generated_filters::number_range_filter::NumberRangeFilterExt as _;
                    let filter = filter.range(
                        #min_expr,
                        #max_expr,
                        cx,
                    );
                };
            }

            // Generate .step() call if step is specified
            if let Some(step_val) = opts.step.as_ref() {
                #[cfg(feature = "rust_decimal")]
                let step_expr = step_val.decimal_tokens("step");
                #[cfg(not(feature = "rust_decimal"))]
                let step_expr = {
                    let _ = step_val;
                    quote! {}
                };

                chain = quote! {
                    #chain
                    let filter = filter.step(#step_expr, cx);
                };
            }

            chain
        },
        FilterComponents::DateRange(_opts) => {
            // Date range filter has no configurable options yet
            quote! {}
        },
        FilterComponents::Faceted(opts) => {
            let mut chain = quote! {};

            // Generate .searchable() call if enabled
            if opts.searchable {
                chain = quote! {
                    #chain
                    use gpui_table::runtime::generated_filters::faceted_filter::FacetedFilterExt as _;
                    let filter = filter.searchable(cx);
                };
            }

            chain
        },
        FilterComponents::InfiniteFaceted(_opts) => {
            quote! {}
        },
    }
}
