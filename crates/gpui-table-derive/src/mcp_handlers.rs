use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, FnArg, GenericArgument, ItemFn, PathArguments, ReturnType,
    Type, TypePath, spanned::Spanned,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum HandlerKind {
    Query,
    RowSource,
    InfallibleRowSource,
}

pub fn expand_query(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`mcp_query` does not accept arguments; infer the table from a `TableQuery<Row>` parameter or a `Result<Vec<Row>, E>` local-source return",
        ));
    }

    let item_fn: ItemFn = syn::parse2(item)?;
    let (handler_kind, table_type, error_type) = infer_handler(&item_fn)?;

    let fn_ident = &item_fn.sig.ident;
    let facade_crate = resolve_crate_path("gpui-table", "::gpui_table");
    let register_ident = match handler_kind {
        HandlerKind::Query => format_ident!("__gpui_table_mcp_register_query_{fn_ident}"),
        HandlerKind::RowSource | HandlerKind::InfallibleRowSource => {
            format_ident!("__gpui_table_mcp_register_row_source_{fn_ident}")
        },
    };
    let is_async = item_fn.sig.asyncness.is_some();
    let expected_response_type = if matches!(handler_kind, HandlerKind::Query) {
        quote!(#facade_crate::mcp::TableQueryResult<#table_type>)
    } else {
        quote!()
    };
    let response_assertion_tokens = match handler_kind {
        HandlerKind::Query => quote! {
            fn __gpui_table_mcp_assert_serialize<T: #facade_crate::mcp::Serialize>() {}
            __gpui_table_mcp_assert_serialize::<#expected_response_type>();
        },
        HandlerKind::RowSource | HandlerKind::InfallibleRowSource => quote! {
            fn __gpui_table_mcp_assert_serialize<T: #facade_crate::mcp::Serialize>() {}
            __gpui_table_mcp_assert_serialize::<#table_type>();
        },
    };

    let register_call = match (handler_kind, is_async) {
        (HandlerKind::Query, false) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .query(move |query| -> Result<#expected_response_type, #error_type> {
                        #fn_ident(query)
                    })
            }
        },
        (HandlerKind::Query, true) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .query_async(move |query| async move {
                        #fn_ident(query).await
                    })
            }
        },
        (HandlerKind::RowSource, false) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .row_source(#fn_ident)
            }
        },
        (HandlerKind::RowSource, true) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .row_source_async(#fn_ident)
            }
        },
        (HandlerKind::InfallibleRowSource, false) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .row_source(move || -> Result<Vec<#table_type>, #error_type> {
                        Ok(#fn_ident())
                    })
            }
        },
        (HandlerKind::InfallibleRowSource, true) => {
            quote! {
                #facade_crate::mcp::table::<#table_type>(server)
                    .row_source_async(move || async move {
                        Ok::<Vec<#table_type>, #error_type>(#fn_ident().await)
                    })
            }
        },
    };

    Ok(quote! {
        #item_fn

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #register_ident(
            server: &mut #facade_crate::mcp::McpServer
        ) -> Result<(), #facade_crate::mcp::McpToolError> {
            fn __gpui_table_mcp_assert_table<T: #facade_crate::mcp::McpTable>() {}
            __gpui_table_mcp_assert_table::<#table_type>();
            fn __gpui_table_mcp_assert_error<T: ::core::fmt::Display>() {}
            __gpui_table_mcp_assert_error::<#error_type>();
            #response_assertion_tokens
            #register_call
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpQueryHandlerRegistration::new(#register_ident)
        }
    })
}

