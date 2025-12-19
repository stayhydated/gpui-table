use chrono::NaiveDate;
use gpui::{prelude::*, App, Context, Entity, IntoElement, Render, Window};
use gpui_component::{button::Button, label::Label, popover::Popover, v_flex, IconName};
use std::rc::Rc;

pub struct DateRangeFilter {
    title: String,
    selected_range: (Option<NaiveDate>, Option<NaiveDate>),
    #[allow(dead_code)]
    on_change: Rc<dyn Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static>,
}

impl DateRangeFilter {
    pub fn build(
        title: impl Into<String>,
        selected_range: (Option<NaiveDate>, Option<NaiveDate>),
        on_change: impl Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        let on_change = Rc::new(on_change);

        cx.new(|_cx| Self {
            title,
            selected_range,
            on_change,
        })
    }
}

impl Render for DateRangeFilter {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.title.clone();
        let label = match self.selected_range {
            (Some(start), Some(end)) => format!("{} - {}", start, end),
            (Some(start), None) => format!("{} - ...", start),
            (None, Some(end)) => format!("... - {}", end),
            (None, None) => title.clone(),
        };

        Popover::new("date-filter-popover")
            .trigger(
                Button::new("date-trigger")
                    .label(label)
                    .icon(IconName::Calendar),
            )
            .content(move |_, _, _| v_flex().p_2().child(Label::new("Date Picker Placeholder")))
    }
}
