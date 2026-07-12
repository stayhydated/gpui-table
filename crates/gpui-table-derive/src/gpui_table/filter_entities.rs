use crate::gpui_table::filter_codegen::get_filter_type_tokens;
use crate::gpui_table::meta::FilterFieldMeta;

use darling::util::Override;
use heck::{ToPascalCase as _, ToTitleCase as _};
use quote::{ToTokens, quote};
use syn::Ident;

/// Generate the FilterEntities struct that holds all filter Entity<T> fields
/// and provides builder methods for creating them.
pub(super) fn generate_filter_entities(
    struct_name: &Ident,
    filter_fields: &[FilterFieldMeta],
    fluent_config: &Option<Override<String>>,
) -> proc_macro2::TokenStream {
    if filter_fields.is_empty() {
        return quote! {};
    }

    let filter_entities_name = Ident::new(
        &format!("{}FilterEntities", struct_name),
        struct_name.span(),
    );
    let delegate_name = Ident::new(&format!("{}TableDelegate", struct_name), struct_name.span());

    // Generate Entity fields for each filter
    let entity_field_defs: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let filter_type_tokens = get_filter_type_tokens(&f.filter_config);
            let field_doc = format!(
                "Entity handle for the `{}` {} filter component.",
                field_ident,
                f.filter_config.kind_label()
            );
            quote! {
                #[doc = #field_doc]
                pub #field_ident: ::gpui::Entity<#filter_type_tokens>,
            }
        })
        .collect();

    // Generate the build method that creates all filter entities
    let filter_builders: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| generate_filter_builder_tokens(f, fluent_config, struct_name))
        .collect();

    // Field names for struct construction
    let field_names: Vec<&Ident> = filter_fields.iter().map(|f| &f.field_ident).collect();

    // Generate clone implementations for each entity
    let clone_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            quote! { #field_ident: self.#field_ident.clone(), }
        })
        .collect();

    // Generate reset calls for each filter entity.
    let reset_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let shape = f.filter_config.shape();
            quote! {
                <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::reset_silent(
                    &self.#field_ident,
                    window,
                    cx,
                );
            }
        })
        .collect();

    // Generate value getter methods for each filter
    let value_getters: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let getter_name = Ident::new(&format!("{}_value", field_ident), field_ident.span());
            let getter_doc = format!(
                "Get the current raw value of the `{}` {} filter. Use `read_values()` when you need the generated wrapper type for matching or query serialization.",
                field_ident,
                f.filter_config.kind_label()
            );
            let raw_value_type = f.filter_config.raw_value_type_tokens();
            let raw_value_expr = f.filter_config.read_raw_value_expr(field_ident);

            quote! {
                #[doc = #getter_doc]
                pub fn #getter_name(&self, cx: &::gpui::App) -> #raw_value_type {
                    #raw_value_expr
                }
            }
        })
        .collect();

    // Generate all_filters render method
    let all_filter_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let ident = &f.field_ident;
            quote! { .child(self.#ident.clone()) }
        })
        .collect();

    // Generate FilterValues struct for client-side filtering
    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());

    let filter_values_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let value_type = f.filter_config.generated_value_type_tokens();
            let field_doc = filter_value_field_doc(f);
            let query_doc = filter_value_query_doc(f);
            quote! {
                #[doc = #field_doc]
                #[doc = #query_doc]
                pub #field_ident: #value_type,
            }
        })
        .collect();

    // Generate read_values method that populates FilterValues from FilterEntities
    let read_values_fields: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let getter_name = Ident::new(&format!("{}_value", field_ident), field_ident.span());
            let raw_value_expr = quote! { self.#getter_name(cx) };
            let wrapped_value_expr = f.filter_config.wrap_raw_value_expr(raw_value_expr);
            quote! {
                #field_ident: #wrapped_value_expr,
            }
        })
        .collect();

    // Generate has_active_filters check expressions
    let has_active_checks: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            quote! { self.#field_ident.is_active() }
        })
        .collect();

    let build_doc = format!(
        "Build all filter entities without wiring them into a specific table state.\n\nUse this when you want to manage reloads manually. The optional `on_filter_change` callback runs after any generated filter changes, and `read_values()` snapshots the current state into `{filter_values_name}`."
    );
    let build_for_table_doc = format!(
        "Build filters and wire them directly into a generated table delegate for client-side filtering.\n\nOn each filter change this reads `{filter_values_name}`, calls `table.delegate_mut().set_filter_values(...)`, and notifies the table so generated `Matchable` logic re-runs against the in-memory rows."
    );
    let build_for_table_loader_doc = format!(
        "Build filters and wire them into a generated table delegate that uses `TableDataLoader` for server-side loading.\n\nOn each filter change this reads `{filter_values_name}`, stores it with `set_filter_values(...)`, resets paging state, and calls `load_data(...)`. Inside your loader implementation, inspect those generated wrapper fields and serialize them with `gpui_table_component::QueryFilterValue::to_query_string()` when the wrapped type supports it."
    );
    let build_for_table_loader_with_doc = format!(
        "Same as `build_for_table_loader(...)`, but lets callers customize delegate state just before each `load_data(...)` call.\n\nThe optional `before_reload` hook runs before every reload, including the initial one. This is useful when `{filter_values_name}` changes should also clear cached rows, paging cursors, or other delegate-owned request state."
    );
    let read_values_doc = format!(
        "Read all current filter state into `{filter_values_name}`.\n\nThe returned fields use the typed wrappers from `gpui_table::core::filter`, which are the same values stored on generated delegates via `set_filter_values(...)`. For server-side loaders, serialize individual fields with `gpui_table_component::QueryFilterValue::to_query_string()` when the wrapped type supports query serialization."
    );
    let filter_values_doc = format!(
        "Typed filter state produced by `{filter_entities_name}`.\n\nEach field uses a wrapper from `gpui_table::core::filter` (`TextValue`, `RangeValue<_>`, `FacetedValue<_>`, or `SingleValue<_>`), which is the shape consumed by generated client-side matching and stored on generated delegates for loader-based tables."
    );
    let has_active_doc = format!(
        "Returns `true` when any field in `{filter_values_name}` currently has an active filter value."
    );

    quote! {
        /// Entity handles for all filter UI components.
        /// Generated by the `#[derive(GpuiTable)]` macro.
        pub struct #filter_entities_name {
            #(#entity_field_defs)*
            __on_filter_change: Option<
                std::rc::Rc<dyn Fn(&mut ::gpui::Window, &mut ::gpui::App) + 'static>,
            >,
        }

        impl Clone for #filter_entities_name {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields)*
                    __on_filter_change: self.__on_filter_change.clone(),
                }
            }
        }

        impl #filter_entities_name {
            #[doc = #build_doc]
            pub fn build(
                on_filter_change: Option<
                    std::rc::Rc<dyn Fn(&mut ::gpui::Window, &mut ::gpui::App) + 'static>,
                >,
                cx: &mut ::gpui::App,
            ) -> Self {
                #(#filter_builders)*

                Self {
                    #(#field_names,)*
                    __on_filter_change: on_filter_change,
                }
            }

            #[doc = #build_for_table_doc]
            pub fn build_for_table(
                table: ::gpui::Entity<::gpui_component::table::TableState<#delegate_name>>,
                cx: &mut ::gpui::App,
            ) -> Self {
                let filters_slot: std::rc::Rc<std::cell::RefCell<Option<Self>>> =
                    std::rc::Rc::new(std::cell::RefCell::new(None));
                let filters_slot_for_change = filters_slot.clone();
                let table_for_change = table.clone();

                let on_filter_change: std::rc::Rc<dyn Fn(&mut ::gpui::Window, &mut ::gpui::App) + 'static> =
                    std::rc::Rc::new(move |_window, cx| {
                        let next_values = {
                            let filters = filters_slot_for_change.borrow();
                            filters.as_ref().map(|filters| filters.read_values(cx))
                        };

                        if let Some(values) = next_values {
                            table_for_change.update(cx, |table, cx| {
                                table.delegate_mut().set_filter_values(values);
                                cx.notify();
                            });
                        }
                    });

                let filters = Self::build(Some(on_filter_change), cx);
                *filters_slot.borrow_mut() = Some(filters.clone());

                let initial_values = filters.read_values(cx);
                table.update(cx, |table, cx| {
                    table.delegate_mut().set_filter_values(initial_values);
                    cx.notify();
                });

                filters
            }

            #[doc = #build_for_table_loader_doc]
            pub fn build_for_table_loader(
                table: ::gpui::Entity<::gpui_component::table::TableState<#delegate_name>>,
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::App,
            ) -> Self {
                let before_reload: std::rc::Rc<
                    dyn Fn(
                        &mut #delegate_name,
                        &mut ::gpui::Window,
                        &mut ::gpui::Context<
                            ::gpui_component::table::TableState<#delegate_name>,
                        >,
                    ) + 'static,
                > = std::rc::Rc::new(|delegate, _window, _cx| {
                    delegate.rows.clear();
                    delegate.eof = false;
                });

                Self::build_for_table_loader_with(table, Some(before_reload), window, cx)
            }

            #[doc = #build_for_table_loader_with_doc]
            pub fn build_for_table_loader_with(
                table: ::gpui::Entity<::gpui_component::table::TableState<#delegate_name>>,
                before_reload: Option<std::rc::Rc<
                    dyn Fn(
                        &mut #delegate_name,
                        &mut ::gpui::Window,
                        &mut ::gpui::Context<::gpui_component::table::TableState<#delegate_name>>,
                    ) + 'static,
                >>,
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::App,
            ) -> Self {
                let filters_slot: std::rc::Rc<std::cell::RefCell<Option<Self>>> =
                    std::rc::Rc::new(std::cell::RefCell::new(None));
                let filters_slot_for_change = filters_slot.clone();
                let table_for_change = table.clone();
                let before_reload_for_change = before_reload.clone();

                let on_filter_change: std::rc::Rc<dyn Fn(&mut ::gpui::Window, &mut ::gpui::App) + 'static> =
                    std::rc::Rc::new(move |window, cx| {
                        let next_values = {
                            let filters = filters_slot_for_change.borrow();
                            filters.as_ref().map(|filters| filters.read_values(cx))
                        };

                        if let Some(values) = next_values {
                            table_for_change.update(cx, |table, cx| {
                                let delegate = table.delegate_mut();
                                delegate.set_filter_values(values);

                                if let Some(ref before_reload) = before_reload_for_change {
                                    before_reload(delegate, window, cx);
                                }

                                use gpui_table::runtime::TableDataLoader as _;
                                delegate.load_data(window, cx);
                                cx.notify();
                            });
                        }
                    });

                let filters = Self::build(Some(on_filter_change), cx);
                *filters_slot.borrow_mut() = Some(filters.clone());

                let initial_values = filters.read_values(cx);
                table.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.set_filter_values(initial_values);

                    if let Some(ref before_reload) = before_reload {
                        before_reload(delegate, window, cx);
                    }

                    use gpui_table::runtime::TableDataLoader as _;
                    delegate.load_data(window, cx);
                    cx.notify();
                });

                filters
            }

            /// Reset all filters and invoke the filter-change callback once.
            pub fn reset_filters(&self, window: &mut ::gpui::Window, cx: &mut ::gpui::App) {
                #(#reset_fields)*

                if let Some(ref on_change) = self.__on_filter_change {
                    let on_change = on_change.clone();
                    window.defer(cx, move |window, cx| {
                        on_change(window, cx);
                    });
                }
            }

            /// Build a localized reset button bound to these filter entities.
            pub fn reset_button(&self) -> gpui_table_component::reset_filters::ResetFilters {
                let filters = self.clone();
                gpui_table_component::reset_filters::ResetFilters::new(move |window, cx| {
                    filters.reset_filters(window, cx);
                })
                .button_id(format!("{}-reset-filters", stringify!(#struct_name)))
            }

            /// Render all filters with a reset button appended.
            pub fn all_filters_with_reset(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().flex_wrap().items_center().gap_2()
                    #(#all_filter_fields)*
                    .child(self.reset_button())
            }

            // Value getters for server-side filtering
            #(#value_getters)*

            #[doc = #read_values_doc]
            pub fn read_values(&self, cx: &::gpui::App) -> #filter_values_name {
                #filter_values_name {
                    #(#read_values_fields)*
                }
            }

            /// Render all filters in a single row.
            pub fn all_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().flex_wrap().items_center().gap_2()
                    #(#all_filter_fields)*
            }
        }

        impl gpui_table::FilterEntitiesExt for #filter_entities_name {
            type Values = #filter_values_name;

            fn read_values(&self, cx: &::gpui::App) -> Self::Values {
                <#filter_entities_name>::read_values(self, cx)
            }

            fn all_filters(&self) -> impl gpui::IntoElement {
                <#filter_entities_name>::all_filters(self)
            }
        }

        #[doc = #filter_values_doc]
        #[derive(Clone, Debug, Default)]
        pub struct #filter_values_name {
            #(#filter_values_fields)*
        }

        impl gpui_table::core::filter::FilterValuesExt for #filter_values_name {
            #[doc = #has_active_doc]
            fn has_active_filters(&self) -> bool {
                #(#has_active_checks)||*
            }
        }

    }
}

