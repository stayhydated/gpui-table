mod delegate;
mod expand;
mod filter_codegen;
mod filter_entities;
mod filter_matching;
#[cfg(feature = "mcp")]
mod mcp;
mod meta;

use darling::FromDeriveInput as _;
use proc_macro::TokenStream;
use syn::DeriveInput;

use self::meta::TableMeta;

pub(crate) fn derive_gpui_table(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match TableMeta::from_derive_input(&input) {
        Ok(meta) => match expand::expand_gpui_table(meta, &input) {
            Ok(ts) => ts.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}
