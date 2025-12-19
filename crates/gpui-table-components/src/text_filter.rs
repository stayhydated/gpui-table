use gpui::{prelude::*, App, Context, Entity, IntoElement, Render, Window};
use gpui_component::{button::Button, label::Label, popover::Popover, v_flex, IconName};
use std::rc::Rc;

pub struct TextFilter {
    title: String,
    value: String,
    on_change: Rc<dyn Fn(String, &mut Window, &mut App) + 'static>,
}

impl TextFilter {
    pub fn build(
        title: impl Into<String>,
        value: String,
        on_change: impl Fn(String, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title: title.into(),
            value,
            on_change: Rc::new(on_change),
        })
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)(self.value.clone(), window, cx);
    }
}

impl Render for TextFilter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.title.clone();
        let display_label = if self.value.is_empty() {
            title.clone()
        } else {
            format!("{}: {}", title, self.value)
        };

        let view = cx.entity().clone();

        Popover::new("text-filter-popover")
            .trigger(
                Button::new("text-trigger")
                    .label(display_label)
                    .icon(IconName::Search),
            )
            .content(move |_, window, cx| {
                let view = view.clone();
                v_flex()
                    .p_2()
                    .w_64()
                    .gap_2()
                    .child(Label::new("Search Input Placeholder"))
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
