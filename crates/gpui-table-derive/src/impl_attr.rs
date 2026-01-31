//! Attribute macro implementation for `#[gpui_table_impl]`.
//!
//! This module handles the processing of impl blocks marked with `#[gpui_table_impl]`,
//! looking for method-level attributes like `#[load_more]` and `#[threshold]`
//! and generating the appropriate trait implementations.
//!
//! # Usage
//!
//! ## Freestanding approach (define methods/consts directly)
//!
//! ```ignore
//! #[gpui_table_impl]
//! impl MyTableDelegate {
//!     #[threshold]
//!     const LOAD_MORE_THRESHOLD: usize = 20;
//!
//!     #[load_more]
//!     pub fn load_more_items(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
//!         // Load data...
//!     }
//! }
//! ```
//!
//! The table struct must also set `#[gpui_table(load_more)]` to wire these hooks
//! into the generated delegate.
//!
//! If no `#[load_more]` method is provided, load_more becomes a no-op and
//! `has_more` returns false, which is useful for delegates that load all data up front.
//!
//! ## Trait-based approach (implement a loader trait)
//!
//! Apply `#[gpui_table_impl]` directly to a trait impl block. The macro will
//! automatically detect the trait and wire it up:
//!
//! ```ignore
//! use gpui_table::TableLoader;
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
use syn::{FnArg, ImplItem, ImplItemConst, ImplItemFn, ItemImpl, Path, ReturnType, parse2};

/// Information about a method marked with `#[load_more]`.
struct LoadMoreMethod {
    /// The method name identifier.
    method_name: syn::Ident,
}

const EXPECTED_LOAD_MORE_SIGNATURE: &str =
    "fn(&mut self, &mut Window, &mut Context<TableState<Self>>)";

/// Information about a const marked with `#[threshold]`.
struct ThresholdConst {
    /// The const name identifier.
    const_name: syn::Ident,
}

/// Validate that the load_more method has the correct signature.
fn validate_load_more_signature(method: &ImplItemFn) -> syn::Result<()> {
    let sig = &method.sig;

    // Check return type is unit (no return type or -> ())
    if !matches!(&sig.output, ReturnType::Default)
        && let ReturnType::Type(_, ty) = &sig.output
    {
        // Allow explicit () return type
        if let syn::Type::Tuple(tuple) = ty.as_ref() {
            if !tuple.elems.is_empty() {
                return Err(syn::Error::new_spanned(
                    &sig.output,
                    format!(
                        "#[load_more] method must have no return type.\nExpected: {}",
                        EXPECTED_LOAD_MORE_SIGNATURE
                    ),
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                &sig.output,
                format!(
                    "#[load_more] method must have no return type.\nExpected: {}",
                    EXPECTED_LOAD_MORE_SIGNATURE
                ),
            ));
        }
    }

    // Check we have exactly 3 arguments: self, window, cx
    if sig.inputs.len() != 3 {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            format!(
                "#[load_more] method must have exactly 3 parameters.\nExpected: {}",
                EXPECTED_LOAD_MORE_SIGNATURE
            ),
        ));
    }

    let mut args = sig.inputs.iter();

    // First arg must be &mut self
    if let Some(FnArg::Receiver(receiver)) = args.next() {
        if receiver.reference.is_none() || receiver.mutability.is_none() {
            return Err(syn::Error::new_spanned(
                receiver,
                format!(
                    "#[load_more] method must take `&mut self`.\nExpected: {}",
                    EXPECTED_LOAD_MORE_SIGNATURE
                ),
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            format!(
                "#[load_more] method must take `&mut self` as the first parameter.\nExpected: {}",
                EXPECTED_LOAD_MORE_SIGNATURE
            ),
        ));
    }

    // Second arg: window: &mut Window
    if let Some(FnArg::Typed(pat_type)) = args.next() {
        // Check it's a mutable reference
        if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref() {
            if type_ref.mutability.is_none() {
                return Err(syn::Error::new_spanned(
                    pat_type,
                    format!(
                        "#[load_more] second parameter must be `&mut Window`.\nExpected: {}",
                        EXPECTED_LOAD_MORE_SIGNATURE
                    ),
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                pat_type,
                format!(
                    "#[load_more] second parameter must be `&mut Window`.\nExpected: {}",
                    EXPECTED_LOAD_MORE_SIGNATURE
                ),
            ));
        }
    }

    // Third arg: cx: &mut Context<TableState<Self>>
    if let Some(FnArg::Typed(pat_type)) = args.next() {
        // Check it's a mutable reference
        if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref() {
            if type_ref.mutability.is_none() {
                return Err(syn::Error::new_spanned(
                    pat_type,
                    format!(
                        "#[load_more] third parameter must be `&mut Context<TableState<Self>>`.\nExpected: {}",
                        EXPECTED_LOAD_MORE_SIGNATURE
                    ),
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                pat_type,
                format!(
                    "#[load_more] third parameter must be `&mut Context<TableState<Self>>`.\nExpected: {}",
                    EXPECTED_LOAD_MORE_SIGNATURE
                ),
            ));
        }
    }

    Ok(())
}

/// Find methods marked with `#[load_more]` in the impl block.
fn find_load_more_method(impl_block: &mut ItemImpl) -> syn::Result<Option<LoadMoreMethod>> {
    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item {
            let mut found = false;
            method.attrs.retain(|attr| {
                if attr.path().is_ident("load_more") {
                    found = true;
                    false // Remove the attribute
                } else {
                    true
                }
            });

            if found {
                // Validate the signature
                validate_load_more_signature(method)?;

                return Ok(Some(LoadMoreMethod {
                    method_name: method.sig.ident.clone(),
                }));
            }
        }
    }
    Ok(None)
}

const VALID_THRESHOLD_TYPES: &[&str] = &["u8", "u16", "u32", "u64", "u128", "usize"];

/// Validate that the threshold const has a valid unsigned integer type.
fn validate_threshold_const(const_item: &ImplItemConst) -> syn::Result<()> {
    if let syn::Type::Path(type_path) = &const_item.ty
        && let Some(ident) = type_path.path.get_ident()
        && VALID_THRESHOLD_TYPES.contains(&ident.to_string().as_str())
    {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        &const_item.ty,
        format!(
            "#[threshold] const must be an unsigned integer type ({}).",
            VALID_THRESHOLD_TYPES.join(", ")
        ),
    ))
}

