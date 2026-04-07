use gpui_component::menu::PopupMenu;

pub fn user_context_menu_route(id: &uuid::Uuid) -> String {
    format!("#user-{id}")
}

pub fn user_context_menu_label(id: &uuid::Uuid) -> String {
    format!("Open user ({id})")
}

#[cfg(feature = "router")]
fn user_context_menu_router_path(id: &uuid::Uuid) -> gpui::SharedString {
    format!("/users/{id}").into()
}

#[cfg(feature = "router")]
#[derive(gpui::Action, Clone, PartialEq, Eq, serde::Deserialize)]
#[action(namespace = some_lib_user_context_menu, no_json)]
pub struct OpenUserRoute(pub gpui::SharedString);

#[cfg(feature = "router")]
pub fn with_user_common_actions(menu: PopupMenu, id: &uuid::Uuid) -> PopupMenu {
    menu.menu(
        user_context_menu_label(id),
        Box::new(OpenUserRoute(user_context_menu_router_path(id))),
    )
    .separator()
    .link("Share user", format!("/share/{id}"))
}

#[cfg(not(feature = "router"))]
pub fn with_user_common_actions(menu: PopupMenu, id: &uuid::Uuid) -> PopupMenu {
    menu.separator().link("Share user", format!("/share/{id}"))
}