fn generate_filter_builder_tokens(
    field: &FilterFieldMeta,
    fluent_config: &Option<Override<String>>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let field_ident = &field.field_ident;
    let title_expr = determine_filter_title_expr(field_ident, fluent_config, struct_name);
    let shape = field.filter_config.shape();
    let constructor = field.filter_config.constructor_expr();
    let constructor_tokens =
        filter_constructor_tokens(shape, constructor, quote! { |cx| #title_expr });

    quote! {
        let #field_ident = {
            let on_filter_change = on_filter_change.clone();
            #constructor_tokens
        };
    }
}

fn filter_constructor_tokens(
    shape: &syn::Path,
    constructor: Option<&syn::Expr>,
    title: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let value = quote! { Default::default() };
    let on_change = quote! {
        move |_value, window, cx| {
            if let Some(ref on_change) = on_filter_change {
                let on_change = on_change.clone();
                window.defer(cx, move |window, cx| {
                    on_change(window, cx);
                });
            }
        }
    };

    if let Some(constructor) = constructor {
        quote! {
            gpui_table::runtime::shape::build_filter_shape::<#shape, _>(
                #constructor,
                #title,
                #value,
                #on_change,
                cx,
            )
        }
    } else {
        quote! {
            <#shape as gpui_table::runtime::shape::GpuiTableFilterShape>::new_for(
                #title,
                #value,
                #on_change,
                cx,
            )
        }
    }
}

