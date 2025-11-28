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
    pub mod item {
        use es_fluent::EsFluentKv;
        use fake::faker::{chrono::en::DateTime, color::en::Color, lorem::en::Word};
        use fake::uuid::UUIDv4;
        use gpui_table::NamedTableRow;
        #[fluent_kv(display = "std")]
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
        pub enum ItemFtl {
            Id,
            Name,
            Color,
            Weight,
            AcquiredOn,
        }
        impl ::std::fmt::Display for ItemFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Id => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_ftl-id", None),
                            ),
                        )
                    }
                    Self::Name => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_ftl-name", None),
                            ),
                        )
                    }
                    Self::Color => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_ftl-color", None),
                            ),
                        )
                    }
                    Self::Weight => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_ftl-weight", None),
                            ),
                        )
                    }
                    Self::AcquiredOn => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("item_ftl-acquired_on", None),
                            ),
                        )
                    }
                }
            }
        }
        impl ItemFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("item_ftl", None)
            }
        }
        impl From<&ItemFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &ItemFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<ItemFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: ItemFtl) -> Self {
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
                ItemFtl::this_ftl()
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
                                    ItemFtl::Name.to_string(),
                                )
                                .width(100f32),
                            gpui_component::table::Column::new(
                                    "color",
                                    ItemFtl::Color.to_string(),
                                )
                                .width(80f32),
                            gpui_component::table::Column::new(
                                    "weight",
                                    ItemFtl::Weight.to_string(),
                                )
                                .width(60f32),
                            gpui_component::table::Column::new(
                                    "acquired_on",
                                    ItemFtl::AcquiredOn.to_string(),
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
        use gpui_table::NamedTableRow;
        use rust_decimal::Decimal;
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
                    created_at,
                }
            }
        }
        pub enum UserDescriptionFtl {
            Id,
            Name,
            Age,
            Debt,
            Email,
            Active,
            CreatedAt,
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
                    Self::Debt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-debt", None),
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
                    Self::CreatedAt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_ftl-created_at",
                                    None,
                                ),
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
            Debt,
            Email,
            Active,
            CreatedAt,
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
                    Self::Debt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-debt", None),
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
                    Self::CreatedAt => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-created_at", None),
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
        pub enum UserTableColumn {
            Name,
            Age,
            Debt,
            Email,
            Active,
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
                    5usize => UserTableColumn::CreatedAt,
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
                    UserTableColumn::CreatedAt => 5usize,
                }
            }
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
                                    "debt",
                                    UserLabelFtl::Debt.to_string(),
                                )
                                .width(150f32)
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
                                .width(70f32),
                            gpui_component::table::Column::new(
                                    "created_at",
                                    UserLabelFtl::CreatedAt.to_string(),
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
                    5usize => Box::new(self.created_at.clone()),
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
                    5usize => {
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
