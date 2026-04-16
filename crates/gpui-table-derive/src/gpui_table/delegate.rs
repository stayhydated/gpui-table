use crate::__crate_paths::gpui::{App, Context, IntoElement, Window};
use crate::__crate_paths::gpui_component::menu::PopupMenu;
use crate::__crate_paths::gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use crate::gpui_table::meta::FilterFieldMeta;

use quote::quote;
use syn::Ident;

pub(super) fn generate_delegate(
    struct_name: &Ident,
    column_enum_name: &Ident,
    sort_arms: Vec<proc_macro2::TokenStream>,
    loading: Option<Ident>,
    load_more: bool,
    filter_fields: &[FilterFieldMeta],
) -> proc_macro2::TokenStream {
    let delegate_name = Ident::new(&format!("{}TableDelegate", struct_name), struct_name.span());
    let has_filters = !filter_fields.is_empty();
    let filter_values_name =
        Ident::new(&format!("{}FilterValues", struct_name), struct_name.span());

    let (load_more_impl, has_more_impl, threshold_impl, data_loader_impl) = if load_more {
        let load_more_impl = quote! {
            fn load_more(&mut self, window: &mut #Window, cx: &mut #Context<#TableState<Self>>) {
                gpui_table::__private::LoadMoreDelegate::load_more(self, window, cx);
            }
        };

        let has_more_impl = quote! {
            fn has_more(&self, app: &#App) -> bool {
                gpui_table::__private::LoadMoreDelegate::has_more(self, app)
            }
        };

        let threshold_impl = quote! {
            fn load_more_threshold(&self) -> usize {
                gpui_table::__private::LoadMoreDelegate::load_more_threshold(self)
            }
        };

        let data_loader_impl = quote! {
            impl gpui_table::runtime::TableDataLoader for #delegate_name {
                fn load_data(&mut self, window: &mut #Window, cx: &mut #Context<#TableState<Self>>) {
                    gpui_table::__private::LoadMoreDelegate::load_more(self, window, cx);
                }
            }
        };

        (
            load_more_impl,
            has_more_impl,
            threshold_impl,
            data_loader_impl,
        )
    } else {
        let load_more_impl = quote! {
            fn load_more(&mut self, _window: &mut #Window, _cx: &mut #Context<#TableState<Self>>) {}
        };

        let has_more_impl = quote! {
            fn has_more(&self, _: &#App) -> bool {
                false
            }
        };

        let threshold_impl = quote! {
            fn load_more_threshold(&self) -> usize {
                10
            }
        };

        let data_loader_impl = quote! {
            impl gpui_table::runtime::TableDataLoader for #delegate_name {
                fn load_data(&mut self, _window: &mut #Window, _cx: &mut #Context<#TableState<Self>>) {}
            }
        };

        (
            load_more_impl,
            has_more_impl,
            threshold_impl,
            data_loader_impl,
        )
    };

    let loading_impl = if let Some(field) = loading {
        quote! {
            fn loading(&self, app: &#App) -> bool {
                self.#field(app)
            }
        }
    } else {
        quote! {
            fn loading(&self, _: &#App) -> bool {
                self.full_loading
            }
        }
    };

    let columns_init_expr =
        quote! { <#struct_name as gpui_table::runtime::TableRowMeta>::table_columns() };
    let precompute_rows_len = if has_filters {
        quote! { let rows_len = rows.len(); }
    } else {
        quote! {}
    };
    let filter_delegate_fields = if has_filters {
        quote! {
            filtered_row_indices: std::cell::RefCell<Vec<usize>>,
            active_filters: std::cell::RefCell<Option<#filter_values_name>>,
            filter_cache_rows_len: std::cell::Cell<usize>,
            filter_cache_dirty: std::cell::Cell<bool>,
        }
    } else {
        quote! {}
    };
    let filter_delegate_init = if has_filters {
        quote! {
            filtered_row_indices: std::cell::RefCell::new((0..rows_len).collect()),
            active_filters: std::cell::RefCell::new(None),
            filter_cache_rows_len: std::cell::Cell::new(rows_len),
            filter_cache_dirty: std::cell::Cell::new(false),
        }
    } else {
        quote! {}
    };
    let filter_delegate_methods = if has_filters {
        quote! {
            fn ensure_filter_cache(&self) {
                use gpui_table::core::filter::{FilterValuesExt as _, Matchable as _};

                if !self.filter_cache_dirty.get() && self.filter_cache_rows_len.get() == self.rows.len() {
                    return;
                }

                let active_filters = self.active_filters.borrow().clone();
                let mut indices = self.filtered_row_indices.borrow_mut();
                indices.clear();

                match active_filters {
                    Some(filters) if filters.has_active_filters() => {
                        indices.extend(self.rows.iter().enumerate().filter_map(|(row_ix, row)| {
                            row.matches_filters(&filters).then_some(row_ix)
                        }));
                    }
                    _ => {
                        indices.extend(0..self.rows.len());
                    }
                }

                self.filter_cache_rows_len.set(self.rows.len());
                self.filter_cache_dirty.set(false);
            }

            fn map_row_index(&self, row_ix: usize) -> usize {
                self.ensure_filter_cache();
                self.filtered_row_indices
                    .borrow()
                    .get(row_ix)
                    .copied()
                    .expect("invalid filtered row index")
            }

            pub fn set_filter_values(&mut self, filters: #filter_values_name) {
                *self.active_filters.get_mut() = Some(filters);
                self.filter_cache_dirty.set(true);
            }

            pub fn clear_filter_values(&mut self) {
                *self.active_filters.get_mut() = None;
                self.filter_cache_dirty.set(true);
            }

            pub fn refresh_filtered_rows(&self) {
                self.filter_cache_dirty.set(true);
                self.ensure_filter_cache();
            }
        }
    } else {
        quote! {}
    };
    let rows_count_impl = if has_filters {
        quote! {
            fn rows_count(&self, _: &#App) -> usize {
                self.ensure_filter_cache();
                self.filtered_row_indices.borrow().len()
            }
        }
    } else {
        quote! {
            fn rows_count(&self, _: &#App) -> usize {
                self.rows.len()
            }
        }
    };
    let render_row_index_map = if has_filters {
        quote! {
            let row_ix = self.map_row_index(row_ix);
        }
    } else {
        quote! {}
    };
    let context_menu_row_index_map = if has_filters {
        quote! {
            let row_ix = self.map_row_index(row_ix);
        }
    } else {
        quote! {}
    };
    let sort_filter_refresh = if has_filters {
        quote! {
            self.filter_cache_dirty.set(true);
        }
    } else {
        quote! {}
    };

    quote! {

        pub struct #delegate_name {
            pub rows: Vec<#struct_name>,
            columns: Vec<#Column>,
            pub visible_rows: std::ops::Range<usize>,
            pub visible_cols: std::ops::Range<usize>,
            pub eof: bool,
            pub loading: bool,
            pub full_loading: bool,
            #filter_delegate_fields
        }

        impl #delegate_name {
            pub fn new(rows: Vec<#struct_name>) -> Self {
                #precompute_rows_len
                Self {
                    rows,
                    columns: #columns_init_expr,
                    visible_rows: Default::default(),
                    visible_cols: Default::default(),
                    eof: false,
                    loading: false,
                    full_loading: false,
                    #filter_delegate_init
                }
            }

            #filter_delegate_methods
        }

        impl #TableDelegate for #delegate_name {
            fn columns_count(&self, _: &#App) -> usize {
                self.columns.len()
            }

            #rows_count_impl

            fn column(&self, col_ix: usize, _: &#App) -> #Column {
                self.columns
                    .get(col_ix)
                    .cloned()
                    .expect("Invalid column index")
            }

            fn render_td(
                &mut self,
                row_ix: usize,
                col_ix: usize,
                window: &mut #Window,
                cx: &mut #Context<#TableState<Self>>,
            ) -> impl #IntoElement {
                use gpui_table::runtime::TableRowStyle;
                #render_row_index_map
                self.rows[row_ix].render_table_cell(#column_enum_name::from(col_ix), window, cx)
            }

            fn context_menu(
                &mut self,
                row_ix: usize,
                menu: #PopupMenu,
                window: &mut #Window,
                cx: &mut #Context<#TableState<Self>>,
            ) -> #PopupMenu {
                use gpui_table::runtime::TableRowContextMenu;
                #context_menu_row_index_map
                self.rows[row_ix].render_table_context_menu(row_ix, menu, window, cx)
            }

            fn visible_rows_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                self.visible_rows = visible_range;
            }

            fn visible_columns_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                self.visible_cols = visible_range;
            }

            #loading_impl
            #has_more_impl
            #load_more_impl
            #threshold_impl

            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: #ColumnSort,
                _: &mut #Window,
                _: &mut #Context<#TableState<Self>>,
            ) {
                match col_ix {
                    #(#sort_arms)*
                    _ => {}
                }

                #sort_filter_refresh
            }
        }

        #data_loader_impl
    }
}
