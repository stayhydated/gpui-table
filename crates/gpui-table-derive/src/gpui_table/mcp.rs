use crate::gpui_table::meta::{FilterFieldMeta, McpToolOptions};
use crate::mcp_handlers::resolve_crate_path;

use component_shape_codegen::{McpToolMetadataParts, mcp_tool_metadata_tokens};
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, Ident, Path};

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

    let filter_descriptors: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|field| filter_descriptor_tokens(&facade_crate, field))
        .collect();
    let filter_decoders: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|field| filter_decode_tokens(&facade_crate, field))
        .collect();

    Ok(quote! {
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
    field: &FilterFieldMeta,
) -> proc_macro2::TokenStream {
    let field_name = field.field_ident.to_string();
    let field_type = field
        .filter_config
        .field_type()
        .to_token_stream()
        .to_string();
    let shape = field.filter_config.shape();

    quote! {
        #facade_crate::mcp::McpTableFilter::for_shape::<#shape>(
            #field_name,
            #facade_crate::schema::registry::RustType::from_macro_tokens_unchecked(#field_type),
        )
    }
}

fn filter_decode_tokens(facade_crate: &Path, field: &FilterFieldMeta) -> proc_macro2::TokenStream {
    let field_ident = &field.field_ident;
    let field_name = field.field_ident.to_string();
    let shape = field.filter_config.shape();

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
