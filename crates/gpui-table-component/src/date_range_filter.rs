use crate::TableFilterComponent;
use chrono::NaiveDate;
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Subscription, Window, div,
    prelude::*, px,
};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt as _,
    button::Button,
    calendar::{Calendar, CalendarEvent, CalendarState, Date},
    popover::Popover,
    separator::Separator,
    v_flex,
};
use std::borrow::Borrow as _;
use std::rc::Rc;

fn app<'a, T>(cx: &'a Context<'_, T>) -> &'a App {
    cx.borrow()
}

mod date_display {
    use chrono::{Datelike as _, NaiveDate};
    use icu::{
        calendar::{Date, Iso},
        datetime::{DateTimeFormatter, DateTimeFormatterPreferences, fieldsets},
        locale::{Locale, locale},
    };
    use jiff::civil;

    type DateFormatter = DateTimeFormatter<fieldsets::YMD>;

    fn formatter_preferences() -> DateTimeFormatterPreferences {
        let locale = gpui_component::locale()
            .parse::<Locale>()
            .unwrap_or(locale!("en-US"));
        locale.into()
    }

    fn date_formatter() -> Option<DateFormatter> {
        DateTimeFormatter::try_new(formatter_preferences(), fieldsets::YMD::medium()).ok()
    }

    fn chrono_naive_date_to_jiff(value: &NaiveDate) -> Option<civil::Date> {
        let month = i8::try_from(value.month()).ok()?;
        let day = i8::try_from(value.day()).ok()?;
        let year = i16::try_from(value.year()).ok()?;
        civil::Date::new(year, month, day).ok()
    }

    fn to_icu_date(value: civil::Date) -> Option<Date<Iso>> {
        let month = u8::try_from(value.month()).ok()?;
        let day = u8::try_from(value.day()).ok()?;
        Date::try_new_iso(i32::from(value.year()), month, day).ok()
    }

    pub(super) fn format_date(value: NaiveDate) -> String {
        chrono_naive_date_to_jiff(&value)
            .and_then(|value| {
                let date = to_icu_date(value)?;
                let formatter = date_formatter()?;
                Some(formatter.format(&date).to_string())
            })
            .unwrap_or_else(|| value.to_string())
    }
}

pub struct DateRangeFilter {
    title: Rc<dyn Fn(&App) -> String>,
    selected_range: (Option<NaiveDate>, Option<NaiveDate>),
    trigger_style: StyleRefinement,
    popover_style: StyleRefinement,
    calendar_style: StyleRefinement,
    clear_button_style: StyleRefinement,
    calendar: Option<Entity<CalendarState>>,
    on_change: Rc<dyn Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static>,
    _subscriptions: Vec<Subscription>,
}

impl component_shape::ComponentShapeMetadata for DateRangeFilter {
    const MCP_INPUT: component_shape::McpInput = component_shape::McpInput::date_range();
}
impl component_shape::DeclaredComponentShape for DateRangeFilter {}
impl component_shape::ComponentShapeFor<NaiveDate> for DateRangeFilter {}
impl component_shape::ComponentShapeFor<Option<NaiveDate>> for DateRangeFilter {}
impl component_shape::ComponentShapeFor<chrono::NaiveDateTime> for DateRangeFilter {}
impl component_shape::ComponentShapeFor<Option<chrono::NaiveDateTime>> for DateRangeFilter {}

impl<Tz> component_shape::ComponentShapeFor<chrono::DateTime<Tz>> for DateRangeFilter where
    Tz: chrono::TimeZone
{
}

impl<Tz> component_shape::ComponentShapeFor<Option<chrono::DateTime<Tz>>> for DateRangeFilter where
    Tz: chrono::TimeZone
{
}

#[cfg(feature = "spacetimedb")]
impl component_shape::ComponentShapeFor<spacetimedb_lib::Timestamp> for DateRangeFilter {}

#[cfg(feature = "spacetimedb")]
impl component_shape::ComponentShapeFor<Option<spacetimedb_lib::Timestamp>> for DateRangeFilter {}

