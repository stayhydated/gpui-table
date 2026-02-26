/// Per-item import tracking for generated prototyping output.
///
/// Each generated table story needs a set of `use` statements. [`ImportItem`]
/// describes a single imported path (optionally aliased), and [`ImportSet`]
/// collects them, deduplicates, and renders grouped `use` statements.
use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// The alias applied to an imported item.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Alias {
    /// Wildcard discard: `use X as _` (used for trait imports that only need
    /// to be in scope).
    Anonymous,
    /// Named rename: `use X as Foo`.
    Rename(&'static str),
}

/// A single item to be imported into a generated file.
///
/// Both `path` and the inner value of [`Alias::Rename`] are `&'static str`
/// because all built-in paths are string literals.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImportItem {
    /// Full path to the imported item, e.g. `"gpui_component::table::DataTable"`.
    pub path: &'static str,
    /// Optional alias applied to the import.
    pub alias: Option<Alias>,
}

impl ImportItem {
    /// An import with no alias.
    pub const fn path(path: &'static str) -> Self {
        Self { path, alias: None }
    }

    /// An import with an alias (e.g. [`Alias::Anonymous`] for trait imports).
    pub const fn aliased(path: &'static str, alias: Alias) -> Self {
        Self {
            path,
            alias: Some(alias),
        }
    }
}

/// A deduplicated, ordered collection of [`ImportItem`]s that can be rendered
/// into grouped `use` statements.
#[derive(Default)]
pub struct ImportSet(std::collections::BTreeSet<ImportItem>);

impl ImportSet {
    /// Insert a single item.
    pub fn add(&mut self, item: ImportItem) {
        self.0.insert(item);
    }

    /// Insert a slice of items (convenience for const arrays).
    pub fn extend_items(&mut self, items: &[ImportItem]) {
        self.0.extend(items.iter().cloned());
    }

    /// Insert an iterator of items.
    pub fn extend(&mut self, items: impl IntoIterator<Item = ImportItem>) {
        for item in items {
            self.0.insert(item);
        }
    }

    /// Render all imports as grouped `use parent::{a, b as c};` token streams.
    ///
    /// Items are grouped by parent path (everything before the last `::`),
    /// sorted, and emitted as a single `use` per group.
    pub fn to_token_stream(&self) -> TokenStream {
        // Group: parent_path → Vec<(name, alias)> — BTreeMap keeps groups sorted.
        let mut grouped: BTreeMap<String, Vec<(&'static str, Option<&Alias>)>> = BTreeMap::new();

        for item in &self.0 {
            let (parent, name) = item.path.rsplit_once("::").unwrap_or(("", item.path));
            grouped
                .entry(parent.to_string())
                .or_default()
                .push((name, item.alias.as_ref()));
        }

        let mut tokens = TokenStream::new();
        for (parent, items) in &grouped {
            if parent.is_empty() {
                // Bare name with no module prefix — skip (already in scope via glob).
                continue;
            }

            let parent_path: syn::Path = syn::parse_str(parent).expect("valid import parent path");

            let item_tokens: Vec<TokenStream> = items
                .iter()
                .map(|(name, alias)| {
                    let name_ident = format_ident!("{}", name);
                    match alias {
                        Some(Alias::Anonymous) => quote! { #name_ident as _ },
                        Some(Alias::Rename(a)) => {
                            let alias_ident = format_ident!("{}", a);
                            quote! { #name_ident as #alias_ident }
                        },
                        None => quote! { #name_ident },
                    }
                })
                .collect();

            if item_tokens.len() == 1 {
                let single = &item_tokens[0];
                tokens.extend(quote! { use #parent_path::#single; });
            } else {
                tokens.extend(quote! { use #parent_path::{#(#item_tokens),*}; });
            }
        }

        tokens
    }
}
