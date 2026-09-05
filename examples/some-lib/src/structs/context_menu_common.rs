use gpui_kit::component::menu::PopupMenu;
#[cfg(feature = "router")]
use gpui_kit::gpui;
#[cfg(feature = "router")]
use std::fmt;

pub fn user_context_menu_route(id: &uuid::Uuid) -> String {
    format!("#user-{id}")
}

pub fn user_context_menu_label(id: &uuid::Uuid) -> String {
    format!("Open user ({id})")
}

#[cfg(feature = "router")]
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
pub enum UserRoute {
    User { id: String },
}

#[cfg(feature = "router")]
impl UserRoute {
    pub fn user(id: &uuid::Uuid) -> Self {
        Self::User { id: id.to_string() }
    }

    pub fn path(&self) -> String {
        match self {
            Self::User { id } => format!("/users/{id}"),
        }
    }
}

#[cfg(feature = "router")]
impl fmt::Display for UserRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.path())
    }
}

#[cfg(feature = "router")]
#[derive(gpui_kit::Action, Clone, serde::Deserialize, Eq, PartialEq)]
#[action(namespace = some_lib_user_context_menu, no_json)]
pub struct OpenUserRoute(pub UserRoute);

#[cfg(feature = "router")]
pub fn with_user_common_actions(id: &uuid::Uuid, menu: PopupMenu) -> PopupMenu {
    menu.menu(
        user_context_menu_label(id),
        Box::new(OpenUserRoute(UserRoute::user(id))),
    )
    .separator()
    .link("Share user", format!("/share/{id}"))
}

#[cfg(not(feature = "router"))]
pub fn with_user_common_actions(id: &uuid::Uuid, menu: PopupMenu) -> PopupMenu {
    menu.separator().link("Share user", format!("/share/{id}"))
}