impl TableFilterComponent for DateRangeFilter {
    type Value = (Option<NaiveDate>, Option<NaiveDate>);

    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType =
        gpui_table_schema::registry::RegistryFilterType::DateRange;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move |_| title.clone()), value, on_change, cx)
    }
}

impl DateRangeFilter {
    /// Create a date range filter with a fixed title.
    pub fn new(
        title: impl Into<String>,
        value: (Option<NaiveDate>, Option<NaiveDate>),
        on_change: impl Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move |_| title.clone()), value, on_change, cx)
    }

    fn new_with_title(
        title: Rc<dyn Fn(&App) -> String>,
        value: (Option<NaiveDate>, Option<NaiveDate>),
        on_change: impl Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_cx| Self {
            title,
            selected_range: value,
            trigger_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            calendar_style: StyleRefinement::default(),
            clear_button_style: StyleRefinement::default(),
            calendar: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
        })
    }

    /// Create a date range filter with a reactive title provider (e.g. for i18n).
    pub fn new_for(
        title: impl Fn(&App) -> String + 'static,
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

    fn reset_inner(&mut self, notify_change: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = (None, None);

        if let Some(calendar) = &self.calendar {
            calendar.update(cx, |cal, cx| {
                cal.set_date(Date::Range(None, None), window, cx);
            });
        }

        if notify_change {
            (self.on_change)((None, None), window, cx);
        }

        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    fn has_value(&self) -> bool {
        self.selected_range.0.is_some() || self.selected_range.1.is_some()
    }

    /// Apply the current filter value via callback.
    /// Call this from parent when you want to trigger the on_change.
    pub fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_change)(self.selected_range, window, cx);
    }

    /// Reset the date range and notify via callback.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset the date range without invoking callback.
    pub fn reset_silent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(false, window, cx);
    }

    /// Replace date bounds without invoking the change callback.
    pub fn set_silent(
        &mut self,
        value: (Option<NaiveDate>, Option<NaiveDate>),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = value;
        if let Some(calendar) = &self.calendar {
            calendar.update(cx, |calendar, cx| {
                calendar.set_date(Date::Range(value.0, value.1), window, cx);
            });
        }
        cx.notify();
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
    date_display::format_date(date)
}

impl Render for DateRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure calendar exists
        self.ensure_calendar(window, cx);

        let title = (self.title)(app(cx));
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity();
        let trigger_style = self.trigger_style.clone();
        let popover_style = self.popover_style.clone();
        let calendar_style = self.calendar_style.clone();
        let clear_button_style = self.clear_button_style.clone();
        let Some(calendar) = self.calendar.clone() else {
            return div().into_any_element();
        };

        // Icon: CircleX when has value (to clear), Calendar otherwise
        let trigger_icon = if has_value {
            IconName::CircleX
        } else {
            IconName::Calendar
        };

        let clear_view = view.clone();
        let trigger = Button::new("date-range-trigger")
            .outline()
            .refine_style(&trigger_style)
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
            .child(title)
            .when(has_value, |b| {
                b.child(Separator::vertical().h(px(16.)).mx_1())
                    .child(range_display)
            });

        Popover::new("date-range-popover")
            .trigger(trigger)
            .content(move |_, _window, _cx| {
                let clear_view_inner = view.clone();
                v_flex()
                    .p_2()
                    .gap_2()
                    .refine_style(&popover_style)
                    .child(
                        // Use Calendar directly with 2 months shown
                        Calendar::new(&calendar)
                            .number_of_months(2)
                            .small()
                            .refine_style(&calendar_style),
                    )
                    .when(has_value, |this| {
                        this.child(
                            Button::new("clear-btn")
                                .outline()
                                .small()
                                .w_full()
                                .label("Clear")
                                .refine_style(&clear_button_style)
                                .on_click(move |_, window, cx| {
                                    clear_view_inner.update(cx, |this, cx| {
                                        this.clear(window, cx);
                                    });
                                }),
                        )
                    })
            })
            .into_any_element()
    }
}

