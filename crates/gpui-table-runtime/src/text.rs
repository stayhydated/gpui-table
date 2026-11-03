//! Layout-preserving readable text for table cell values.
use gpui_kit::{
    A11ySubtreeBuilder, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Role, SharedString, Window, accesskit,
};

/// Renders the same text visually and through the native accessibility tree.
/// Custom cell formatters can use this helper for their final displayed value.
/// Place multiple text cells under distinct GPUI element ID scopes.
pub fn text_cell(value: impl Into<SharedString>) -> gpui_kit::AnyElement {
    let value = value.into();
    CellText {
        inner: value.clone().into_element(),
        value,
    }
    .into_any_element()
}

struct CellText<E> {
    inner: E,
    value: SharedString,
}
impl<E> CellText<E> {
    fn text_run(&self) -> accesskit::Node {
        let mut node = accesskit::Node::new(Role::TextRun);
        node.set_value(self.value.to_string());
        node.set_character_lengths(
            self.value
                .chars()
                .map(|c| c.len_utf8() as u8)
                .collect::<Vec<_>>(),
        );
        node
    }
}
impl<E: Element> IntoElement for CellText<E> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl<E: Element> Element for CellText<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some("table-cell-text".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .paint(id, inspector, bounds, layout, prepaint, window, cx)
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(Role::Label)
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        node.set_label(self.value.to_string());
    }

    fn a11y_synthetic_children(
        &mut self,
        _: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        let id = builder.synthetic_node_id(0);
        let mut run = self.text_run();
        if let Some(bounds) = builder.parent_node().bounds() {
            run.set_bounds(bounds);
        }
        builder.push_child(id, run);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cell_text_exposes_the_displayed_unicode_without_focus_or_actions() {
        let value: SharedString = "Parity recipient — Montréal 🦀".into();
        let cell = CellText {
            inner: value.clone().into_element(),
            value: value.clone(),
        };
        assert_eq!(cell.a11y_role(), Some(Role::Label));
        let mut node = accesskit::Node::new(cell.a11y_role().unwrap());
        cell.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some(value.as_ref()));
        assert!(!node.supports_action(accesskit::Action::Focus));
        assert!(!node.supports_action(accesskit::Action::Click));
        assert_eq!(cell.id(), Some("table-cell-text".into()));
        let run = cell.text_run();
        assert_eq!(run.value(), Some(value.as_ref()));
        assert_eq!(
            run.character_lengths()
                .iter()
                .map(|n| usize::from(*n))
                .sum::<usize>(),
            value.len()
        );
        assert_eq!(run.character_lengths().len(), value.chars().count());
    }
}
