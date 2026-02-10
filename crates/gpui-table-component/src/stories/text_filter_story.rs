use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, Render, StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{StyledExt as _, v_flex};

use crate::{TextFilterExt as _, text_filter::TextFilter};

#[gpui_storybook::story("Table Filters")]
pub struct TextFilterStory {
    focus_handle: FocusHandle,
    default_filter: Entity<TextFilter>,
    numeric_filter: Entity<TextFilter>,
    alphanumeric_filter: Entity<TextFilter>,
}

impl TextFilterStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let default_filter = TextFilter::new_for(
            || "Search".to_string(),
            String::new(),
            |_value, _window, _cx| {},
            cx,
        )
        .container_style(StyleRefinement::default().w(px(420.)), cx)
        .input_style(StyleRefinement::default().w(px(360.)), cx);

        let numeric_filter = TextFilter::new_for(
            || "Order ID".to_string(),
            "42".to_string(),
            |_value, _window, _cx| {},
            cx,
        )
        .numeric_only(cx)
        .container_style(StyleRefinement::default().w(px(420.)), cx)
        .input_style(StyleRefinement::default().w(px(280.)), cx);

        let alphanumeric_filter = TextFilter::new_for(
            || "SKU".to_string(),
            "SKU-123".to_string(),
            |_value, _window, _cx| {},
            cx,
        )
        .alphanumeric_only(cx)
        .container_style(StyleRefinement::default().w(px(420.)), cx)
        .input_style(StyleRefinement::default().w(px(280.)), cx);

        Self {
            focus_handle: cx.focus_handle(),
            default_filter,
            numeric_filter,
            alphanumeric_filter,
        }
    }
}

impl Focusable for TextFilterStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for TextFilterStory {
    fn title() -> String {
        "Text Filter".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for TextFilterStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let default_value = self.default_filter.read(cx).value().to_string();
        let numeric_value = self.numeric_filter.read(cx).value().to_string();
        let alphanumeric_value = self.alphanumeric_filter.read(cx).value().to_string();

        v_flex()
            .id("text-filter-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(div().text_lg().font_semibold().child("TextFilter modes"))
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Default"))
                    .child(self.default_filter.clone())
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Current value: \"{default_value}\"")),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Numeric only"))
                    .child(self.numeric_filter.clone())
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Current value: \"{numeric_value}\"")),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Alphanumeric only"))
                    .child(self.alphanumeric_filter.clone())
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Current value: \"{alphanumeric_value}\"")),
                    ),
            )
    }
}