/// Extension trait for chainable configuration on `Entity<DateRangeFilter>`.
pub trait DateRangeFilterExt: Sized {
    /// Set style refinement for the trigger button.
    fn trigger_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the popover root content.
    fn popover_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the calendar.
    fn calendar_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the clear button in the popover.
    fn clear_button_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
}

impl DateRangeFilterExt for Entity<DateRangeFilter> {
    fn trigger_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.trigger_style = style;
        });
        self
    }

    fn popover_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.popover_style = style;
        });
        self
    }

    fn calendar_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.calendar_style = style;
        });
        self
    }

    fn clear_button_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _| {
            this.clear_button_style = style;
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{DateRangeFilter, DateRangeFilterExt as _, format_date};
    use chrono::NaiveDate;
    use gpui::{Empty, StyleRefinement, TestAppContext, VisualTestContext};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn formats_dates_with_icu4x() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).expect("valid date");
        assert_eq!(format_date(date), "Jan 31, 2026");
    }

    #[gpui::test]
    fn range_display_and_configuration_cover_each_bound_shape(cx: &mut TestAppContext) {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();

        let both = cx.update(|cx| {
            DateRangeFilter::new("Created", (Some(start), Some(end)), |_, _, _| {}, cx)
                .trigger_style(StyleRefinement::default(), cx)
                .popover_style(StyleRefinement::default(), cx)
                .calendar_style(StyleRefinement::default(), cx)
                .clear_button_style(StyleRefinement::default(), cx)
        });
        both.read_with(cx, |filter, cx| {
            assert!(filter.has_value());
            assert_eq!(filter.value(), (Some(start), Some(end)));
            assert_eq!(filter.format_range(), "Jan 1, 2026 - Jan 31, 2026");
            assert_eq!((filter.title)(cx), "Created");
            assert!(filter.calendar.is_none());
        });

        let same = cx.update(|cx| {
            DateRangeFilter::new_for(
                |_| "Date".into(),
                (Some(start), Some(start)),
                |_, _, _| {},
                cx,
            )
        });
        same.read_with(cx, |filter, _| {
            assert_eq!(filter.format_range(), "Jan 1, 2026")
        });

        let start_only =
            cx.update(|cx| DateRangeFilter::new("Date", (Some(start), None), |_, _, _| {}, cx));
        start_only.read_with(cx, |filter, _| {
            assert_eq!(filter.format_range(), "Jan 1, 2026")
        });

        let end_only =
            cx.update(|cx| DateRangeFilter::new("Date", (None, Some(end)), |_, _, _| {}, cx));
        end_only.read_with(cx, |filter, _| {
            assert_eq!(filter.format_range(), "... - Jan 31, 2026")
        });

        let empty = cx.update(|cx| DateRangeFilter::new("Date", (None, None), |_, _, _| {}, cx));
        empty.read_with(cx, |filter, _| {
            assert!(!filter.has_value());
            assert_eq!(filter.format_range(), "");
        });
    }

    #[gpui::test]
    fn date_filter_apply_and_reset_paths_use_window_context(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let changes = Rc::new(RefCell::new(Vec::new()));
        let changes_for_callback = changes.clone();
        let filter = cx.update(|cx| {
            DateRangeFilter::new(
                "Created",
                (Some(start), Some(end)),
                move |value, _, _| changes_for_callback.borrow_mut().push(value),
                cx,
            )
        });
        let window = cx.add_window(|_, _| Empty);
        let mut visual = VisualTestContext::from_window(window.into(), cx);

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.apply(window, cx);
                filter.reset(window, cx);
            });
        });
        assert_eq!(
            &*changes.borrow(),
            &[(Some(start), Some(end)), (None, None)]
        );

        visual.update(|window, cx| {
            filter.update(cx, |filter, cx| {
                filter.selected_range = (Some(start), None);
                filter.reset_silent(window, cx);
            });
        });
        assert_eq!(changes.borrow().len(), 2);
        filter.read_with(&visual.cx, |filter, _| {
            assert_eq!(filter.value(), (None, None))
        });
        drop(filter);
        drop(visual);
        cx.run_until_parked();
    }
}
