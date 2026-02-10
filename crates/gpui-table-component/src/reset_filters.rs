use es_fluent::{EsFluent, ToFluentString as _};
use gpui::{
    App, IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div,
};
use gpui_component::{Sizable as _, StyledExt as _, button::Button};
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
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_reset = self.on_reset.clone();

        div().refine_style(&self.style).child(
            Button::new(self.button_id)
                .outline()
                .small()
                .label(ResetFiltersFtl::Reset.to_fluent_string())
                .refine_style(&self.button_style)
                .on_click(move |_, window, cx| {
                    on_reset(window, cx);
                }),
        )
    }
}
