use chrono::NaiveDate;
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, Render, StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{Sizable as _, StyledExt as _, button::Button, h_flex, v_flex};

use crate::{DateRangeFilterExt as _, date_range_filter::DateRangeFilter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DateRangeStoryMode {
    Empty,
    Preset,
    Styled,
}

impl DateRangeStoryMode {
    fn label(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Preset => "Preset",
            Self::Styled => "Styled",
        }
    }
}

#[gpui_storybook::story("Table Filters")]
pub struct DateRangeFilterStory {
    focus_handle: FocusHandle,
    mode: DateRangeStoryMode,
    date_range_filter: Entity<DateRangeFilter>,
}

impl DateRangeFilterStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mode = DateRangeStoryMode::Preset;
        let date_range_filter = Self::build_filter(mode, cx);

        Self {
            focus_handle: cx.focus_handle(),
            mode,
            date_range_filter,
        }
    }

    fn build_filter(mode: DateRangeStoryMode, cx: &mut App) -> Entity<DateRangeFilter> {
        match mode {
            DateRangeStoryMode::Empty => DateRangeFilter::new_for(
                || "Created at".to_string(),
                (None, None),
                |_value, _window, _cx| {},
                cx,
            ),
            DateRangeStoryMode::Preset => DateRangeFilter::new_for(
                || "Created at".to_string(),
                (
                    NaiveDate::from_ymd_opt(2026, 1, 1),
                    NaiveDate::from_ymd_opt(2026, 1, 31),
                ),
                |_value, _window, _cx| {},
                cx,
            ),
            DateRangeStoryMode::Styled => DateRangeFilter::new_for(
                || "Updated at".to_string(),
                (
                    NaiveDate::from_ymd_opt(2026, 2, 1),
                    NaiveDate::from_ymd_opt(2026, 2, 14),
                ),
                |_value, _window, _cx| {},
                cx,
            )
            .trigger_style(StyleRefinement::default().w(px(360.)), cx)
            .popover_style(StyleRefinement::default().w(px(420.)), cx)
            .calendar_style(StyleRefinement::default().w_full(), cx)
            .clear_button_style(StyleRefinement::default().font_semibold(), cx),
        }
    }

    fn set_mode(&mut self, mode: DateRangeStoryMode, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.date_range_filter = Self::build_filter(mode, cx);
        cx.notify();
    }
}

impl Focusable for DateRangeFilterStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for DateRangeFilterStory {
    fn title() -> String {
        "Date Range Filter".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for DateRangeFilterStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (start, end) = self.date_range_filter.read(cx).value();
        let value_display = format!("Current range: start={start:?}, end={end:?}");

        v_flex()
            .id("date-range-filter-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("DateRangeFilter modes"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("date-range-story-mode-empty")
                            .outline()
                            .small()
                            .label(DateRangeStoryMode::Empty.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(DateRangeStoryMode::Empty, cx);
                            })),
                    )
                    .child(
                        Button::new("date-range-story-mode-preset")
                            .outline()
                            .small()
                            .label(DateRangeStoryMode::Preset.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(DateRangeStoryMode::Preset, cx);
                            })),
                    )
                    .child(
                        Button::new("date-range-story-mode-styled")
                            .outline()
                            .small()
                            .label(DateRangeStoryMode::Styled.label())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_mode(DateRangeStoryMode::Styled, cx);
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
                    .child(self.date_range_filter.clone())
                    .child(div().text_sm().child(value_display)),
            )
    }
}
