#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod i18n {
    #[folder = "/home/mark/Documents/GitHub/gpui-table/examples/some-lib/../i18n"]
    struct SomeLibI18nAssets;
    impl SomeLibI18nAssets {
        fn matcher() -> rust_embed::utils::PathMatcher {
            const INCLUDES: &[&str] = &[];
            const EXCLUDES: &[&str] = &[];
            static PATH_MATCHER: ::std::sync::OnceLock<rust_embed::utils::PathMatcher> = ::std::sync::OnceLock::new();
            PATH_MATCHER
                .get_or_init(|| rust_embed::utils::PathMatcher::new(INCLUDES, EXCLUDES))
                .clone()
        }
        /// Get an embedded file and its metadata.
        pub fn get(file_path: &str) -> ::std::option::Option<rust_embed::EmbeddedFile> {
            let rel_file_path = file_path.replace("\\", "/");
            let file_path = ::std::path::Path::new(
                    "/home/mark/Documents/GitHub/gpui-table/examples/some-lib/../i18n",
                )
                .join(&rel_file_path);
            let canonical_file_path = file_path.canonicalize().ok()?;
            if !canonical_file_path
                .starts_with("/home/mark/Documents/GitHub/gpui-table/examples/i18n")
            {
                let metadata = ::std::fs::symlink_metadata(&file_path).ok()?;
                if !metadata.is_symlink() {
                    return ::std::option::Option::None;
                }
            }
            let path_matcher = Self::matcher();
            if path_matcher.is_path_included(&rel_file_path) {
                rust_embed::utils::read_file_from_fs(&canonical_file_path).ok()
            } else {
                ::std::option::Option::None
            }
        }
        /// Iterates over the file paths in the folder.
        pub fn iter() -> impl ::std::iter::Iterator<
            Item = ::std::borrow::Cow<'static, str>,
        > {
            use ::std::path::Path;
            rust_embed::utils::get_files(
                    ::std::string::String::from(
                        "/home/mark/Documents/GitHub/gpui-table/examples/some-lib/../i18n",
                    ),
                    Self::matcher(),
                )
                .map(|e| ::std::borrow::Cow::from(e.rel_path))
        }
    }
    impl rust_embed::RustEmbed for SomeLibI18nAssets {
        fn get(file_path: &str) -> ::std::option::Option<rust_embed::EmbeddedFile> {
            SomeLibI18nAssets::get(file_path)
        }
        fn iter() -> rust_embed::Filenames {
            rust_embed::Filenames::Dynamic(
                ::std::boxed::Box::new(SomeLibI18nAssets::iter()),
            )
        }
    }
    impl es_fluent::__manager_core::EmbeddedAssets for SomeLibI18nAssets {
        fn domain() -> &'static str {
            "some-lib"
        }
    }
    static SOME_LIB_I18N_MODULE_DATA: es_fluent::__manager_core::EmbeddedModuleData = es_fluent::__manager_core::EmbeddedModuleData {
        name: "some-lib",
        domain: "some-lib",
        supported_languages: &[
            {
                #[allow(dead_code)]
                enum ProcMacroHack {
                    Value = ("\"en\"", 0).1,
                }
                unsafe {
                    ::unic_langid_macros::LanguageIdentifier::from_raw_parts_unchecked(
                        unsafe {
                            ::unic_langid_macros::subtags::Language::from_raw_unchecked(
                                28261u64,
                            )
                        },
                        None,
                        None,
                        None,
                    )
                }
            },
        ],
    };
    #[allow(non_upper_case_globals)]
    const _: () = {
        static __INVENTORY: ::inventory::Node = ::inventory::Node {
            value: &{
                &es_fluent::__manager_core::EmbeddedI18nModule::<
                    SomeLibI18nAssets,
                >::new(&SOME_LIB_I18N_MODULE_DATA)
                    as &dyn es_fluent::__manager_core::I18nModule
            },
            next: ::inventory::core::cell::UnsafeCell::new(
                ::inventory::core::option::Option::None,
            ),
        };
        #[link_section = ".text.startup"]
        unsafe extern "C" fn __ctor() {
            unsafe { ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY) }
        }
        #[used]
        #[link_section = ".init_array"]
        static __CTOR: unsafe extern "C" fn() = __ctor;
    };
}
pub mod structs {
    pub mod infinite_scroll {
        use fake::faker::lorem::en::Sentence;
        use fake::faker::name::en::Name;
        use fake::{Dummy, Fake, Faker};
        use gpui::{AsyncWindowContext, Context, Window};
        use gpui_component::table::TableState;
        use gpui_table::NamedTableRow;
        use std::time::Duration;
        #[table(load_more = "Self::load_more_data")]
        pub struct InfiniteRow {
            #[dummy(faker = "1..10000")]
            #[table(width = 80.)]
            pub id: u64,
            #[dummy(faker = "Name()")]
            #[table(sortable)]
            pub name: String,
            #[dummy(faker = "Sentence(3..6)")]
            #[table(width = 300.)]
            pub description: String,
        }
        impl ::fake::Dummy<::fake::Faker> for InfiniteRow {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                rng: &mut R,
            ) -> Self {
                let id: u64 = ::fake::Fake::fake_with_rng::<u64, _>(&(1..10000), rng);
                let name: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(Name()), rng);
                let description: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(Sentence(3..6)), rng);
                InfiniteRow {
                    id,
                    name,
                    description,
                }
            }
        }
        pub enum InfiniteRowTableColumn {
            Id,
            Name,
            Description,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InfiniteRowTableColumn {
            #[inline]
            fn clone(&self) -> InfiniteRowTableColumn {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for InfiniteRowTableColumn {}
        #[automatically_derived]
        impl ::core::fmt::Debug for InfiniteRowTableColumn {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        InfiniteRowTableColumn::Id => "Id",
                        InfiniteRowTableColumn::Name => "Name",
                        InfiniteRowTableColumn::Description => "Description",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for InfiniteRowTableColumn {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for InfiniteRowTableColumn {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for InfiniteRowTableColumn {
            #[inline]
            fn eq(&self, other: &InfiniteRowTableColumn) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl From<usize> for InfiniteRowTableColumn {
            fn from(ix: usize) -> Self {
                match ix {
                    0usize => InfiniteRowTableColumn::Id,
                    1usize => InfiniteRowTableColumn::Name,
                    2usize => InfiniteRowTableColumn::Description,
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!("Invalid column index: {0}", ix),
                        );
                    }
                }
            }
        }
        impl From<InfiniteRowTableColumn> for usize {
            fn from(col: InfiniteRowTableColumn) -> Self {
                match col {
                    InfiniteRowTableColumn::Id => 0usize,
                    InfiniteRowTableColumn::Name => 1usize,
                    InfiniteRowTableColumn::Description => 2usize,
                }
            }
        }
        impl gpui_table::TableRowMeta for InfiniteRow {
            const TABLE_ID: &'static str = "InfiniteRow";
            const TABLE_TITLE: &'static str = "InfiniteRow";
            fn table_title() -> String {
                Self::TABLE_TITLE.to_string()
            }
            fn table_columns() -> &'static [gpui_component::table::Column] {
                static COLUMNS: std::sync::OnceLock<
                    Vec<gpui_component::table::Column>,
                > = std::sync::OnceLock::new();
                COLUMNS
                    .get_or_init(|| <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            gpui_component::table::Column::new("id", "Id").width(80f32),
                            gpui_component::table::Column::new("name", "Name")
                                .width(100f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "description",
                                    "Description",
                                )
                                .width(300f32),
                        ]),
                    ))
            }
            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    0usize => Box::new(self.id.clone()),
                    1usize => Box::new(self.name.clone()),
                    2usize => Box::new(self.description.clone()),
                    _ => Box::new(String::new()),
                }
            }
        }
        impl gpui_table::TableRowStyle for InfiniteRow {
            type ColumnId = InfiniteRowTableColumn;
            fn render_table_cell(
                &self,
                col: Self::ColumnId,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::AnyElement {
                use gpui::IntoElement;
                gpui_table::default_render_cell(self, col.into(), window, cx)
                    .into_any_element()
            }
        }
        pub struct InfiniteRowTableDelegate {
            pub rows: Vec<InfiniteRow>,
            #[new(default)]
            pub visible_rows: std::ops::Range<usize>,
            #[new(default)]
            pub visible_cols: std::ops::Range<usize>,
            #[new(default)]
            pub eof: bool,
            #[new(default)]
            pub loading: bool,
            #[new(default)]
            pub full_loading: bool,
        }
        impl InfiniteRowTableDelegate {
            ///Constructs a new `InfiniteRowTableDelegate`.
            pub fn new(rows: Vec<InfiniteRow>) -> Self {
                InfiniteRowTableDelegate {
                    rows: rows,
                    visible_rows: ::core::default::Default::default(),
                    visible_cols: ::core::default::Default::default(),
                    eof: ::core::default::Default::default(),
                    loading: ::core::default::Default::default(),
                    full_loading: ::core::default::Default::default(),
                }
            }
        }
        impl gpui_component::table::TableDelegate for InfiniteRowTableDelegate {
            fn columns_count(&self, _: &gpui::App) -> usize {
                <InfiniteRow as gpui_table::TableRowMeta>::table_columns().len()
            }
            fn rows_count(&self, _: &gpui::App) -> usize {
                self.rows.len()
            }
            fn column(
                &self,
                col_ix: usize,
                _: &gpui::App,
            ) -> &gpui_component::table::Column {
                &<InfiniteRow as gpui_table::TableRowMeta>::table_columns()[col_ix]
            }
            fn render_td(
                &self,
                row_ix: usize,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> impl gpui::IntoElement {
                use gpui_table::TableRowStyle;
                self.rows[row_ix]
                    .render_table_cell(InfiniteRowTableColumn::from(col_ix), window, cx)
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
            fn is_eof(&self, _: &gpui::App) -> bool {
                self.eof
            }
            fn loading(&self, _: &gpui::App) -> bool {
                self.loading
            }
            fn load_more(
                &mut self,
                window: &mut gpui::Window,
                cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                Self::load_more_data(self, window, cx);
            }
            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: gpui_component::table::ColumnSort,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                match col_ix {
                    1usize => {
                        self.rows
                            .sort_by(|a, b| {
                                let a_val = &a.name;
                                let b_val = &b.name;
                                match sort {
                                    gpui_component::table::ColumnSort::Ascending => {
                                        a_val
                                            .partial_cmp(b_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    gpui_component::table::ColumnSort::Descending => {
                                        b_val
                                            .partial_cmp(a_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            });
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InfiniteRow {
            #[inline]
            fn clone(&self) -> InfiniteRow {
                InfiniteRow {
                    id: ::core::clone::Clone::clone(&self.id),
                    name: ::core::clone::Clone::clone(&self.name),
                    description: ::core::clone::Clone::clone(&self.description),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InfiniteRow {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "InfiniteRow",
                    "id",
                    &self.id,
                    "name",
                    &self.name,
                    "description",
                    &&self.description,
                )
            }
        }
        impl InfiniteRowTableDelegate {
            pub fn load_more_data(
                &mut self,
                _window: &mut Window,
                cx: &mut Context<TableState<Self>>,
            ) {
                if self.loading || self.eof {
                    return;
                }
                self.loading = true;
                cx.notify();
                cx.spawn(async move |view, cx| {
                        cx.background_executor().timer(Duration::from_millis(500)).await;
                        let new_rows: Vec<InfiniteRow> = (0..20)
                            .map(|_| Faker.fake())
                            .collect();
                        _ = cx
                            .update(|cx| {
                                view.update(
                                        cx,
                                        |table, cx| {
                                            let delegate = table.delegate_mut();
                                            delegate.rows.extend(new_rows);
                                            delegate.loading = false;
                                            if delegate.rows.len() >= 500 {
                                                delegate.eof = true;
                                            }
                                            cx.notify();
                                        },
                                    )
                                    .unwrap();
                            });
                    })
                    .detach();
            }
        }
    }
    pub mod item {
        use es_fluent::EsFluentKv;
        use fake::faker::{chrono::en::DateTime, color::en::Color, lorem::en::Word};
        use fake::uuid::UUIDv4;
        use gpui_table::NamedTableRow;
        #[fluent_kv(this)]
        #[table(fluent, custom_style)]
        pub struct Item {
            #[table(skip)]
            #[dummy(faker = "UUIDv4")]
            id: uuid::Uuid,
            #[table(width = 100.)]
            #[dummy(faker = "Word()")]
            name: String,
            #[table(width = 80.)]
            #[dummy(faker = "Color()")]
            color: String,
            #[table(width = 60.)]
            #[dummy(faker = "18..67")]
            weight: u8,
            #[table(width = 50.)]
            #[dummy(faker = "DateTime()")]
            acquired_on: chrono::DateTime<chrono::Utc>,
        }
        impl ::fake::Dummy<::fake::Faker> for Item {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                rng: &mut R,
            ) -> Self {
                let id: uuid::Uuid = ::fake::Fake::fake_with_rng::<
                    uuid::Uuid,
                    _,
                >(&(UUIDv4), rng);
                let name: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(Word()), rng);
                let color: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(Color()), rng);
                let weight: u8 = ::fake::Fake::fake_with_rng::<u8, _>(&(18..67), rng);
                let acquired_on: chrono::DateTime<chrono::Utc> = ::fake::Fake::fake_with_rng::<
                    chrono::DateTime<chrono::Utc>,
                    _,
                >(&(DateTime()), rng);
                Item {
                    id,
                    name,
                    color,
                    weight,
                    acquired_on,
                }
            }
        }
        pub enum ItemKvFtl {
            Id,
            Name,
            Color,
            Weight,
            AcquiredOn,
        }
        impl ::std::fmt::Display for ItemKvFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_kv_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_kv_ftl-name", None),
                            ),
                        )
                    }
                    Self::Color => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_kv_ftl-color", None),
                            ),
                        )
                    }
                    Self::Weight => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_kv_ftl-weight", None),
                            ),
                        )
                    }
                    Self::AcquiredOn => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_kv_ftl-acquired_on", None),
                            ),
                        )
                    }
                }
            }
        }
        impl From<&ItemKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &ItemKvFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<ItemKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: ItemKvFtl) -> Self {
                (&value).into()
            }
        }
        impl Item {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("item", None)
            }
        }
        pub enum ItemTableColumn {
            Name,
            Color,
            Weight,
            AcquiredOn,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ItemTableColumn {
            #[inline]
            fn clone(&self) -> ItemTableColumn {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for ItemTableColumn {}
        #[automatically_derived]
        impl ::core::fmt::Debug for ItemTableColumn {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        ItemTableColumn::Name => "Name",
                        ItemTableColumn::Color => "Color",
                        ItemTableColumn::Weight => "Weight",
                        ItemTableColumn::AcquiredOn => "AcquiredOn",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for ItemTableColumn {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ItemTableColumn {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ItemTableColumn {
            #[inline]
            fn eq(&self, other: &ItemTableColumn) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl From<usize> for ItemTableColumn {
            fn from(ix: usize) -> Self {
                match ix {
                    0usize => ItemTableColumn::Name,
                    1usize => ItemTableColumn::Color,
                    2usize => ItemTableColumn::Weight,
                    3usize => ItemTableColumn::AcquiredOn,
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!("Invalid column index: {0}", ix),
                        );
                    }
                }
            }
        }
        impl From<ItemTableColumn> for usize {
            fn from(col: ItemTableColumn) -> Self {
                match col {
                    ItemTableColumn::Name => 0usize,
                    ItemTableColumn::Color => 1usize,
                    ItemTableColumn::Weight => 2usize,
                    ItemTableColumn::AcquiredOn => 3usize,
                }
            }
        }
        impl gpui_table::TableRowMeta for Item {
            const TABLE_ID: &'static str = "Item";
            const TABLE_TITLE: &'static str = "Item";
            fn table_title() -> String {
                Item::this_ftl()
            }
            fn table_columns() -> &'static [gpui_component::table::Column] {
                static COLUMNS: std::sync::OnceLock<
                    Vec<gpui_component::table::Column>,
                > = std::sync::OnceLock::new();
                COLUMNS
                    .get_or_init(|| <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            gpui_component::table::Column::new(
                                    "name",
                                    ItemKvFtl::Name.to_string(),
                                )
                                .width(100f32),
                            gpui_component::table::Column::new(
                                    "color",
                                    ItemKvFtl::Color.to_string(),
                                )
                                .width(80f32),
                            gpui_component::table::Column::new(
                                    "weight",
                                    ItemKvFtl::Weight.to_string(),
                                )
                                .width(60f32),
                            gpui_component::table::Column::new(
                                    "acquired_on",
                                    ItemKvFtl::AcquiredOn.to_string(),
                                )
                                .width(50f32),
                        ]),
                    ))
            }
            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    0usize => Box::new(self.name.clone()),
                    1usize => Box::new(self.color.clone()),
                    2usize => Box::new(self.weight.clone()),
                    3usize => Box::new(self.acquired_on.clone()),
                    _ => Box::new(String::new()),
                }
            }
        }
        pub struct ItemTableDelegate {
            pub rows: Vec<Item>,
            #[new(default)]
            pub visible_rows: std::ops::Range<usize>,
            #[new(default)]
            pub visible_cols: std::ops::Range<usize>,
            #[new(default)]
            pub eof: bool,
            #[new(default)]
            pub loading: bool,
            #[new(default)]
            pub full_loading: bool,
        }
        impl ItemTableDelegate {
            ///Constructs a new `ItemTableDelegate`.
            pub fn new(rows: Vec<Item>) -> Self {
                ItemTableDelegate {
                    rows: rows,
                    visible_rows: ::core::default::Default::default(),
                    visible_cols: ::core::default::Default::default(),
                    eof: ::core::default::Default::default(),
                    loading: ::core::default::Default::default(),
                    full_loading: ::core::default::Default::default(),
                }
            }
        }
        impl gpui_component::table::TableDelegate for ItemTableDelegate {
            fn columns_count(&self, _: &gpui::App) -> usize {
                <Item as gpui_table::TableRowMeta>::table_columns().len()
            }
            fn rows_count(&self, _: &gpui::App) -> usize {
                self.rows.len()
            }
            fn column(
                &self,
                col_ix: usize,
                _: &gpui::App,
            ) -> &gpui_component::table::Column {
                &<Item as gpui_table::TableRowMeta>::table_columns()[col_ix]
            }
            fn render_td(
                &self,
                row_ix: usize,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> impl gpui::IntoElement {
                use gpui_table::TableRowStyle;
                self.rows[row_ix]
                    .render_table_cell(ItemTableColumn::from(col_ix), window, cx)
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
            fn is_eof(&self, _: &gpui::App) -> bool {
                self.eof
            }
            fn loading(&self, _: &gpui::App) -> bool {
                self.loading
            }
            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: gpui_component::table::ColumnSort,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                match col_ix {
                    _ => {}
                }
            }
        }
        impl gpui_table::TableRowStyle for Item {
            type ColumnId = ItemTableColumn;
            fn render_table_cell(
                &self,
                col: Self::ColumnId,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::AnyElement {
                use gpui::{IntoElement, ParentElement, Styled, div};
                match col {
                    ItemTableColumn::Weight => {
                        return div()
                            .child(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("{0} g", self.weight))
                                }),
                            )
                            .text_color(gpui::black())
                            .bg(gpui::white())
                            .px_1()
                            .rounded_md()
                            .into_any_element();
                    }
                    _ => {}
                }
                gpui_table::default_render_cell(self, col.into(), window, cx)
                    .into_any_element()
            }
        }
    }
    pub mod user {
        use es_fluent::EsFluentKv;
        use fake::decimal::PositiveDecimal;
        use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
        use fake::uuid::UUIDv4;
        use gpui_table::{NamedTableRow, TableCell};
        use rust_decimal::Decimal;
        pub enum UserStatus {
            Active,
            Suspended,
            Offline,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UserStatus {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        UserStatus::Active => "Active",
                        UserStatus::Suspended => "Suspended",
                        UserStatus::Offline => "Offline",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UserStatus {
            #[inline]
            fn clone(&self) -> UserStatus {
                match self {
                    UserStatus::Active => UserStatus::Active,
                    UserStatus::Suspended => UserStatus::Suspended,
                    UserStatus::Offline => UserStatus::Offline,
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UserStatus {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UserStatus {
            #[inline]
            fn eq(&self, other: &UserStatus) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl gpui_table::TableCell for UserStatus {
            fn draw(
                &self,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::AnyElement {
                use gpui::IntoElement;
                use es_fluent::ToFluentString as _;
                match self {
                    Self::Active => self.to_fluent_string().into_any_element(),
                    Self::Suspended => self.to_fluent_string().into_any_element(),
                    Self::Offline => self.to_fluent_string().into_any_element(),
                }
            }
        }
        impl ::fake::Dummy<::fake::Faker> for UserStatus {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                rng: &mut R,
            ) -> Self {
                let options = [0usize, 1usize, 2usize];
                match ::fake::rand::seq::IndexedRandom::choose(
                        <_ as ::std::convert::AsRef<[usize]>>::as_ref(&options),
                        rng,
                    )
                    .unwrap()
                {
                    0usize => UserStatus::Active,
                    1usize => UserStatus::Suspended,
                    2usize => UserStatus::Offline,
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
        }
        impl ::es_fluent::FluentDisplay for UserStatus {
            fn fluent_fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> ::std::fmt::Result {
                match self {
                    Self::Active => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_status-Active", None),
                            ),
                        )
                    }
                    Self::Suspended => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_status-Suspended", None),
                            ),
                        )
                    }
                    Self::Offline => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_status-Offline", None),
                            ),
                        )
                    }
                }
            }
        }
        impl From<&UserStatus> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserStatus) -> Self {
                use ::es_fluent::ToFluentString as _;
                value.to_fluent_string().into()
            }
        }
        impl From<UserStatus> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserStatus) -> Self {
                (&value).into()
            }
        }
        #[fluent_kv(this, keys = ["description", "label"])]
        #[table(fluent = "label")]
        pub struct User {
            #[table(skip)]
            #[dummy(faker = "UUIDv4")]
            id: uuid::Uuid,
            #[table(sortable, width = 150.)]
            #[dummy(faker = "Name()")]
            name: String,
            #[table(sortable, width = 80.)]
            #[dummy(faker = "18..67")]
            age: u8,
            #[table(sortable, width = 150.)]
            #[dummy(faker = "PositiveDecimal")]
            debt: Decimal,
            #[table(width = 200.)]
            #[dummy(faker = "SafeEmail()")]
            email: String,
            #[table(width = 70.)]
            active: bool,
            #[table(width = 100.)]
            status: UserStatus,
            #[table(sortable, width = 300.)]
            #[dummy(faker = "DateTime()")]
            created_at: chrono::DateTime<chrono::Utc>,
        }
        impl ::fake::Dummy<::fake::Faker> for User {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                rng: &mut R,
            ) -> Self {
                let id: uuid::Uuid = ::fake::Fake::fake_with_rng::<
                    uuid::Uuid,
                    _,
                >(&(UUIDv4), rng);
                let name: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(Name()), rng);
                let age: u8 = ::fake::Fake::fake_with_rng::<u8, _>(&(18..67), rng);
                let debt: Decimal = ::fake::Fake::fake_with_rng::<
                    Decimal,
                    _,
                >(&(PositiveDecimal), rng);
                let email: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(SafeEmail()), rng);
                let active: bool = ::fake::Fake::fake_with_rng::<
                    bool,
                    _,
                >(&::fake::Faker, rng);
                let status: UserStatus = ::fake::Fake::fake_with_rng::<
                    UserStatus,
                    _,
                >(&::fake::Faker, rng);
                let created_at: chrono::DateTime<chrono::Utc> = ::fake::Fake::fake_with_rng::<
                    chrono::DateTime<chrono::Utc>,
                    _,
                >(&(DateTime()), rng);
                User {
                    id,
                    name,
                    age,
                    debt,
                    email,
                    active,
                    status,
                    created_at,
                }
            }
        }
        pub enum UserDescriptionKvFtl {
            Id,
            Name,
            Age,
            Debt,
            Email,
            Active,
            Status,
            CreatedAt,
        }
        impl ::std::fmt::Display for UserDescriptionKvFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_kv_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_kv_ftl-name", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_kv_ftl-age", None),
                            ),
                        )
                    }
                    Self::Debt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_kv_ftl-debt", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_kv_ftl-email", None),
                            ),
                        )
                    }
                    Self::Active => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_kv_ftl-active",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::Status => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_kv_ftl-status",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::CreatedAt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_kv_ftl-created_at",
                                    None,
                                ),
                            ),
                        )
                    }
                }
            }
        }
        impl UserDescriptionKvFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_description_kv_ftl", None)
            }
        }
        impl From<&UserDescriptionKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserDescriptionKvFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserDescriptionKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserDescriptionKvFtl) -> Self {
                (&value).into()
            }
        }
        pub enum UserLabelKvFtl {
            Id,
            Name,
            Age,
            Debt,
            Email,
            Active,
            Status,
            CreatedAt,
        }
        impl ::std::fmt::Display for UserLabelKvFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-name", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-age", None),
                            ),
                        )
                    }
                    Self::Debt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-debt", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-email", None),
                            ),
                        )
                    }
                    Self::Active => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-active", None),
                            ),
                        )
                    }
                    Self::Status => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-status", None),
                            ),
                        )
                    }
                    Self::CreatedAt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_kv_ftl-created_at", None),
                            ),
                        )
                    }
                }
            }
        }
        impl UserLabelKvFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_label_kv_ftl", None)
            }
        }
        impl From<&UserLabelKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserLabelKvFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserLabelKvFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserLabelKvFtl) -> Self {
                (&value).into()
            }
        }
        impl User {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user", None)
            }
        }
        pub enum UserTableColumn {
            Name,
            Age,
            Debt,
            Email,
            Active,
            Status,
            CreatedAt,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UserTableColumn {
            #[inline]
            fn clone(&self) -> UserTableColumn {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for UserTableColumn {}
        #[automatically_derived]
        impl ::core::fmt::Debug for UserTableColumn {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        UserTableColumn::Name => "Name",
                        UserTableColumn::Age => "Age",
                        UserTableColumn::Debt => "Debt",
                        UserTableColumn::Email => "Email",
                        UserTableColumn::Active => "Active",
                        UserTableColumn::Status => "Status",
                        UserTableColumn::CreatedAt => "CreatedAt",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for UserTableColumn {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for UserTableColumn {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for UserTableColumn {
            #[inline]
            fn eq(&self, other: &UserTableColumn) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl From<usize> for UserTableColumn {
            fn from(ix: usize) -> Self {
                match ix {
                    0usize => UserTableColumn::Name,
                    1usize => UserTableColumn::Age,
                    2usize => UserTableColumn::Debt,
                    3usize => UserTableColumn::Email,
                    4usize => UserTableColumn::Active,
                    5usize => UserTableColumn::Status,
                    6usize => UserTableColumn::CreatedAt,
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!("Invalid column index: {0}", ix),
                        );
                    }
                }
            }
        }
        impl From<UserTableColumn> for usize {
            fn from(col: UserTableColumn) -> Self {
                match col {
                    UserTableColumn::Name => 0usize,
                    UserTableColumn::Age => 1usize,
                    UserTableColumn::Debt => 2usize,
                    UserTableColumn::Email => 3usize,
                    UserTableColumn::Active => 4usize,
                    UserTableColumn::Status => 5usize,
                    UserTableColumn::CreatedAt => 6usize,
                }
            }
        }
        impl gpui_table::TableRowMeta for User {
            const TABLE_ID: &'static str = "User";
            const TABLE_TITLE: &'static str = "User";
            fn table_title() -> String {
                UserLabelKvFtl::this_ftl()
            }
            fn table_columns() -> &'static [gpui_component::table::Column] {
                static COLUMNS: std::sync::OnceLock<
                    Vec<gpui_component::table::Column>,
                > = std::sync::OnceLock::new();
                COLUMNS
                    .get_or_init(|| <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            gpui_component::table::Column::new(
                                    "name",
                                    UserLabelKvFtl::Name.to_string(),
                                )
                                .width(150f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "age",
                                    UserLabelKvFtl::Age.to_string(),
                                )
                                .width(80f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "debt",
                                    UserLabelKvFtl::Debt.to_string(),
                                )
                                .width(150f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "email",
                                    UserLabelKvFtl::Email.to_string(),
                                )
                                .width(200f32),
                            gpui_component::table::Column::new(
                                    "active",
                                    UserLabelKvFtl::Active.to_string(),
                                )
                                .width(70f32),
                            gpui_component::table::Column::new(
                                    "status",
                                    UserLabelKvFtl::Status.to_string(),
                                )
                                .width(100f32),
                            gpui_component::table::Column::new(
                                    "created_at",
                                    UserLabelKvFtl::CreatedAt.to_string(),
                                )
                                .width(300f32)
                                .sortable(),
                        ]),
                    ))
            }
            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    0usize => Box::new(self.name.clone()),
                    1usize => Box::new(self.age.clone()),
                    2usize => Box::new(self.debt.clone()),
                    3usize => Box::new(self.email.clone()),
                    4usize => Box::new(self.active.clone()),
                    5usize => Box::new(self.status.clone()),
                    6usize => Box::new(self.created_at.clone()),
                    _ => Box::new(String::new()),
                }
            }
        }
        impl gpui_table::TableRowStyle for User {
            type ColumnId = UserTableColumn;
            fn render_table_cell(
                &self,
                col: Self::ColumnId,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::AnyElement {
                use gpui::IntoElement;
                gpui_table::default_render_cell(self, col.into(), window, cx)
                    .into_any_element()
            }
        }
        pub struct UserTableDelegate {
            pub rows: Vec<User>,
            #[new(default)]
            pub visible_rows: std::ops::Range<usize>,
            #[new(default)]
            pub visible_cols: std::ops::Range<usize>,
            #[new(default)]
            pub eof: bool,
            #[new(default)]
            pub loading: bool,
            #[new(default)]
            pub full_loading: bool,
        }
        impl UserTableDelegate {
            ///Constructs a new `UserTableDelegate`.
            pub fn new(rows: Vec<User>) -> Self {
                UserTableDelegate {
                    rows: rows,
                    visible_rows: ::core::default::Default::default(),
                    visible_cols: ::core::default::Default::default(),
                    eof: ::core::default::Default::default(),
                    loading: ::core::default::Default::default(),
                    full_loading: ::core::default::Default::default(),
                }
            }
        }
        impl gpui_component::table::TableDelegate for UserTableDelegate {
            fn columns_count(&self, _: &gpui::App) -> usize {
                <User as gpui_table::TableRowMeta>::table_columns().len()
            }
            fn rows_count(&self, _: &gpui::App) -> usize {
                self.rows.len()
            }
            fn column(
                &self,
                col_ix: usize,
                _: &gpui::App,
            ) -> &gpui_component::table::Column {
                &<User as gpui_table::TableRowMeta>::table_columns()[col_ix]
            }
            fn render_td(
                &self,
                row_ix: usize,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> impl gpui::IntoElement {
                use gpui_table::TableRowStyle;
                self.rows[row_ix]
                    .render_table_cell(UserTableColumn::from(col_ix), window, cx)
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
            fn is_eof(&self, _: &gpui::App) -> bool {
                self.eof
            }
            fn loading(&self, _: &gpui::App) -> bool {
                self.loading
            }
            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: gpui_component::table::ColumnSort,
                _: &mut gpui::Window,
                _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                match col_ix {
                    0usize => {
                        self.rows
                            .sort_by(|a, b| {
                                let a_val = &a.name;
                                let b_val = &b.name;
                                match sort {
                                    gpui_component::table::ColumnSort::Ascending => {
                                        a_val
                                            .partial_cmp(b_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    gpui_component::table::ColumnSort::Descending => {
                                        b_val
                                            .partial_cmp(a_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            });
                    }
                    1usize => {
                        self.rows
                            .sort_by(|a, b| {
                                let a_val = &a.age;
                                let b_val = &b.age;
                                match sort {
                                    gpui_component::table::ColumnSort::Ascending => {
                                        a_val
                                            .partial_cmp(b_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    gpui_component::table::ColumnSort::Descending => {
                                        b_val
                                            .partial_cmp(a_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            });
                    }
                    2usize => {
                        self.rows
                            .sort_by(|a, b| {
                                let a_val = &a.debt;
                                let b_val = &b.debt;
                                match sort {
                                    gpui_component::table::ColumnSort::Ascending => {
                                        a_val
                                            .partial_cmp(b_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    gpui_component::table::ColumnSort::Descending => {
                                        b_val
                                            .partial_cmp(a_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            });
                    }
                    6usize => {
                        self.rows
                            .sort_by(|a, b| {
                                let a_val = &a.created_at;
                                let b_val = &b.created_at;
                                match sort {
                                    gpui_component::table::ColumnSort::Ascending => {
                                        a_val
                                            .partial_cmp(b_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    gpui_component::table::ColumnSort::Descending => {
                                        b_val
                                            .partial_cmp(a_val)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => std::cmp::Ordering::Equal,
                                }
                            });
                    }
                    _ => {}
                }
            }
        }
    }
}
