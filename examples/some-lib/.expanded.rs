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
    pub mod fruit {
        use es_fluent::EsFluentKv;
        use gpui_table::NamedTableRow;
        #[fluent_kv(display = "std")]
        #[fluent_kv(this)]
        #[table(fluent, custom_style)]
        pub struct Fruit {
            #[table(skip)]
            id: usize,
            #[table(width = 100.)]
            name: String,
            #[table(width = 80.)]
            color: String,
            #[table(width = 60.)]
            weight_grams: u32,
            #[table(width = 50.)]
            ripe: bool,
        }
        impl gpui_table::TableRowMeta for Fruit {
            const TABLE_ID: &'static str = "Fruit";
            const TABLE_TITLE: &'static str = "Fruit";
            fn table_title() -> String {
                FruitFtl::this_ftl()
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
                                    FruitFtl::Name.to_string(),
                                )
                                .width(100f32),
                            gpui_component::table::Column::new(
                                    "color",
                                    FruitFtl::Color.to_string(),
                                )
                                .width(80f32),
                            gpui_component::table::Column::new(
                                    "weight_grams",
                                    FruitFtl::WeightGrams.to_string(),
                                )
                                .width(60f32),
                            gpui_component::table::Column::new(
                                    "ripe",
                                    FruitFtl::Ripe.to_string(),
                                )
                                .width(50f32),
                        ]),
                    ))
            }
            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    0usize => Box::new(self.name.clone()),
                    1usize => Box::new(self.color.clone()),
                    2usize => Box::new(self.weight_grams.clone()),
                    3usize => Box::new(self.ripe.clone()),
                    _ => Box::new(String::new()),
                }
            }
        }
        pub struct FruitTableDelegate {
            pub rows: Vec<Fruit>,
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
        impl FruitTableDelegate {
            ///Constructs a new `FruitTableDelegate`.
            pub fn new(rows: Vec<Fruit>) -> Self {
                FruitTableDelegate {
                    rows: rows,
                    visible_rows: ::core::default::Default::default(),
                    visible_cols: ::core::default::Default::default(),
                    eof: ::core::default::Default::default(),
                    loading: ::core::default::Default::default(),
                    full_loading: ::core::default::Default::default(),
                }
            }
        }
        impl gpui_component::table::TableDelegate for FruitTableDelegate {
            fn columns_count(&self, _: &gpui::App) -> usize {
                <Fruit as gpui_table::TableRowMeta>::table_columns().len()
            }
            fn rows_count(&self, _: &gpui::App) -> usize {
                self.rows.len()
            }
            fn column(
                &self,
                col_ix: usize,
                _: &gpui::App,
            ) -> &gpui_component::table::Column {
                &<Fruit as gpui_table::TableRowMeta>::table_columns()[col_ix]
            }
            fn render_td(
                &self,
                row_ix: usize,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> impl gpui::IntoElement {
                use gpui_table::TableRowStyle;
                self.rows[row_ix].render_table_cell(col_ix, window, cx)
            }
            fn render_tr(
                &self,
                row_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::Stateful<gpui::Div> {
                use gpui_table::TableRowStyle;
                self.rows[row_ix].render_table_row(row_ix, window, cx)
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
        pub enum FruitFtl {
            Id,
            Name,
            Color,
            WeightGrams,
            Ripe,
        }
        impl ::std::fmt::Display for FruitFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("fruit_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("fruit_ftl-name", None),
                            ),
                        )
                    }
                    Self::Color => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("fruit_ftl-color", None),
                            ),
                        )
                    }
                    Self::WeightGrams => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("fruit_ftl-weight_grams", None),
                            ),
                        )
                    }
                    Self::Ripe => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("fruit_ftl-ripe", None),
                            ),
                        )
                    }
                }
            }
        }
        impl FruitFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("fruit_ftl", None)
            }
        }
        impl From<&FruitFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &FruitFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<FruitFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: FruitFtl) -> Self {
                (&value).into()
            }
        }
        impl ::fake::Dummy<::fake::Faker> for Fruit {
            fn dummy_with_rng<R: ::fake::Rng + ?Sized>(
                _: &::fake::Faker,
                rng: &mut R,
            ) -> Self {
                let id: usize = ::fake::Fake::fake_with_rng::<
                    usize,
                    _,
                >(&::fake::Faker, rng);
                let name: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&::fake::Faker, rng);
                let color: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&::fake::Faker, rng);
                let weight_grams: u32 = ::fake::Fake::fake_with_rng::<
                    u32,
                    _,
                >(&::fake::Faker, rng);
                let ripe: bool = ::fake::Fake::fake_with_rng::<
                    bool,
                    _,
                >(&::fake::Faker, rng);
                Fruit {
                    id,
                    name,
                    color,
                    weight_grams,
                    ripe,
                }
            }
        }
        impl gpui_table::TableRowStyle for Fruit {
            fn render_table_cell(
                &self,
                col_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::AnyElement {
                use gpui::{IntoElement, ParentElement, Styled, div};
                if col_ix == 3 {
                    if self.ripe {
                        return div()
                            .child("RIPE")
                            .text_color(gpui::red())
                            .bg(gpui::yellow())
                            .px_1()
                            .rounded_md()
                            .into_any_element();
                    }
                }
                gpui_table::default_render_cell(self, col_ix, window, cx)
                    .into_any_element()
            }
        }
    }
    pub mod user {
        use es_fluent::EsFluentKv;
        use fake::faker::{internet::en::SafeEmail, name::en::Name};
        use fake::uuid::UUIDv4;
        use gpui_table::NamedTableRow;
        #[fluent_kv(display = "std")]
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
            #[dummy(faker = "18..90")]
            age: u8,
            #[table(width = 200.)]
            #[dummy(faker = "SafeEmail()")]
            email: String,
            #[table(width = 50.)]
            active: bool,
        }
        impl gpui_table::TableRowMeta for User {
            const TABLE_ID: &'static str = "User";
            const TABLE_TITLE: &'static str = "User";
            fn table_title() -> String {
                UserLabelFtl::this_ftl()
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
                                    UserLabelFtl::Name.to_string(),
                                )
                                .width(150f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "age",
                                    UserLabelFtl::Age.to_string(),
                                )
                                .width(80f32)
                                .sortable(),
                            gpui_component::table::Column::new(
                                    "email",
                                    UserLabelFtl::Email.to_string(),
                                )
                                .width(200f32),
                            gpui_component::table::Column::new(
                                    "active",
                                    UserLabelFtl::Active.to_string(),
                                )
                                .width(50f32),
                        ]),
                    ))
            }
            fn cell_value(&self, col_ix: usize) -> Box<dyn gpui_table::TableCell + '_> {
                match col_ix {
                    0usize => Box::new(self.name.clone()),
                    1usize => Box::new(self.age.clone()),
                    2usize => Box::new(self.email.clone()),
                    3usize => Box::new(self.active.clone()),
                    _ => Box::new(String::new()),
                }
            }
        }
        impl gpui_table::TableRowStyle for User {}
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
                self.rows[row_ix].render_table_cell(col_ix, window, cx)
            }
            fn render_tr(
                &self,
                row_ix: usize,
                window: &mut gpui::Window,
                cx: &mut gpui::App,
            ) -> gpui::Stateful<gpui::Div> {
                use gpui_table::TableRowStyle;
                self.rows[row_ix].render_table_row(row_ix, window, cx)
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
                    _ => {}
                }
            }
        }
        pub enum UserDescriptionFtl {
            Id,
            Name,
            Age,
            Email,
            Active,
        }
        impl ::std::fmt::Display for UserDescriptionFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-name", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-age", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-email", None),
                            ),
                        )
                    }
                    Self::Active => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-active", None),
                            ),
                        )
                    }
                }
            }
        }
        impl UserDescriptionFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_description_ftl", None)
            }
        }
        impl From<&UserDescriptionFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserDescriptionFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserDescriptionFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserDescriptionFtl) -> Self {
                (&value).into()
            }
        }
        pub enum UserLabelFtl {
            Id,
            Name,
            Age,
            Email,
            Active,
        }
        impl ::std::fmt::Display for UserLabelFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-name", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-age", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-email", None),
                            ),
                        )
                    }
                    Self::Active => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-active", None),
                            ),
                        )
                    }
                }
            }
        }
        impl UserLabelFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_label_ftl", None)
            }
        }
        impl From<&UserLabelFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserLabelFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserLabelFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserLabelFtl) -> Self {
                (&value).into()
            }
        }
        impl User {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user", None)
            }
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
                let age: u8 = ::fake::Fake::fake_with_rng::<u8, _>(&(18..90), rng);
                let email: String = ::fake::Fake::fake_with_rng::<
                    String,
                    _,
                >(&(SafeEmail()), rng);
                let active: bool = ::fake::Fake::fake_with_rng::<
                    bool,
                    _,
                >(&::fake::Faker, rng);
                User {
                    id,
                    name,
                    age,
                    email,
                    active,
                }
            }
        }
    }
}