fn filter_value_field_doc(field: &FilterFieldMeta) -> String {
    let field_ident = &field.field_ident;
    let wrapper_ty = generated_filter_value_type_name(field);
    format!(
        "Current value for the `{field_ident}` {} filter, stored as `{wrapper_ty}`.",
        field.filter_config.kind_label()
    )
}

fn filter_value_query_doc(_field: &FilterFieldMeta) -> String {
    "This wrapper can be serialized with `gpui_table_component::QueryFilterValue` for loader-style query building.".to_string()
}

fn generated_filter_value_type_name(field: &FilterFieldMeta) -> String {
    compact_type_name(&field.filter_config.generated_value_type_tokens())
}

fn compact_type_name(tokens: &impl ToTokens) -> String {
    tokens
        .to_token_stream()
        .to_string()
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace(" >", ">")
        .replace(" , ", ", ")
        .replace(" ( ", "(")
        .replace(" )", ")")
}

/// Determine the title expression for a filter based on fluent config.
fn determine_filter_title_expr(
    field_ident: &Ident,
    fluent_config: &Option<Override<String>>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    if let Some(fluent) = fluent_config {
        let fluent_enum_ident = match fluent {
            Override::Explicit(key) => {
                let key_cap = key.to_pascal_case();
                Ident::new(
                    &format!("{}{}Variants", struct_name, key_cap),
                    struct_name.span(),
                )
            },
            Override::Inherit => {
                Ident::new(&format!("{}Variants", struct_name), struct_name.span())
            },
        };

        let field_name = field_ident.to_string().to_pascal_case();
        let fluent_variant_ident = Ident::new(&field_name, field_ident.span());

        quote! {
            gpui_table_component::i18n::localize_message(
                cx,
                &#fluent_enum_ident::#fluent_variant_ident
            )
        }
    } else {
        let raw_title = field_ident.to_string().to_title_case();
        quote! { #raw_title.to_string() }
    }
}
