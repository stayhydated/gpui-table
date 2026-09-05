use es_fluent::EsFluent;
use gpui_kit::component::{Sizable as _, StyledExt as _, button::Button};
use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div, gpui,
};
use std::rc::Rc;

#[derive(Clone, Copy, EsFluent)]
enum ResetFiltersFtl {
    Reset,
}

/// Reset control that clears all generated table filters.
#[derive(IntoElement)]
pub struct ResetFilters {
    style: StyleRefinement,
    button_style: StyleRefinement,
    button_id: String,
    on_reset: Rc<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl ResetFilters {
    /// Create a reset control with the given callback.
    pub fn new(on_reset: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Self {
            style: StyleRefinement::default(),
            button_style: StyleRefinement::default(),
            button_id: "table-reset-filters".to_string(),
            on_reset: Rc::new(on_reset),
        }
    }

    /// Set a custom id for the underlying button.
    pub fn button_id(mut self, id: impl Into<String>) -> Self {
        self.button_id = id.into();
        self
    }

    /// Set style refinement for the reset button.
    pub fn button_style(mut self, style: StyleRefinement) -> Self {
        self.button_style = style;
        self
    }
}

impl Styled for ResetFilters {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ResetFilters {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_reset = self.on_reset.clone();

        div().refine_style(&self.style).child(
            Button::new(self.button_id)
                .outline()
                .small()
                .label(crate::i18n::localize_message(cx, &ResetFiltersFtl::Reset))
                .refine_style(&self.button_style)
                .on_click(move |_, window, cx| {
                    on_reset(window, cx);
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ResetFilters;
    use gpui_kit::gpui;
    use gpui_kit::{
        Empty, IntoElement as _, RenderOnce as _, StyleRefinement, Styled as _, TestAppContext,
        VisualTestContext,
    };

    #[gpui_kit::test]
    fn reset_control_builders_render_with_default_and_custom_ids(cx: &mut TestAppContext) {
        cx.update(gpui_kit::component::init);
        cx.update(|cx| crate::i18n::init(cx).unwrap());
        let default = ResetFilters::new(|_, _| {});
        assert_eq!(default.button_id, "table-reset-filters");

        let mut custom = ResetFilters::new(|_, _| {})
            .button_id("clear-all")
            .button_style(StyleRefinement::default());
        *custom.style() = StyleRefinement::default();
        assert_eq!(custom.button_id, "clear-all");

        let window = cx.add_window(|_, _| Empty);
        let mut visual = VisualTestContext::from_window(window.into(), cx);
        visual.update(|window, cx| {
            let _ = default.render(window, cx).into_any_element();
            let _ = custom.render(window, cx).into_any_element();
        });
    }
}
