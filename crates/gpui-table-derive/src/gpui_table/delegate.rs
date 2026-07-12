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
            fn load_more(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<gpui_component::table::TableState<Self>>) {
                gpui_table::__private::LoadMoreDelegate::load_more(self, window, cx);
            }
        };

        let has_more_impl = quote! {
            fn has_more(&self, app: &gpui::App) -> bool {
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
                fn load_data(
                    &mut self,
                    window: &mut gpui::Window,
                    cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
                ) {
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
            fn load_more(&mut self, _window: &mut gpui::Window, _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>) {}
        };

        let has_more_impl = quote! {
            fn has_more(&self, _: &gpui::App) -> bool {
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
                fn load_data(
                    &mut self,
                    _window: &mut gpui::Window,
                    _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
                ) {
                }
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
            fn loading(&self, app: &gpui::App) -> bool {
                self.#field(app)
            }
        }
    } else {
        quote! {
            fn loading(&self, _: &gpui::App) -> bool {
                self.full_loading
            }
        }
    };

    let columns_init_expr = quote! { <#struct_name as gpui_table::TableRowMeta>::table_columns() };
    let precompute_rows_len = quote! { let rows_len = rows.len(); };
    let filter_delegate_fields = if has_filters {
        quote! {
            filtered_row_indices: std::cell::RefCell<Vec<usize>>,
            row_scope: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(&#struct_name) -> bool + 'static>>>,
            active_filters: std::cell::RefCell<Option<#filter_values_name>>,
            filter_cache_rows_len: std::cell::Cell<usize>,
            filter_cache_dirty: std::cell::Cell<bool>,
        }
    } else {
        quote! {
            filtered_row_indices: std::cell::RefCell<Vec<usize>>,
            row_scope: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(&#struct_name) -> bool + 'static>>>,
            filter_cache_rows_len: std::cell::Cell<usize>,
            filter_cache_dirty: std::cell::Cell<bool>,
        }
    };
    let filter_delegate_init = if has_filters {
        quote! {
            filtered_row_indices: std::cell::RefCell::new((0..rows_len).collect()),
            row_scope: std::cell::RefCell::new(None),
            active_filters: std::cell::RefCell::new(None),
            filter_cache_rows_len: std::cell::Cell::new(rows_len),
            filter_cache_dirty: std::cell::Cell::new(false),
        }
    } else {
        quote! {
            filtered_row_indices: std::cell::RefCell::new((0..rows_len).collect()),
            row_scope: std::cell::RefCell::new(None),
            filter_cache_rows_len: std::cell::Cell::new(rows_len),
            filter_cache_dirty: std::cell::Cell::new(false),
        }
    };
    let filter_delegate_methods = if has_filters {
        quote! {
            fn ensure_filter_cache(&self) {
                use gpui_table::core::filter::{FilterValuesExt as _, Matchable as _};

                if !self.filter_cache_dirty.get() && self.filter_cache_rows_len.get() == self.rows.len() {
                    return;
                }

                let active_filters = self.active_filters.borrow().clone();
                let row_scope = self.row_scope.borrow();
                let mut indices = self.filtered_row_indices.borrow_mut();
                indices.clear();

                match active_filters {
                    Some(filters) if filters.has_active_filters() => {
                        indices.extend(self.rows.iter().enumerate().filter_map(|(row_ix, row)| {
                            let in_scope = row_scope.as_ref().map_or(true, |scope| scope(row));
                            (in_scope && row.matches_filters(&filters)).then_some(row_ix)
                        }));
                    }
                    _ => {
                        indices.extend(self.rows.iter().enumerate().filter_map(|(row_ix, row)| {
                            row_scope.as_ref().map_or(true, |scope| scope(row)).then_some(row_ix)
                        }));
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

            /// Replaces the generated filter values used to select visible rows.
            pub fn set_filter_values(&mut self, filters: #filter_values_name) {
                *self.active_filters.get_mut() = Some(filters);
                self.filter_cache_dirty.set(true);
            }

            /// Clears the active generated filter values.
            pub fn clear_filter_values(&mut self) {
                *self.active_filters.get_mut() = None;
                self.filter_cache_dirty.set(true);
            }

            /// Restricts visible rows to those accepted by `scope`.
            ///
            /// The scope is applied together with any active generated filter values.
            pub fn set_row_scope(&mut self, scope: impl Fn(&#struct_name) -> bool + 'static) {
                *self.row_scope.get_mut() = Some(std::rc::Rc::new(scope));
                self.filter_cache_dirty.set(true);
            }

            /// Clears the row scope so every row is eligible for display.
            pub fn clear_row_scope(&mut self) {
                *self.row_scope.get_mut() = None;
                self.filter_cache_dirty.set(true);
            }

            /// Returns whether a row scope is active.
            pub fn has_row_scope(&self) -> bool {
                self.row_scope.borrow().is_some()
            }

            /// Returns source-row indices for the rows currently visible through the delegate.
            pub fn visible_row_indices(&self) -> Vec<usize> {
                self.ensure_filter_cache();
                self.filtered_row_indices.borrow().clone()
            }

            /// Invalidates and immediately rebuilds the visible-row cache.
            ///
            /// Call this after mutating row values in place when filters or the row scope
            /// may produce different results without changing the row count.
            pub fn refresh_filtered_rows(&self) {
                self.filter_cache_dirty.set(true);
                self.ensure_filter_cache();
            }
        }
    } else {
        quote! {
            fn ensure_filter_cache(&self) {
                if !self.filter_cache_dirty.get() && self.filter_cache_rows_len.get() == self.rows.len() {
                    return;
                }

                let row_scope = self.row_scope.borrow();
                let mut indices = self.filtered_row_indices.borrow_mut();
                indices.clear();

                indices.extend(self.rows.iter().enumerate().filter_map(|(row_ix, row)| {
                    row_scope.as_ref().map_or(true, |scope| scope(row)).then_some(row_ix)
                }));

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

            /// Restricts visible rows to those accepted by `scope`.
            pub fn set_row_scope(&mut self, scope: impl Fn(&#struct_name) -> bool + 'static) {
                *self.row_scope.get_mut() = Some(std::rc::Rc::new(scope));
                self.filter_cache_dirty.set(true);
            }

            /// Clears the row scope so every row is eligible for display.
            pub fn clear_row_scope(&mut self) {
                *self.row_scope.get_mut() = None;
                self.filter_cache_dirty.set(true);
            }

            /// Returns whether a row scope is active.
            pub fn has_row_scope(&self) -> bool {
                self.row_scope.borrow().is_some()
            }

            /// Returns source-row indices for the rows currently visible through the delegate.
            pub fn visible_row_indices(&self) -> Vec<usize> {
                self.ensure_filter_cache();
                self.filtered_row_indices.borrow().clone()
            }

            /// Invalidates and immediately rebuilds the visible-row cache.
            ///
            /// Call this after mutating row values in place when the row scope may produce
            /// different results without changing the row count.
            pub fn refresh_filtered_rows(&self) {
                self.filter_cache_dirty.set(true);
                self.ensure_filter_cache();
            }
        }
    };
    let rows_count_impl = quote! {
        fn rows_count(&self, _: &gpui::App) -> usize {
            self.ensure_filter_cache();
            self.filtered_row_indices.borrow().len()
        }
    };
    let render_row_index_map = quote! {
        let row_ix = self.map_row_index(row_ix);
    };
    let context_menu_row_index_map = quote! {
        let row_ix = self.map_row_index(row_ix);
    };
    let sort_filter_refresh = quote! {
        self.filter_cache_dirty.set(true);
    };

    quote! {

        /// Table delegate generated for this row type.
        pub struct #delegate_name {
            pub rows: Vec<#struct_name>,
            columns: Vec<gpui_component::table::Column>,
            pub visible_rows: std::ops::Range<usize>,
            pub visible_cols: std::ops::Range<usize>,
            pub eof: bool,
            pub loading: bool,
            pub full_loading: bool,
            #filter_delegate_fields
        }

        impl #delegate_name {
            /// Creates a delegate backed by `rows`.
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

        impl gpui_component::table::TableDelegate for #delegate_name {
            fn columns_count(&self, _: &gpui::App) -> usize {
                self.columns.len()
            }

            #rows_count_impl

            fn column(&self, col_ix: usize, _: &gpui::App) -> gpui_component::table::Column {
                let mut column = self.columns
                    .get(col_ix)
                    .cloned()
                    .expect("Invalid column index");
                if let Some(fresh_column) = <#struct_name as gpui_table::TableRowMeta>::table_columns()
                    .into_iter()
                    .find(|fresh_column| fresh_column.key == column.key)
                {
                    column.name = fresh_column.name;
                }
                column
            }

            fn render_td(
                &mut self,
                row_ix: usize,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) -> impl gpui::IntoElement {
                use gpui_table::runtime::TableRowStyle;
                #render_row_index_map
                self.rows[row_ix].render_table_cell(#column_enum_name::from(col_ix), window, cx)
            }

            fn context_menu(
                &mut self,
                row_ix: usize,
                menu: gpui_component::menu::PopupMenu,
                window: &mut gpui::Window,
                cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) -> gpui_component::menu::PopupMenu {
                use gpui_table::runtime::TableRowContextMenu;
                #context_menu_row_index_map
                self.rows[row_ix].render_table_context_menu(row_ix, menu, window, cx)
            }

            fn visible_rows_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                self.visible_rows = visible_range;
            }

            fn visible_columns_changed(
                &mut self,
                visible_range: std::ops::Range<usize>,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
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
                sort: gpui_component::table::ColumnSort,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
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
