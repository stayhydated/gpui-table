use std::collections::HashSet;

use chrono::NaiveDate;
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, Render,
    StyleRefinement, Styled as _, Window, div, prelude::*, px,
};
use gpui_component::{StyledExt as _, v_flex};
use rust_decimal::Decimal;

use crate::{
    NumberRangeFilterExt as _, date_range_filter::DateRangeFilter, faceted_filter::FacetedFilter,
    number_range_filter::NumberRangeFilter, reset_filters::ResetFilters, text_filter::TextFilter,
};

#[gpui_storybook::story("Table Filters")]
pub struct ResetFiltersStory {
    focus_handle: FocusHandle,
    text_filter: Entity<TextFilter>,
    faceted_filter: Entity<FacetedFilter<bool>>,
    number_filter: Entity<NumberRangeFilter>,
    date_filter: Entity<DateRangeFilter>,
    reset_count: usize,
}

impl ResetFiltersStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut selected_values = HashSet::new();
        selected_values.insert(true);

        Self {
            focus_handle: cx.focus_handle(),
            text_filter: TextFilter::new_for(
                || "Title".to_string(),
                "Invoice 42".to_string(),
                |_value, _window, _cx| {},
                cx,
            ),
            faceted_filter: FacetedFilter::<bool>::new_for(
                || "Archived".to_string(),
                selected_values,
                |_value, _window, _cx| {},
                cx,
            ),
            number_filter: NumberRangeFilter::new_for(
                || "Price".to_string(),
                (Some(Decimal::new(10, 0)), Some(Decimal::new(90, 0))),
                |_value, _window, _cx| {},
                cx,
            )
            .range(Decimal::ZERO, Decimal::new(100, 0), cx)
            .step(Decimal::new(5, 0), cx),
            date_filter: DateRangeFilter::new_for(
                || "Created at".to_string(),
                (
                    NaiveDate::from_ymd_opt(2026, 1, 1),
                    NaiveDate::from_ymd_opt(2026, 1, 31),
                ),
                |_value, _window, _cx| {},
                cx,
            ),
            reset_count: 0,
        }
    }
}

impl Focusable for ResetFiltersStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for ResetFiltersStory {
    fn title() -> String {
        "Reset Filters".into()
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Render for ResetFiltersStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_value = self.text_filter.read(cx).value().to_string();
        let (min, max) = self.number_filter.read(cx).value();
        let (start, end) = self.date_filter.read(cx).value();
        let mut faceted_values = self
            .faceted_filter
            .read(cx)
            .value()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        faceted_values.sort();
        let faceted_display = if faceted_values.is_empty() {
            "(none)".to_string()
        } else {
            faceted_values.join(", ")
        };

        let view = cx.entity().clone();
        let text_filter = self.text_filter.clone();
        let faceted_filter = self.faceted_filter.clone();
        let number_filter = self.number_filter.clone();
        let date_filter = self.date_filter.clone();
        let reset_control = ResetFilters::new(move |window, cx| {
            text_filter.update(cx, |filter, cx| {
                filter.reset_silent(window, cx);
            });
            faceted_filter.update(cx, |filter, cx| {
                filter.reset_silent(window, cx);
            });
            number_filter.update(cx, |filter, cx| {
                filter.reset_silent(window, cx);
            });
            date_filter.update(cx, |filter, cx| {
                filter.reset_silent(window, cx);
            });

            view.update(cx, |this, cx| {
                this.reset_count += 1;
                cx.notify();
            });
        })
        .button_id("reset-filters-story-button")
        .button_style(StyleRefinement::default().font_semibold());

        v_flex()
            .id("reset-filters-story")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(div().text_lg().font_semibold().child("ResetFilters"))
            .child(
                div()
                    .text_sm()
                    .child("Edit any filters, then click reset to clear all values."),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(self.text_filter.clone())
                    .child(self.faceted_filter.clone())
                    .child(self.number_filter.clone())
                    .child(self.date_filter.clone())
                    .child(reset_control),
            )
            .child(
                v_flex()
                    .gap_1()
                    .p_3()
                    .border_1()
                    .rounded(px(8.))
                    .child(div().text_sm().font_semibold().child("Current values"))
                    .child(div().text_sm().child(format!("text: \"{text_value}\"")))
                    .child(div().text_sm().child(format!("faceted: {faceted_display}")))
                    .child(
                        div()
                            .text_sm()
                            .child(format!("number: min={min:?}, max={max:?}")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(format!("date: start={start:?}, end={end:?}")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(format!("reset clicks: {}", self.reset_count)),
                    ),
            )
    }
}