fn parse_result_return_type(output: &ReturnType) -> syn::Result<(Type, Type)> {
    let ReturnType::Type(_, ty) = output else {
        return Err(syn::Error::new(
            output.span(),
            "mcp_query handlers must return Result<T, E> for explicit MCP error handling",
        ));
    };

    let Type::Path(TypePath { path, qself }) = ty.as_ref() else {
        return Err(syn::Error::new(
            output.span(),
            "mcp_query handlers must return Result<T, E> for explicit MCP error handling",
        ));
    };
    if qself.is_some() {
        return Err(syn::Error::new(
            output.span(),
            "mcp_query handlers must return Result<T, E> for explicit MCP error handling",
        ));
    }

    parse_result_type(path).ok_or_else(|| {
        syn::Error::new(
            output.span(),
            "mcp_query handlers must return Result<T, E> for explicit MCP error handling",
        )
    })
}

pub(crate) fn resolve_crate_path(package_name: &str, fallback: &str) -> syn::Path {
    let path = match crate_name(package_name) {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(name)) => format!("::{name}"),
        Err(_) => fallback.to_string(),
    };

    syn::parse_str(&path).expect("crate path resolver produced a valid Rust path")
}

fn infer_handler(item_fn: &ItemFn) -> syn::Result<(HandlerKind, Type, Type)> {
    let has_receiver = item_fn
        .sig
        .inputs
        .iter()
        .any(|argument| matches!(argument, FnArg::Receiver(_)));
    if has_receiver {
        return Err(syn::Error::new(
            item_fn.sig.ident.span(),
            "mcp_query handlers cannot be methods; use a free function",
        ));
    }

    let typed_arguments: Vec<_> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Typed(argument) => Some(argument),
            FnArg::Receiver(_) => None,
        })
        .collect();

    if typed_arguments.len() > 1 {
        return Err(syn::Error::new(
            typed_arguments[1].ty.span(),
            "mcp_query requires either a `TableQuery<Row>` first parameter or no parameters for a local row source",
        ));
    }

    if let Some(first_arg) = typed_arguments.first() {
        if let Some(row_type) = table_query_row_type(first_arg.ty.as_ref()) {
            let (response_type, error_type) = parse_result_return_type(&item_fn.sig.output)?;
            let result_row_type = table_query_result_type(&response_type).ok_or_else(|| {
                syn::Error::new(
                    response_type.span(),
                    "mcp_query custom backends must return `Result<TableQueryResult<Row>, E>`",
                )
            })?;
            let expected = type_signature_key(&row_type);
            let returned = type_signature_key(&result_row_type);
            if expected != returned {
                return Err(syn::Error::new(
                    response_type.span(),
                    format!(
                        "mcp_query custom backends require matching row types: query parameter is `{}` but return type is `TableQueryResult<{}`>",
                        expected, returned
                    ),
                ));
            }
            return Ok((HandlerKind::Query, row_type, error_type));
        }

        return Err(syn::Error::new(
            first_arg.ty.span(),
            "mcp_query requires a `TableQuery<Row>` first parameter for custom backends, or no parameters and `Result<Vec<Row>, E>` or `Vec<Row>` for local row sources",
        ));
    }

    let ReturnType::Type(_, return_type) = &item_fn.sig.output else {
        return Err(syn::Error::new(
            item_fn.sig.ident.span(),
            "mcp_query requires either a `TableQuery<Row>` first parameter or a zero-argument `Result<Vec<Row>, E>` or `Vec<Row>` return type",
        ));
    };

    if let Some(row_type) = vec_inner_type(return_type.as_ref()) {
        return Ok((
            HandlerKind::InfallibleRowSource,
            row_type,
            syn::parse_quote!(::std::string::String),
        ));
    }

    let (response_type, error_type) =
        parse_result_return_type(&item_fn.sig.output).map_err(|_| {
            syn::Error::new(
                return_type.span(),
                "mcp_query local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`",
            )
        })?;
    let row_type = vec_inner_type(&response_type).ok_or_else(|| {
        syn::Error::new(
            return_type.span(),
            "mcp_query local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`",
        )
    })?;

    Ok((HandlerKind::RowSource, row_type, error_type))
}

