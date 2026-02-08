//! Calendar-backed date range filter component.

use crate::TableFilterComponent;
use chrono::NaiveDate;
use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable as _,
    button::Button,
    calendar::{Calendar, CalendarEvent, CalendarState, Date},
    divider::Divider,
    popover::Popover,
    v_flex,
};
use std::rc::Rc;

/// A date range filter built on the GPUI calendar control.
///
/// Selecting dates updates the range; changes are applied when the popover closes.
pub struct DateRangeFilter {
    title: Rc<dyn Fn() -> String>,
    selected_range: (Option<NaiveDate>, Option<NaiveDate>),
    /// Value when the popover was opened, used to detect changes
    value_on_open: (Option<NaiveDate>, Option<NaiveDate>),
    calendar: Option<Entity<CalendarState>>,
    on_change: Rc<dyn Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static>,
    _subscriptions: Vec<Subscription>,
}

impl TableFilterComponent for DateRangeFilter {
    type Value = (Option<NaiveDate>, Option<NaiveDate>);

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::DateRange;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move || title.clone()), value, on_change, cx)
    }
}

impl DateRangeFilter {
    fn new_with_title(
        title: Rc<dyn Fn() -> String>,
        value: (Option<NaiveDate>, Option<NaiveDate>),
        on_change: impl Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title,
            selected_range: value,
            value_on_open: value,
            calendar: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
        })
    }

    /// Create a date range filter with a reactive title provider (e.g. for i18n).
    pub fn new_for(
        title: impl Fn() -> String + 'static,
        value: (Option<NaiveDate>, Option<NaiveDate>),
        on_change: impl Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn ensure_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.calendar.is_none() {
            let (start, end) = self.selected_range;
            let calendar = cx.new(|cx| {
                let mut cal = CalendarState::new(window, cx);
                cal.set_date(Date::Range(start, end), window, cx);
                cal
            });

            // Subscribe to calendar selection changes
            let subscription = cx.subscribe_in(
                &calendar,
                window,
                |this: &mut Self, _, event: &CalendarEvent, window, cx| {
                    let CalendarEvent::Selected(date) = event;
                    let (start, end) = match date {
                        Date::Range(start, end) => (*start, *end),
                        Date::Single(date) => (*date, None),
                    };
                    this.selected_range = (start, end);
                    (this.on_change)(this.selected_range, window, cx);
                    cx.notify();
                },
            );

            self._subscriptions.push(subscription);
            self.calendar = Some(calendar);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = (None, None);
        if let Some(calendar) = &self.calendar {
            calendar.update(cx, |cal, cx| {
                cal.set_date(Date::Range(None, None), window, cx);
            });
        }
        (self.on_change)((None, None), window, cx);
        cx.notify();
    }

    fn has_value(&self) -> bool {
        self.selected_range.0.is_some() || self.selected_range.1.is_some()
    }

    /// Record the current value when popover opens.
    fn on_popover_open(&mut self) {
        self.value_on_open = self.selected_range;
    }

    /// Apply the current filter value via callback, only if it changed.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply_if_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range != self.value_on_open {
            (self.on_change)(self.selected_range, window, cx);
        }
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)(self.selected_range, window, cx);
    }

    /// Get the current filter value.
    pub fn value(&self) -> (Option<NaiveDate>, Option<NaiveDate>) {
        self.selected_range
    }

    fn format_range(&self) -> String {
        match self.selected_range {
            (Some(start), Some(end)) => {
                if start == end {
                    // Same date, no range separator needed
                    format_date(start)
                } else {
                    format!("{} - {}", format_date(start), format_date(end))
                }
            },
            (Some(start), None) => format_date(start),
            (None, Some(end)) => format!("... - {}", format_date(end)),
            (None, None) => String::new(),
        }
    }
}

fn format_date(date: NaiveDate) -> String {
    date.format("%b %d, %Y").to_string()
}

impl Render for DateRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure calendar exists
        self.ensure_calendar(window, cx);

        let title = (self.title)();
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity().clone();
        let calendar = self.calendar.clone().unwrap();

        // Icon: CircleX when has value (to clear), Calendar otherwise
        let trigger_icon = if has_value {
            IconName::CircleX
        } else {
            IconName::Calendar
        };

        let clear_view = view.clone();
        let trigger = Button::new("date-range-trigger")
            .outline()
            .child(
                div()
                    .id("clear-icon")
                    .when(has_value, |this| {
                        this.cursor_pointer()
                            .rounded_sm()
                            .hover(|s| s.opacity(1.0))
                            .opacity(0.7)
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                clear_view.update(cx, |this, cx| {
                                    this.clear(window, cx);
                                });
                            })
                    })
                    .child(Icon::new(trigger_icon).xsmall()),
            )
            .child(title.clone())
            .when(has_value, |b| {
                b.child(Divider::vertical().h(px(16.)).mx_1())
                    .child(range_display)
            });

        let apply_view = view.clone();
        Popover::new("date-range-popover")
            .trigger(trigger)
            .on_open_change(move |open, window, cx| {
                apply_view.update(cx, |this, cx| {
                    if *open {
                        // Record the value when popover opens
                        this.on_popover_open();
                    } else {
                        // When popover closes, apply only if value changed
                        this.apply_if_changed(window, cx);
                    }
                });
            })
            .content(move |_, _window, _cx| {
                let clear_view_inner = view.clone();
                v_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        // Use Calendar directly with 2 months shown
                        Calendar::new(&calendar).number_of_months(2).small(),
                    )
                    .when(has_value, |this| {
                        this.child(
                            Button::new("clear-btn")
                                .outline()
                                .small()
                                .w_full()
                                .label("Clear")
                                .on_click(move |_, window, cx| {
                                    clear_view_inner.update(cx, |this, cx| {
                                        this.clear(window, cx);
                                    });
                                }),
                        )
                    })
            })
    }
}
