use es_fluent::{EsFluentThis, EsFluentVariants};
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use gpui::{App, Window};
use gpui_component::IconName;
use gpui_component::menu::PopupMenu;
use gpui_table::{Filterable, GpuiTable, TableCell, TableRowContextMenu};
use rust_decimal::Decimal;

use crate::structs::context_menu_common;

#[derive(
    Clone, Debug, Eq, Hash, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell,
)]
#[filter(fluent)]
pub enum UserStatus {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(icon = IconName::CircleX)]
    Suspended,
    #[filter(icon = IconName::Moon)]
    Offline,
}

#[derive(Clone, fake::Dummy, EsFluentThis, EsFluentVariants, GpuiTable)]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_table(
    fluent = "label",
    filters,
    custom_context_menu,
    context_menu_route_fn = crate::structs::context_menu_common::user_context_menu_route,
    context_menu_label_fn = crate::structs::context_menu_common::user_context_menu_label
)]
pub struct User {
    #[gpui_table(skip, context_menu_id)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    pub id: uuid::Uuid,

    #[gpui_table(sortable, width = 150., filter(text()))]
    #[dummy(faker = "Name()")]
    pub name: String,

    #[gpui_table(sortable, width = 80., filter(number_range()))]
    #[dummy(faker = "18..67")]
    pub age: u8,

    #[gpui_table(sortable, width = 150., filter(number_range()))]
    #[dummy(faker = "PositiveDecimal")]
    pub debt: Decimal,

    #[gpui_table(width = 200., filter(text()))]
    #[dummy(faker = "SafeEmail()")]
    pub email: String,

    #[gpui_table(width = 70., filter(faceted()))]
    pub active: bool,

    #[gpui_table(width = 100., filter(faceted()))]
    pub status: UserStatus,

    #[gpui_table(sortable, width = 300., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(not(feature = "router"))]
impl TableRowContextMenu for User {
    fn render_table_context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> PopupMenu {
        use gpui_table::TableRowGeneratedContextMenu as _;
        let menu = self.render_generated_table_context_menu(row_ix, menu, window, cx);
        context_menu_common::with_user_common_actions(menu, &self.id)
    }
}

#[cfg(feature = "router")]
impl TableRowContextMenu for User {
    fn render_table_context_menu(
        &self,
        _row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut App,
    ) -> PopupMenu {
        context_menu_common::with_user_common_actions(menu, &self.id)
    }
}
