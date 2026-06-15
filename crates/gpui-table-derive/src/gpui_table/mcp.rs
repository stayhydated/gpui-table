use crate::gpui_table::meta::{FilterFieldMeta, McpToolOptions};
use crate::mcp_handlers::resolve_crate_path;

use component_shape_codegen::{McpToolMetadataParts, mcp_tool_metadata_tokens};
use koruma_derive_core::{ParsedValidatorUse, ValidatorTypeArg};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, Expr, Ident, Lit, LitStr, Path, UnOp, spanned::Spanned as _};

struct FilterDescriptorTokens {
    constants: Vec<TokenStream>,
    descriptor: TokenStream,
}

struct ValidationRuleTokenPlan {
    param_const: Option<TokenStream>,
    rule: TokenStream,
}

pub(super) fn generate_mcp_impl(
    struct_name: &Ident,
    table_id: &str,
    table_title: &str,
    filter_fields: &[FilterFieldMeta],
    mcp_tool_options: Option<&McpToolOptions>,
    original_input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(field) = filter_fields
        .iter()
        .find(|field| matches!(field.field_ident.to_string().as_str(), "limit" | "offset"))
    {
        let field_ident = &field.field_ident;
        return Err(syn::Error::new(
            field_ident.span(),
            "MCP table filters cannot be named `limit` or `offset`; those argument names are reserved for pagination",
        ));
    }

    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());
    let filter_values_type = quote! { #filter_values_name };
    let filters_const_ident = format_ident!("__{}GpuiTableMcpFilters", struct_name);
    let descriptor_fn_ident = format_ident!("__{}_gpui_table_mcp_descriptor", struct_name);
    let facade_crate = resolve_crate_path("gpui-table", "::gpui_table");
    let tool_metadata = tool_metadata_tokens(&facade_crate, mcp_tool_options, original_input)?;

    let filter_descriptor_tokens = filter_fields
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            filter_descriptor_tokens(&facade_crate, struct_name, field_index, field)
        })
        .collect::<Vec<_>>();
    let filter_validation_consts = filter_descriptor_tokens
        .iter()
        .flat_map(|descriptor| descriptor.constants.iter())
        .collect::<Vec<_>>();
    let filter_descriptors = filter_descriptor_tokens
        .iter()
        .map(|descriptor| &descriptor.descriptor)
        .collect::<Vec<_>>();
    let filter_decoders: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            filter_decode_tokens(&facade_crate, struct_name, field_index, field)
        })
        .collect();

    Ok(quote! {
        #(#filter_validation_consts)*

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub const #filters_const_ident: &[#facade_crate::mcp::McpTableFilter] = &[
            #(#filter_descriptors),*
        ];

        impl #facade_crate::mcp::McpTable for #struct_name {
            type FilterValues = #filter_values_type;

            fn descriptor() -> #facade_crate::mcp::McpTableDescriptor {
                #facade_crate::mcp::McpTableDescriptor::new(
                    stringify!(#struct_name),
                    #table_id,
                    #table_title,
                    #facade_crate::schema::registry::RustPath::from_macro_tokens_unchecked(
                        module_path!()
                    ),
                    #filters_const_ident,
                    #tool_metadata,
                )
            }

            fn decode_query(
                call: #facade_crate::mcp::McpToolCall
            ) -> Result<#facade_crate::mcp::TableQuery<Self>, #facade_crate::mcp::McpToolError> {
                let mut __gpui_table_filters = #filter_values_type::default();
                let mut __gpui_table_arguments = call.into_arguments();
                let __gpui_table_limit =
                    __gpui_table_arguments.take_present_tool_value::<usize>("limit")?;
                let __gpui_table_offset =
                    __gpui_table_arguments
                        .take_present_tool_value::<usize>("offset")?
                        .unwrap_or(0);
                #(#filter_decoders)*

                __gpui_table_arguments.finish()?;

                Ok(#facade_crate::mcp::TableQuery {
                    filters: __gpui_table_filters,
                    limit: __gpui_table_limit,
                    offset: __gpui_table_offset,
                })
            }
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #descriptor_fn_ident() -> #facade_crate::mcp::McpTableDescriptor {
            <#struct_name as #facade_crate::mcp::McpTable>::descriptor()
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpTableRegistration::new(#descriptor_fn_ident)
        }
    })
}

