use crate::TableFilterComponent;
use chrono::NaiveDate;
use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    calendar::Date,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider,
    h_flex,
    popover::Popover,
    v_flex,
};
use std::rc::Rc;

pub struct DateRangeFilter {
    title: String,
    selected_range: (Option<NaiveDate>, Option<NaiveDate>),
    date_picker: Option<Entity<DatePickerState>>,
    on_change: Rc<dyn Fn((Option<NaiveDate>, Option<NaiveDate>), &mut Window, &mut App) + 'static>,
    _subscriptions: Vec<Subscription>,
}

impl TableFilterComponent for DateRangeFilter {
    type Value = (Option<NaiveDate>, Option<NaiveDate>);

    const FILTER_TYPE: gpui_table_core::registry::RegistryFilterType =
        gpui_table_core::registry::RegistryFilterType::DateRange;

    fn build(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|_cx| Self {
            title,
            selected_range: value,
            date_picker: None,
            on_change: Rc::new(on_change),
            _subscriptions: Vec::new(),
        })
    }
}

impl DateRangeFilter {
    fn ensure_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.date_picker.is_none() {
            let (start, end) = self.selected_range;
            let picker = cx.new(|cx| {
                let mut picker = DatePickerState::range(window, cx);
                if start.is_some() || end.is_some() {
                    picker.set_date(Date::Range(start, end), window, cx);
                }
                picker
            });

            // Subscribe to date picker changes for immediate feedback
            let subscription = cx.subscribe(
                &picker,
                |this: &mut Self, _, event: &DatePickerEvent, cx| {
                    let DatePickerEvent::Change(date) = event;
                    let (start, end) = match date {
                        Date::Range(start, end) => (*start, *end),
                        Date::Single(date) => (*date, None),
                    };
                    this.selected_range = (start, end);
                    cx.notify();
                },
            );

            self._subscriptions.push(subscription);
            self.date_picker = Some(picker);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = (None, None);
        if let Some(picker) = &self.date_picker {
            picker.update(cx, |picker, cx| {
                picker.set_date(Date::Range(None, None), window, cx);
            });
        }
        (self.on_change)((None, None), window, cx);
        cx.notify();
    }

    fn has_value(&self) -> bool {
        self.selected_range.0.is_some() || self.selected_range.1.is_some()
    }

    fn format_range(&self) -> String {
        match self.selected_range {
            (Some(start), Some(end)) => {
                format!("{} - {}", format_date(start), format_date(end))
            },
            (Some(start), None) => format_date(start),
            (None, Some(end)) => format!("... - {}", format_date(end)),
            (None, None) => String::new(),
        }
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (start, end) = self.selected_range;
        (self.on_change)((start, end), window, cx);
        cx.notify();
    }
}

fn format_date(date: NaiveDate) -> String {
    date.format("%b %d, %Y").to_string()
}

impl Render for DateRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure date picker exists
        self.ensure_date_picker(window, cx);

        let title = self.title.clone();
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity().clone();
        let date_picker = self.date_picker.clone().unwrap();

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

        Popover::new("date-range-popover")
            .trigger(trigger)
            .content(move |_, _window, cx| {
                let apply_view = view.clone();
                let clear_view_inner = view.clone();
                v_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(title.clone()),
                    )
                    .child(
                        // Use DatePicker with appearance(false) for inline calendar display
                        // with 2 months shown side by side
                        div().w_full().bg(cx.theme().secondary).rounded_md().child(
                            DatePicker::new(&date_picker)
                                .number_of_months(2)
                                .appearance(false)
                                .cleanable(true),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("clear-btn")
                                    .outline()
                                    .small()
                                    .flex_1()
                                    .label("Clear")
                                    .on_click(move |_, window, cx| {
                                        clear_view_inner.update(cx, |this, cx| {
                                            this.clear(window, cx);
                                        });
                                    }),
                            )
                            .child(
                                Button::new("apply-btn")
                                    .primary()
                                    .small()
                                    .flex_1()
                                    .label("Apply")
                                    .on_click(move |_, window, cx| {
                                        apply_view.update(cx, |this, cx| {
                                            this.apply(window, cx);
                                        });
                                    }),
                            ),
                    )
            })
    }
}
