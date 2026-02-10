use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, Render, StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{Sizable as _, StyledExt as _, button::Button, h_flex, v_flex};
use rust_decimal::Decimal;

use crate::{NumberRangeFilterExt as _, number_range_filter::NumberRangeFilter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumberRangeStoryMode {
    Basic,
    Bounded,
    Styled,
}

impl NumberRangeStoryMode {
    fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Bounded => "Bounded",
            Self::Styled => "Styled",
        }
    }
}

#[gpui_storybook::story("Table Filters")]
pub struct NumberRangeFilterStory {
    focus_handle: FocusHandle,
    mode: NumberRangeStoryMode,
    number_range_filter: Entity<NumberRangeFilter>,
}

impl NumberRangeFilterStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mode = NumberRangeStoryMode::Bounded;
        let number_range_filter = Self::build_filter(mode, cx);

        Self {
            focus_handle: cx.focus_handle(),
            mode,
            number_range_filter,
        }
    }

    fn build_filter(mode: NumberRangeStoryMode, cx: &mut App) -> Entity<NumberRangeFilter> {
        match mode {
            NumberRangeStoryMode::Basic => NumberRangeFilter::new_for(
                || "Price".to_string(),
                (None, None),
                |_value, _window, _cx| {},
                cx,
            ),
            NumberRangeStoryMode::Bounded => NumberRangeFilter::new_for(
                || "Price".to_string(),
                (Some(Decimal::new(10, 0)), Some(Decimal::new(80, 0))),
                |_value, _window, _cx| {},
                cx,
            )
            .range(Decimal::ZERO, Decimal::new(100, 0), cx)
            .step(Decimal::new(5, 0), cx),
            NumberRangeStoryMode::Styled => NumberRangeFilter::new_for(
                || "Weight".to_string(),
                (Some(Decimal::new(5, 0)), Some(Decimal::new(25, 0))),
                |_value, _window, _cx| {},
                cx,
            )
            .range(Decimal::ZERO, Decimal::new(50, 0), cx)
            .step(Decimal::ONE, cx)
            .trigger_style(StyleRefinement::default().w(px(360.)), cx)
            .popover_style(StyleRefinement::default().w(px(420.)), cx)
            .inputs_row_style(StyleRefinement::default().w_full(), cx)
            .min_input_style(StyleRefinement::default().w(px(150.)), cx)
            .max_input_style(StyleRefinement::default().w(px(150.)), cx)
            .between_style(StyleRefinement::default().font_semibold(), cx)
            .slider_style(StyleRefinement::default().w_full(), cx)
            .clear_button_style(StyleRefinement::default().font_semibold(), cx),
        }
    }

    fn set_mode(&mut self, mode: NumberRangeStoryMode, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.number_range_filter = Self::build_filter(mode, cx);
        cx.notify();
    }
}

impl Focusable for NumberRangeFilterStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for NumberRangeFilterStory {
    fn title() -> String {
        "Number Range Filter".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for NumberRangeFilterStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (min, max) = self.number_range_filter.read(cx).value();
        let value_display = format!("Current range: min={min:?}, max={max:?}");

        v_flex()
            .id("number-range-filter-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("NumberRangeFilter modes"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("number-range-story-mode-basic")
                            .outline()
                            .small()
                            .label(NumberRangeStoryMode::Basic.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(NumberRangeStoryMode::Basic, cx);
                            })),
                    )
                    .child(
                        Button::new("number-range-story-mode-bounded")
                            .outline()
                            .small()
                            .label(NumberRangeStoryMode::Bounded.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(NumberRangeStoryMode::Bounded, cx);
                            })),
                    )
                    .child(
                        Button::new("number-range-story-mode-styled")
                            .outline()
                            .small()
                            .label(NumberRangeStoryMode::Styled.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(NumberRangeStoryMode::Styled, cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .child(format!("Current mode: {}", self.mode.label())),
                    )
                    .child(self.number_range_filter.clone())
                    .child(div().text_sm().child(value_display)),
            )
    }
}
