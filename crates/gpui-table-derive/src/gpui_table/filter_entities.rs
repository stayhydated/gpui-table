use crate::__crate_paths::gpui::{App, Context, Entity, Window};
use crate::__crate_paths::gpui_component::table::TableState;
use crate::components::FilterComponents;
use crate::gpui_table::filter_codegen::{generate_filter_chain_methods, get_filter_type_tokens};
use crate::gpui_table::meta::FilterFieldMeta;

use darling::util::Override;
use heck::{ToPascalCase as _, ToTitleCase as _};
use quote::quote;
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
            let filter_type_tokens = get_filter_type_tokens(&f.filter_config, Some(&f.field_type));
            quote! {
                pub #field_ident: #Entity<#filter_type_tokens>,
            }
        })
        .collect();

    // Generate the build method that creates all filter entities
    let filter_builders: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let filter_type_tokens = get_filter_type_tokens(&f.filter_config, Some(&f.field_type));

            // Determine the title expression based on fluent config
            let title_expr =
                determine_filter_title_expr(&f.field_ident, fluent_config, struct_name);

            // Check if this is a FacetedFilter using the enum method
            if f.filter_config.is_faceted() {
                // Generate chain methods for options
                let chain_methods = generate_filter_chain_methods(&f.filter_config);

                // For FacetedFilter<T>, use new_for (type is already in the generic parameter)
                quote! {
                    let #field_ident = {
                        let on_filter_change = on_filter_change.clone();
                        let filter = #filter_type_tokens::new_for(
                            || #title_expr,
                            Default::default(),
                            move |_value, window, cx| {
                                // Notify callback for server-side filtering
                                if let Some(ref on_change) = on_filter_change {
                                    let on_change = on_change.clone();
                                    window.defer(cx, move |window, cx| {
                                        on_change(window, cx);
                                    });
                                }
                            },
                            cx,
                        );
                        #chain_methods
                        filter
                    };
                }
            } else {
                // Generate chain methods for options
                let chain_methods = generate_filter_chain_methods(&f.filter_config);

                // For other filters, use the reactive constructor for i18n updates.
                quote! {
                    let #field_ident = {
                        let on_filter_change = on_filter_change.clone();
                        let filter = #filter_type_tokens::new_for(
                            || #title_expr,
                            Default::default(),
                            move |_value, window, cx| {
                                // Notify callback for server-side filtering
                                if let Some(ref on_change) = on_filter_change {
                                    let on_change = on_change.clone();
                                    window.defer(cx, move |window, cx| {
                                        on_change(window, cx);
                                    });
                                }
                            },
                            cx,
                        );
                        #chain_methods
                        filter
                    };
                }
            }
        })
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
            quote! {
                self.#field_ident.update(cx, |filter, cx| {
                    filter.reset_silent(window, cx);
                });
            }
        })
        .collect();

    // Generate value getter methods for each filter
    let value_getters: Vec<proc_macro2::TokenStream> = filter_fields
        .iter()
        .map(|f| {
            let field_ident = &f.field_ident;
            let getter_name = Ident::new(&format!("{}_value", field_ident), field_ident.span());

            match &f.filter_config {
                FilterComponents::Text(_) => {
                    quote! {
                        /// Get the current value of the #field_ident text filter.
                        pub fn #getter_name(&self, cx: &#App) -> String {
                            self.#field_ident.read(cx).value().to_string()
                        }
                    }
                }
                FilterComponents::NumberRange(_) => {
                    quote! {
                        /// Get the current value of the #field_ident number range filter.
                        pub fn #getter_name(&self, cx: &#App) -> (Option<gpui_table::__deps::rust_decimal::Decimal>, Option<gpui_table::__deps::rust_decimal::Decimal>) {
                            self.#field_ident.read(cx).value()
                        }
                    }
                }
                FilterComponents::Faceted(_) => {
                    let field_type = &f.field_type;
                    quote! {
                        /// Get the current value of the #field_ident faceted filter.
                        pub fn #getter_name(&self, cx: &#App) -> std::collections::HashSet<#field_type> {
                            self.#field_ident.read(cx).value().clone()
                        }
                    }
                }
                FilterComponents::InfiniteFaceted(_) => {
                    let field_type = &f.field_type;
                    quote! {
                        /// Get the current value of the #field_ident infinite faceted filter.
                        pub fn #getter_name(&self, cx: &#App) -> Option<#field_type> {
                            self.#field_ident.read(cx).value()
                        }
                    }
                }
                FilterComponents::DateRange(_) => {
                    quote! {
                        /// Get the current value of the #field_ident date range filter.
                        pub fn #getter_name(&self, cx: &#App) -> (Option<gpui_table::__deps::chrono::NaiveDate>, Option<gpui_table::__deps::chrono::NaiveDate>) {
                            self.#field_ident.read(cx).value()
                        }
                    }
                }
            }
        })
        .collect();

    // Generate render helpers that group filters by type
    let (text_filters, number_filters, faceted_filters, date_filters) =
        categorize_filters(filter_fields);

    let text_filter_render = if !text_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = text_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all text filters as children (returns impl IntoElement).
            pub fn text_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let number_filter_render = if !number_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = number_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all number range filters as children (returns impl IntoElement).
            pub fn number_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let faceted_filter_render = if !faceted_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = faceted_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all faceted filters as children (returns impl IntoElement).
            pub fn faceted_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let date_filter_render = if !date_filters.is_empty() {
        let fields: Vec<proc_macro2::TokenStream> = date_filters
            .iter()
            .map(|f| {
                let ident = &f.field_ident;
                quote! { .child(self.#ident.clone()) }
            })
            .collect();
        quote! {
            /// Render all date range filters as children (returns impl IntoElement).
            pub fn date_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().items_center().gap_2()
                    #(#fields)*
            }
        }
    } else {
        quote! {}
    };

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
            let value_type = match &f.filter_config {
                FilterComponents::Text(_) => quote! { gpui_table::core::filter::TextValue },
                FilterComponents::NumberRange(_) => {
                    quote! { gpui_table::core::filter::RangeValue<gpui_table::__deps::rust_decimal::Decimal> }
                },
                FilterComponents::Faceted(_) => {
                    let ty = &f.field_type;
                    quote! { gpui_table::core::filter::FacetedValue<#ty> }
                },
                FilterComponents::InfiniteFaceted(_) => {
                    let ty = &f.field_type;
                    quote! { gpui_table::core::filter::SingleValue<#ty> }
                },
                FilterComponents::DateRange(_) => {
                    quote! { gpui_table::core::filter::RangeValue<gpui_table::__deps::chrono::NaiveDate> }
                },
            };
            quote! {
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
            match &f.filter_config {
                FilterComponents::Text(_) => quote! {
                    #field_ident: gpui_table::core::filter::TextValue::from(self.#getter_name(cx)),
                },
                FilterComponents::NumberRange(_) => quote! {
                    #field_ident: gpui_table::core::filter::RangeValue::from(self.#getter_name(cx)),
                },
                FilterComponents::Faceted(_) => quote! {
                    #field_ident: gpui_table::core::filter::FacetedValue::from(self.#getter_name(cx)),
                },
                FilterComponents::InfiniteFaceted(_) => quote! {
                    #field_ident: gpui_table::core::filter::SingleValue::from(self.#getter_name(cx)),
                },
                FilterComponents::DateRange(_) => quote! {
                    #field_ident: gpui_table::core::filter::RangeValue::from(self.#getter_name(cx)),
                },
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

    quote! {
        /// Entity handles for all filter UI components.
        /// Generated by the `#[derive(GpuiTable)]` macro.
        pub struct #filter_entities_name {
            #(#entity_field_defs)*
            __on_filter_change: Option<std::rc::Rc<dyn Fn(&mut #Window, &mut #App) + 'static>>,
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
            /// Build all filter entities for server-side filtering.
            ///
            /// # Arguments
            /// * `on_filter_change` - Optional callback invoked after any filter changes.
            ///   Use this for triggering data reload with new filter parameters.
            /// * `cx` - The application context
            pub fn build(
                on_filter_change: Option<std::rc::Rc<dyn Fn(&mut #Window, &mut #App) + 'static>>,
                cx: &mut #App,
            ) -> Self {
                use gpui_table::runtime::generated_filters::TableFilterComponent as _;

                #(#filter_builders)*

                Self {
                    #(#field_names,)*
                    __on_filter_change: on_filter_change,
                }
            }

            /// Build filters and wire them directly into a generated table delegate.
            ///
            /// On each filter change this updates `table.delegate_mut().set_filter_values(...)`
            /// and triggers a table refresh for client-side filtering.
            pub fn build_for_table(
                table: #Entity<#TableState<#delegate_name>>,
                cx: &mut #App,
            ) -> Self {
                let filters_slot: std::rc::Rc<std::cell::RefCell<Option<Self>>> =
                    std::rc::Rc::new(std::cell::RefCell::new(None));
                let filters_slot_for_change = filters_slot.clone();
                let table_for_change = table.clone();

                let on_filter_change: std::rc::Rc<dyn Fn(&mut #Window, &mut #App) + 'static> =
                    std::rc::Rc::new(move |_window, cx| {
                        let next_values = {
                            let filters = filters_slot_for_change.borrow();
                            filters.as_ref().map(|filters| {
                                gpui_table::runtime::FilterEntitiesExt::read_values(filters, cx)
                            })
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

                let initial_values =
                    gpui_table::runtime::FilterEntitiesExt::read_values(&filters, cx);
                table.update(cx, |table, cx| {
                    table.delegate_mut().set_filter_values(initial_values);
                    cx.notify();
                });

                filters
            }

            /// Build filters and wire them into a generated table delegate that uses
            /// `TableDataLoader` for server-side loading.
            ///
            /// On each filter change this method:
            /// 1. reads current filter values,
            /// 2. updates `table.delegate_mut().set_filter_values(...)`,
            /// 3. resets paging state (`rows.clear(); eof = false;`),
            /// 4. calls `table.delegate_mut().load_data(...)`.
            ///
            /// The same sequence is also run once during initialization.
            pub fn build_for_table_loader(
                table: #Entity<#TableState<#delegate_name>>,
                window: &mut #Window,
                cx: &mut #App,
            ) -> Self {
                let before_reload: std::rc::Rc<
                    dyn Fn(
                        &mut #delegate_name,
                        &mut #Window,
                        &mut #Context<#TableState<#delegate_name>>,
                    ) + 'static,
                > = std::rc::Rc::new(|delegate, _window, _cx| {
                    delegate.rows.clear();
                    delegate.eof = false;
                });

                Self::build_for_table_loader_with(table, Some(before_reload), window, cx)
            }

            /// Same as `build_for_table_loader(...)` but allows customizing pre-reload
            /// delegate state handling.
            ///
            /// The optional `before_reload` hook runs before every `load_data(...)` call,
            /// including the initial load.
            pub fn build_for_table_loader_with(
                table: #Entity<#TableState<#delegate_name>>,
                before_reload: Option<std::rc::Rc<
                    dyn Fn(
                        &mut #delegate_name,
                        &mut #Window,
                        &mut #Context<#TableState<#delegate_name>>,
                    ) + 'static,
                >>,
                window: &mut #Window,
                cx: &mut #App,
            ) -> Self {
                let filters_slot: std::rc::Rc<std::cell::RefCell<Option<Self>>> =
                    std::rc::Rc::new(std::cell::RefCell::new(None));
                let filters_slot_for_change = filters_slot.clone();
                let table_for_change = table.clone();
                let before_reload_for_change = before_reload.clone();

                let on_filter_change: std::rc::Rc<dyn Fn(&mut #Window, &mut #App) + 'static> =
                    std::rc::Rc::new(move |window, cx| {
                        let next_values = {
                            let filters = filters_slot_for_change.borrow();
                            filters.as_ref().map(|filters| {
                                gpui_table::runtime::FilterEntitiesExt::read_values(filters, cx)
                            })
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

                let initial_values =
                    gpui_table::runtime::FilterEntitiesExt::read_values(&filters, cx);
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
            pub fn reset_filters(&self, window: &mut #Window, cx: &mut #App) {
                #(#reset_fields)*

                if let Some(ref on_change) = self.__on_filter_change {
                    let on_change = on_change.clone();
                    window.defer(cx, move |window, cx| {
                        on_change(window, cx);
                    });
                }
            }

            /// Build a localized reset button bound to these filter entities.
            pub fn reset_button(&self) -> gpui_table::runtime::generated_filters::reset_filters::ResetFilters {
                let filters = self.clone();
                gpui_table::runtime::generated_filters::reset_filters::ResetFilters::new(move |window, cx| {
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

            #text_filter_render
            #number_filter_render
            #faceted_filter_render
            #date_filter_render

            // Value getters for server-side filtering
            #(#value_getters)*
        }

        impl gpui_table::runtime::FilterEntitiesExt for #filter_entities_name {
            type Values = #filter_values_name;

            fn read_values(&self, cx: &#App) -> Self::Values {
                #filter_values_name {
                    #(#read_values_fields)*
                }
            }

            fn all_filters(&self) -> impl gpui::IntoElement {
                use gpui::{ParentElement as _, Styled as _};
                gpui::div().flex().flex_wrap().items_center().gap_2()
                    #(#all_filter_fields)*
            }
        }

        /// Plain data struct holding all filter values.
        /// Generated by the `#[derive(GpuiTable)]` macro for client-side filtering.
        #[derive(Clone, Debug, Default)]
        pub struct #filter_values_name {
            #(#filter_values_fields)*
        }

        impl gpui_table::core::filter::FilterValuesExt for #filter_values_name {
            fn has_active_filters(&self) -> bool {
                #(#has_active_checks)||*
            }
        }

    }
}

/// Categorize filters by their type for grouped rendering.
fn categorize_filters(
    filter_fields: &[FilterFieldMeta],
) -> (
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
    Vec<&FilterFieldMeta>,
) {
    let mut text = Vec::new();
    let mut number = Vec::new();
    let mut faceted = Vec::new();
    let mut date = Vec::new();

    for f in filter_fields {
        match &f.filter_config {
            FilterComponents::Text(_) => text.push(f),
            FilterComponents::NumberRange(_) => number.push(f),
            FilterComponents::Faceted(_) | FilterComponents::InfiniteFaceted(_) => faceted.push(f),
            FilterComponents::DateRange(_) => date.push(f),
        }
    }

    (text, number, faceted, date)
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

        quote! { { use es_fluent::ToFluentString as _; #fluent_enum_ident::#fluent_variant_ident.to_fluent_string() } }
    } else {
        let raw_title = field_ident.to_string().to_title_case();
        quote! { #raw_title.to_string() }
    }
}