fn tool_metadata_tokens(
    facade_crate: &Path,
    options: Option<&McpToolOptions>,
    original_input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let mcp_crate: Path = syn::parse_quote!(#facade_crate::mcp);
    mcp_tool_metadata_tokens(
        &mcp_crate,
        &original_input.attrs,
        McpToolMetadataParts {
            name: options.and_then(|options| options.name.as_deref()),
            title: options.and_then(|options| options.title.as_deref()),
            description: options.and_then(|options| options.description.as_deref()),
            read_only: options.and_then(|options| options.read_only),
            destructive: options.and_then(|options| options.destructive),
            idempotent: options.and_then(|options| options.idempotent),
            open_world: options.and_then(|options| options.open_world),
        },
        original_input.ident.span(),
    )
}

fn filter_descriptor_tokens(
    facade_crate: &Path,
    struct_name: &Ident,
    field_index: usize,
    field: &FilterFieldMeta,
) -> FilterDescriptorTokens {
    let field_name = field.field_ident.to_string();
    let field_type = field
        .filter_config
        .field_type()
        .to_token_stream()
        .to_string();
    let shape = field.filter_config.shape();
    let (constants, validation_rules) =
        validation_rules_tokens(facade_crate, struct_name, field_index, field);
    let validation_rules = validation_rules.unwrap_or_else(|| quote! {});

    let descriptor = quote! {
        #facade_crate::mcp::McpTableFilter::for_shape::<#shape>(
            #field_name,
            #facade_crate::schema::registry::RustType::from_macro_tokens_unchecked(#field_type),
        )
        #validation_rules
    };

    FilterDescriptorTokens {
        constants,
        descriptor,
    }
}

fn filter_decode_tokens(
    facade_crate: &Path,
    struct_name: &Ident,
    field_index: usize,
    field: &FilterFieldMeta,
) -> proc_macro2::TokenStream {
    let field_ident = &field.field_ident;
    let field_name = field.field_ident.to_string();
    let shape = field.filter_config.shape();
    let validation = validation_decode_tokens(facade_crate, struct_name, field_index, field);

    if validation.is_some() {
        quote! {
            if let Some(__gpui_table_value) = __gpui_table_arguments.take_raw(#field_name) {
                __gpui_table_filters.#field_ident =
                    <#shape as #facade_crate::mcp::McpFilterShapeValidation>::decode_filter_with_validation(
                        #field_name,
                        #facade_crate::mcp::McpAny::from(__gpui_table_value),
                        |__gpui_table_filter_raw_value| {
                            #validation
                        },
                    )?;
            }
        }
    } else {
        quote! {
            if let Some(__gpui_table_value) = __gpui_table_arguments.take_raw(#field_name) {
                __gpui_table_filters.#field_ident =
                    <#shape as #facade_crate::mcp::McpFilterShape>::decode_filter(
                        #field_name,
                        #facade_crate::mcp::McpAny::from(__gpui_table_value),
                    )?;
            }
        }
    }
}

fn validation_rules_tokens(
    facade_crate: &Path,
    struct_name: &Ident,
    field_index: usize,
    field: &FilterFieldMeta,
) -> (Vec<TokenStream>, Option<TokenStream>) {
    let Some(validation) = field.validation.as_ref() else {
        return (Vec::new(), None);
    };
    if validation.is_empty() {
        return (Vec::new(), None);
    }

    let rules_const_ident = filter_validation_rules_const_ident(struct_name, field_index);
    let mut constants = Vec::new();
    let mut rules = Vec::new();

    for (rule_index, validator) in validation.validators().iter().enumerate() {
        let plan = validation_rule_token_plan(
            facade_crate,
            validator,
            struct_name,
            field_index,
            rule_index,
        );
        if let Some(param_const) = plan.param_const {
            constants.push(param_const);
        }
        rules.push(plan.rule);
    }
    if validation.is_newtype() {
        let field_type = field
            .filter_config
            .field_type()
            .to_token_stream()
            .to_string();
        let field_type = LitStr::new(&field_type, field.field_ident.span());
        rules.push(quote! {
            #facade_crate::mcp::McpValidationRule::new(
                #facade_crate::mcp::McpValidationScope::Filter,
                "newtype",
                #field_type,
                None,
                #facade_crate::mcp::McpValidationTypeArgMode::None,
                #facade_crate::mcp::MCP_VALIDATION_PARAMS_NONE,
            )
        });
    }

    constants.push(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #rules_const_ident: &[#facade_crate::mcp::McpValidationRule] = &[
            #(#rules),*
        ];
    });

    (
        constants,
        Some(quote! {
            .with_validation_rules(#rules_const_ident)
        }),
    )
}