fn vec_inner_type(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, qself }) = ty else {
        return None;
    };
    if qself.is_some() {
        return None;
    }

    let segment = path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }

    single_type_argument(&segment.arguments)
}

fn table_query_result_type(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, qself }) = ty else {
        return None;
    };
    if qself.is_some() {
        return None;
    }

    let segment = path.segments.last()?;
    if segment.ident != "TableQueryResult" {
        return None;
    }

    single_type_argument(&segment.arguments)
}

fn table_query_row_type(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, qself }) = ty else {
        return None;
    };
    if qself.is_some() {
        return None;
    }

    let segment = path.segments.last()?;
    if segment.ident != "TableQuery" {
        return None;
    }

    single_type_argument(&segment.arguments)
}

fn type_signature_key(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn parse_result_type(path: &syn::Path) -> Option<(Type, Type)> {
    if !is_std_result_path(path) {
        return None;
    }

    let segment = path.segments.last()?;
    parse_result_type_arguments(&segment.arguments)
}

fn parse_result_type_arguments(arguments: &PathArguments) -> Option<(Type, Type)> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = arguments
    else {
        return None;
    };

    let mut type_arguments = args.iter();
    let GenericArgument::Type(ok_type) = type_arguments.next()? else {
        return None;
    };
    let GenericArgument::Type(error_type) = type_arguments.next()? else {
        return None;
    };
    if type_arguments.next().is_some() {
        return None;
    }

    Some((ok_type.clone(), error_type.clone()))
}

fn is_std_result_path(path: &syn::Path) -> bool {
    let segments: Vec<_> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();

    match segments.as_slice() {
        [result] if result == "Result" => true,
        [first, second, third] => {
            matches!(
                (first.as_str(), second.as_str(), third.as_str()),
                ("std", "result", "Result")
                    | ("core", "result", "Result")
                    | ("alloc", "result", "Result")
            )
        },
        _ => false,
    }
}

