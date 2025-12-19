use gpui::{prelude::*, App, Context, Entity, IntoElement, Render, Window};
use gpui_component::{button::Button, h_flex, label::Label, popover::Popover, v_flex};
use std::rc::Rc;

pub struct NumberRangeFilter {
    title: String,
    min: Option<f64>,
    max: Option<f64>,
    on_change: Rc<dyn Fn((Option<f64>, Option<f64>), &mut Window, &mut App) + 'static>,
}

impl NumberRangeFilter {
    pub fn build(
        title: impl Into<String>,
        range: (Option<f64>, Option<f64>),
        on_change: impl Fn((Option<f64>, Option<f64>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title,
            min: range.0,
            max: range.1,
            on_change: Rc::new(on_change),
        })
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)((self.min, self.max), window, cx);
    }
}

impl Render for NumberRangeFilter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.title.clone();
        let label = match (self.min, self.max) {
            (Some(min), Some(max)) => format!("{} - {}", min, max),
            (Some(min), None) => format!(">= {}", min),
            (None, Some(max)) => format!("<= {}", max),
            (None, None) => title.clone(),
        };

        let view = cx.entity().clone();

        Popover::new("number-filter-popover")
            .trigger(Button::new("number-trigger").label(label))
            .content(move |_, window, cx| {
                let view = view.clone();
                v_flex()
                    .p_2()
                    .gap_2()
                    .w_64()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new("Min"))
                            .child(Label::new("Input Placeholder")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new("Max"))
                            .child(Label::new("Input Placeholder")),
                    )
                    .child(Button::new("apply-btn").label("Apply").on_click(
                        move |_, window, cx| {
                            let view = view.clone();
                            view.update(cx, |this, cx| {
                                this.apply(window, cx);
                            });
                        },
                    ))
            })
    }
}