/// Find consts marked with `#[threshold]` in the impl block.
fn find_threshold_const(impl_block: &mut ItemImpl) -> syn::Result<Option<ThresholdConst>> {
    for item in &mut impl_block.items {
        if let ImplItem::Const(const_item) = item {
            let mut found = false;
            const_item.attrs.retain(|attr| {
                if attr.path().is_ident("threshold") {
                    found = true;
                    false // Remove the attribute
                } else {
                    true
                }
            });

            if found {
                // Validate the const type
                validate_threshold_const(const_item)?;

                return Ok(Some(ThresholdConst {
                    const_name: const_item.ident.clone(),
                }));
            }
        }
    }
    Ok(None)
}

/// Main entry point for the `#[gpui_table_impl]` attribute macro.
pub fn gpui_table_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match gpui_table_impl_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn gpui_table_impl_inner(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut impl_block: ItemImpl = parse2(item)?;

    // Get the type being implemented (clone to avoid borrow issues)
    let self_ty = impl_block.self_ty.clone();

    // Determine the loader trait:
    // 1. If attr is non-empty, use the provided trait path
    // 2. If the impl block is a trait impl (e.g., `impl TableLoader for Foo`), use that trait
    // 3. Otherwise, use freestanding approach with #[load_more] and #[threshold]
    let loader_trait: Option<Path> = if !attr.is_empty() {
        Some(parse2(attr)?)
    } else if let Some((_, ref trait_path, _)) = impl_block.trait_ {
        // This is a trait impl block like `impl TableLoader for ItemTableDelegate`
        Some(trait_path.clone())
    } else {
        None
    };

    // Generate additional trait implementations
    let additional_impls = if let Some(trait_path) = loader_trait {
        // Trait-based approach: delegate to the specified trait
        generate_trait_based_impl(&self_ty, &trait_path)
    } else {
        // Freestanding approach: look for #[load_more] and #[threshold] attributes
        generate_freestanding_impl(&mut impl_block, &self_ty)?
    };

    // Output the original impl block plus any additional implementations
    Ok(quote! {
        #impl_block
        #additional_impls
    })
}

/// Generate implementations that delegate to a user-provided trait.
fn generate_trait_based_impl(self_ty: &syn::Type, trait_path: &Path) -> TokenStream {
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

/// Generate implementations from freestanding #[load_more] and #[threshold] attributes.
fn generate_freestanding_impl(
    impl_block: &mut ItemImpl,
    self_ty: &syn::Type,
) -> syn::Result<TokenStream> {
    // Find marked items
    let load_more_method = find_load_more_method(impl_block)?;
    let threshold_const = find_threshold_const(impl_block)?;

    let threshold_impl = if let Some(threshold) = &threshold_const {
        let const_name = &threshold.const_name;
        quote! {
            fn load_more_threshold(&self) -> usize {
                Self::#const_name
            }
        }
    } else {
        TokenStream::new()
    };

    let (has_more_impl, load_more_impl) = if let Some(load_more) = load_more_method {
        let method_name = &load_more.method_name;
        (
            quote! {
                fn has_more(&self, _: &gpui::App) -> bool {
                    if self.loading {
                        return false;
                    }
                    !self.eof
                }
            },
            quote! {
                fn load_more(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<gpui_component::table::TableState<Self>>) {
                    Self::#method_name(self, window, cx);
                }
            },
        )
    } else {
        (
            quote! {
                fn has_more(&self, _: &gpui::App) -> bool {
                    false
                }
            },
            quote! {
                fn load_more(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<gpui_component::table::TableState<Self>>) {}
            },
        )
    };

    Ok(quote! {
        impl gpui_table::__private::LoadMoreDelegate for #self_ty {
            #has_more_impl
            #threshold_impl
            #load_more_impl
        }
    })
}