fn single_type_argument(arguments: &PathArguments) -> Option<Type> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = arguments
    else {
        return None;
    };

    let mut arguments = args.iter();
    let GenericArgument::Type(argument) = arguments.next()? else {
        return None;
    };
    if arguments.next().is_some() {
        return None;
    }

    Some(argument.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse2;

    fn expand_error(item: &str) -> String {
        let item: ItemFn = parse2(item.parse().expect("valid test function")).expect("parse item");
        expand_query(quote! {}, quote! { #item })
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn rejects_non_result_return_type() {
        let error = expand_error("fn rows() -> String { String::new() }");
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }

    #[test]
    fn rejects_result_with_wrong_number_of_type_arguments() {
        let error = expand_error("fn rows() -> Result<String> { Ok(String::new()) }");
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }

    #[test]
    fn accepts_explicit_std_result_paths() {
        let item: ItemFn = parse2(
            "fn rows() -> std::result::Result<Vec<String>, String> { Ok(vec![String::from(\"x\")]) }"
                .parse()
                .expect("valid test function"),
        )
        .expect("parse item");
        assert!(expand_query(quote! {}, quote! { #item }).is_ok());
    }

    #[test]
    fn requires_query_response_serialize_bound() {
        let item: ItemFn = parse2(
            "fn rows(_query: gpui_table::mcp::TableQuery<String>) -> std::result::Result<gpui_table::mcp::TableQueryResult<String>, String> { Ok(gpui_table::mcp::TableQueryResult { rows: vec![String::from(\"ann\")], total: 1, offset: 0, limit: None }) }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand_query(quote! {}, quote! { #item })
            .expect("mcp_query should accept serializable query responses");
        let expanded = expanded.to_string();

        assert!(expanded.contains("mcp_assert_table"));
        assert!(expanded.contains("mcp_assert_serialize"));
        assert!(expanded.contains("TableQueryResult"));
    }

    #[test]
    fn requires_row_source_response_serialize_bound() {
        let item: ItemFn = parse2(
            "fn rows() -> std::result::Result<Vec<String>, String> { Ok(vec![String::from(\"ann\")]) }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand_query(quote! {}, quote! { #item })
            .expect("mcp_query should accept serializable row sources");
        let expanded = expanded.to_string();

        assert!(expanded.contains("mcp_assert_table"));
        assert!(expanded.contains("mcp_assert_serialize"));
        assert!(expanded.contains("< String >"));
        assert!(expanded.contains("String"));
    }

    #[test]
    fn accepts_infallible_row_sources() {
        let item: ItemFn = parse2(
            "fn rows() -> Vec<String> { vec![String::from(\"ann\")] }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand_query(quote! {}, quote! { #item })
            .expect("mcp_query should accept infallible row sources");
        let expanded = expanded.to_string();

        assert!(expanded.contains("row_source"));
        assert!(expanded.contains("Ok"));
        assert!(expanded.contains("std :: string :: String"));
    }

    #[test]
    fn accepts_async_infallible_row_sources() {
        let item: ItemFn = parse2(
            "async fn rows() -> Vec<String> { vec![String::from(\"ann\")] }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand_query(quote! {}, quote! { #item })
            .expect("mcp_query should accept async infallible row sources");
        let expanded = expanded.to_string();

        assert!(expanded.contains("row_source_async"));
        assert!(expanded.contains("Ok"));
    }

    #[test]
    fn rejects_result_with_extra_type_arguments() {
        let error =
            expand_error("fn rows() -> Result<String, String, usize> { Ok(String::new()) }");
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }

    #[test]
    fn rejects_result_with_non_type_generic_arguments() {
        let error =
            expand_error("fn rows() -> Result<String, 'static> { Ok(String::from(\"x\")) }");
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }

    #[test]
    fn rejects_query_handler_with_non_table_query_result_return_type() {
        let error = expand_error(
            r#"fn rows(query: gpui_table::mcp::TableQuery<String>) -> Result<String, String> { Ok("x".to_string()) }"#,
        );
        assert!(error.contains("must return `Result<TableQueryResult<Row>, E>`"));
    }

    #[test]
    fn rejects_query_handler_with_mismatched_row_type() {
        let error = expand_error(
            r#"fn rows(query: gpui_table::mcp::TableQuery<String>) -> Result<gpui_table::mcp::TableQueryResult<usize>, String> { Ok(gpui_table::mcp::TableQueryResult { rows: vec![], total: 0, offset: 0, limit: None }) }"#,
        );
        assert!(error.contains("require matching row types"));
    }

    #[test]
    fn rejects_query_handler_with_too_many_parameters() {
        let error = expand_error(
            "fn rows(query: gpui_table::mcp::TableQuery<String>, ctx: usize) -> Result<gpui_table::mcp::TableQueryResult<String>, String> { todo!() }",
        );
        assert!(error.contains(
            "either a `TableQuery<Row>` first parameter or no parameters for a local row source"
        ));
    }

    #[test]
    fn rejects_row_source_with_argument() {
        let error =
            expand_error("fn rows(query: usize) -> Result<Vec<String>, String> { Ok(vec![]) }");
        assert!(error.contains("requires a `TableQuery<Row>` first parameter for custom backends, or no parameters and `Result<Vec<Row>, E>` or `Vec<Row>` for local row sources"));
    }

    #[test]
    fn rejects_local_row_source_with_non_vec_result() {
        let error = expand_error("fn rows() -> Result<String, String> { Ok(String::new()) }");
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }

    #[test]
    fn rejects_non_standard_result_path() {
        let error = expand_error(
            "fn rows() -> gpui_table::Result<Vec<String>, String> { Ok(vec![String::from(\"x\")]) }",
        );
        assert!(
            error.contains("local row sources must return `Result<Vec<Row>, E>` or `Vec<Row>`")
        );
    }
}
