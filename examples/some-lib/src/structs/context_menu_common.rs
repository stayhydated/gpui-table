use gpui_component::menu::PopupMenu;

pub fn user_context_menu_route(id: &uuid::Uuid) -> String {
    format!("#user-{id}")
}

pub fn user_context_menu_label(id: &uuid::Uuid) -> String {
    format!("Open user ({id})")
}

pub fn with_user_common_actions(menu: PopupMenu, id: &uuid::Uuid) -> PopupMenu {
    menu.separator().link("Share user", format!("/share/{id}"))
}