fn validation_rule_token_plan(
    facade_crate: &Path,
    validator_use: &ParsedValidatorUse,
    struct_name: &Ident,
    field_index: usize,
    rule_index: usize,
) -> ValidationRuleTokenPlan {
    let validator = validator_use.validator();
    let validator_name = LitStr::new(&validator.name().to_string(), validator.name().span());
    let validator_path = LitStr::new(&validator.path_name(), validator.path().span());
    let label_tokens = validator_use
        .label()
        .map(|label| {
            let label = LitStr::new(&label.to_string(), validator_use.source_span());
            quote! { Some(#label) }
        })
        .unwrap_or_else(|| quote! { None });
    let type_arg_tokens = match validator.type_arg() {
        ValidatorTypeArg::None => quote! { #facade_crate::mcp::McpValidationTypeArgMode::None },
        ValidatorTypeArg::Infer => quote! { #facade_crate::mcp::McpValidationTypeArgMode::Infer },
        ValidatorTypeArg::Explicit(_) => {
            quote! { #facade_crate::mcp::McpValidationTypeArgMode::Explicit }
        },
    };
    let params = validator
        .setter_calls()
        .iter()
        .flat_map(|call| {
            let method_name = call.method().to_string();
            let arg_count = call.args().len();
            call.args()
                .iter()
                .enumerate()
                .map(move |(index, arg)| {
                    let param_name = if arg_count == 1 {
                        method_name.clone()
                    } else {
                        format!("{method_name}[{index}]")
                    };
                    let param_name = LitStr::new(&param_name, call.method().span());
                    let expr = arg.as_expr();
                    if let Some(literal) = literal_expr_string(expr) {
                        let literal = LitStr::new(&literal, expr.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::literal(
                                #param_name,
                                #literal
                            )
                        }
                    } else {
                        let expr = LitStr::new(&expr.to_token_stream().to_string(), expr.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::expr(
                                #param_name,
                                #expr
                            )
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let params_ident = filter_validation_params_const_ident(struct_name, field_index, rule_index);
    let (param_const, params_tokens) = if params.is_empty() {
        (
            None,
            quote! { #facade_crate::mcp::MCP_VALIDATION_PARAMS_NONE },
        )
    } else {
        (
            Some(quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                const #params_ident: &[#facade_crate::mcp::McpValidationParam] = &[
                    #(#params),*
                ];
            }),
            quote! { #params_ident },
        )
    };

    let rule = quote! {
        #facade_crate::mcp::McpValidationRule::new(
            #facade_crate::mcp::McpValidationScope::Filter,
            #validator_name,
            #validator_path,
            #label_tokens,
            #type_arg_tokens,
            #params_tokens,
        )
    };

    ValidationRuleTokenPlan { param_const, rule }
}

fn validation_decode_tokens(
    facade_crate: &Path,
    struct_name: &Ident,
    field_index: usize,
    field: &FilterFieldMeta,
) -> Option<TokenStream> {
    let validation = field.validation.as_ref()?;
    if validation.is_empty() {
        return None;
    }

    let field_name = field.field_ident.to_string();
    let field_type = field.filter_config.field_type();
    let shape = field.filter_config.shape();
    let rules_const_ident = filter_validation_rules_const_ident(struct_name, field_index);
    let newtype_rule_index = validation.validators().len();
    let checks = validation
        .validators()
        .iter()
        .enumerate()
        .map(|(rule_index, validator)| {
            let builder = validator_builder_expr(validator);
            quote! {
                {
                    let __gpui_table_validator_builder = #builder;
                    let __gpui_table_validation_rule = #rules_const_ident[#rule_index];
                    let __gpui_table_validator =
                        ::koruma::__private::BuildValidator::build_validator(
                            ::koruma::__private::CaptureValueRef::capture_value_ref(
                                __gpui_table_validator_builder,
                                __gpui_table_filter_raw_value,
                            )
                        );
                    if !::koruma::Validate::validate(
                        &__gpui_table_validator,
                        __gpui_table_filter_raw_value,
                    ) {
                        __gpui_table_validation_issues.push(
                            #facade_crate::mcp::McpValidationIssue::for_filter_rule(
                                #field_name,
                                __gpui_table_validation_rule,
                                ::std::format!(
                                    "{} validation failed",
                                    __gpui_table_validation_rule.validator(),
                                ),
                            )
                        );
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    let newtype_check = validation.is_newtype().then(|| {
        quote! {
            {
                let __gpui_table_validation_rule = #rules_const_ident[#newtype_rule_index];
                if !<#shape as #facade_crate::mcp::McpKorumaNewtypeFilterValidation<
                    #field_type
                >>::validate_koruma_newtype_filter(__gpui_table_filter_raw_value) {
                    __gpui_table_validation_issues.push(
                        #facade_crate::mcp::McpValidationIssue::for_filter_rule(
                            #field_name,
                            __gpui_table_validation_rule,
                            "newtype validation failed",
                        )
                    );
                }
            }
        }
    });

    Some(quote! {
        let mut __gpui_table_validation_issues = ::std::vec::Vec::new();
        #(#checks)*
        #newtype_check
        if __gpui_table_validation_issues.is_empty() {
            Ok(())
        } else {
            Err(#facade_crate::mcp::validation_issues_error(
                __gpui_table_validation_issues
            ))
        }
    })
}

fn validator_builder_expr(validator_use: &ParsedValidatorUse) -> TokenStream {
    let validator = validator_use.validator();
    let validator_path = validator.path();
    let builder_type = match validator.type_arg() {
        ValidatorTypeArg::None => quote! { #validator_path },
        ValidatorTypeArg::Infer => quote! { #validator_path::<_> },
        ValidatorTypeArg::Explicit(ty) => quote! { #validator_path::<#ty> },
    };
    let mut setter_calls = validator.setter_calls().iter();
    let Some(first_call) = setter_calls.next() else {
        return quote! { #builder_type::__koruma_builder() };
    };

    let first_method = first_call.method();
    let first_args = first_call
        .args()
        .iter()
        .map(|arg| arg.as_expr())
        .collect::<Vec<_>>();
    let rest_calls = setter_calls
        .map(|call| {
            let method = call.method();
            let args = call
                .args()
                .iter()
                .map(|arg| arg.as_expr())
                .collect::<Vec<_>>();
            quote! { .#method(#(#args),*) }
        })
        .collect::<Vec<_>>();

    quote! {
        #builder_type::#first_method(#(#first_args),*)
            #(#rest_calls)*
    }
}

fn filter_validation_rules_const_ident(struct_name: &Ident, field_index: usize) -> Ident {
    format_ident!("__{}GpuiTableMcpValidationRules{field_index}", struct_name)
}

fn filter_validation_params_const_ident(
    struct_name: &Ident,
    field_index: usize,
    rule_index: usize,
) -> Ident {
    format_ident!(
        "__{}GpuiTableMcpValidationParams{field_index}_{rule_index}",
        struct_name
    )
}

fn literal_expr_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(lit) => literal_string(&lit.lit),
        Expr::Unary(unary)
            if matches!(unary.op, UnOp::Neg(_))
                && matches!(
                    unary.expr.as_ref(),
                    Expr::Lit(expr_lit)
                        if matches!(expr_lit.lit, Lit::Int(_) | Lit::Float(_))
                ) =>
        {
            let Expr::Lit(lit) = unary.expr.as_ref() else {
                return None;
            };
            literal_string(&lit.lit).map(|literal| format!("-{literal}"))
        },
        _ => None,
    }
}

fn literal_string(lit: &Lit) -> Option<String> {
    match lit {
        Lit::Int(lit) => Some(lit.base10_digits().to_string()),
        Lit::Float(lit) => Some(lit.base10_digits().to_string()),
        Lit::Bool(lit) => Some(lit.value.to_string()),
        Lit::Str(lit) => Some(lit.value()),
        _ => None,
    }
}
