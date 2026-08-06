//! Attribute macro implementation for `#[gpui_table_impl]`.
//!
//! This module handles trait impl blocks marked with `#[gpui_table_impl]` and
//! wires the implemented loader trait into the generated table delegate.
//!
//! # Usage
//!
//! ```ignore
//! use gpui_table::runtime::TableLoader;
//!
//! #[gpui_table_impl]
//! impl TableLoader for MyTableDelegate {
//!     const THRESHOLD: usize = 20;
//!
//!     fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
//!         // Load data...
//!     }
//! }
//! ```
//!
//! The trait must provide:
//! - `fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>)`
//! - `const THRESHOLD: usize` (optional, defaults to 10)

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, Path, parse2};

/// Main entry point for the `#[gpui_table_impl]` attribute macro.
pub fn gpui_table_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match gpui_table_impl_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn gpui_table_impl_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`#[gpui_table_impl]` does not accept arguments; apply it to an `impl TableLoader for <Row>TableDelegate` block",
        ));
    }

    let impl_block: ItemImpl = parse2(item)?;

    // Get the type being implemented (clone to avoid borrow issues)
    let self_ty = impl_block.self_ty.clone();

    let Some((ref trait_path, _)) = impl_block.trait_ else {
        return Err(syn::Error::new_spanned(
            &impl_block.self_ty,
            "`#[gpui_table_impl]` must be applied to an `impl TableLoader for <Row>TableDelegate` block",
        ));
    };

    let additional_impls = generate_loader_delegate_impl(&self_ty, trait_path);

    // Output the original impl block plus any additional implementations
    Ok(quote! {
        #impl_block
        #additional_impls
    })
}

/// Generate implementations that delegate to a user-provided trait.
fn generate_loader_delegate_impl(self_ty: &syn::Type, trait_path: &Path) -> TokenStream {
    quote! {
        impl gpui_table::__private::LoadMoreDelegate for #self_ty {
            fn has_more(&self, _: &gpui::App) -> bool {
                if self.loading {
                    return false;
                }
                !self.eof
            }

            fn load_more_threshold(&self) -> usize {
                <Self as #trait_path>::THRESHOLD
            }

            fn load_more(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<gpui_component::table::TableState<Self>>) {
                <Self as #trait_path>::load_more(self, window, cx);
            }
        }
    }
}
