use std::collections::HashSet;

use chrono::NaiveDate;
use es_fluent::{ThisFtl as _, ToFluentString};
use fake::Fake;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use gpui_table::components::{DateRangeFilter, FacetedFilter, NumberRangeFilter, TextFilter};
use gpui_table_components::TableFilterComponent;
use rust_decimal::Decimal;
use some_lib::structs::filter_showcase::{
    Category, FilterShowcase, FilterShowcaseLabelKvFtl, FilterShowcaseTableDelegate, Priority,
};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct FilterShowcaseStory {
    table: Entity<TableState<FilterShowcaseTableDelegate>>,
    /// Original unfiltered data
    all_rows: Vec<FilterShowcase>,

    // TextFilter instances
    filter_name: Entity<TextFilter>,
    filter_email: Entity<TextFilter>,
    filter_company: Entity<TextFilter>,
    filter_description: Entity<TextFilter>,

    // NumberRangeFilter instances
    filter_age: Entity<NumberRangeFilter>,
    filter_score: Entity<NumberRangeFilter>,
    filter_amount: Entity<NumberRangeFilter>,
    filter_quantity: Entity<NumberRangeFilter>,

    // FacetedFilter instances
    filter_active: Entity<FacetedFilter>,
    filter_verified: Entity<FacetedFilter>,
    filter_priority: Entity<FacetedFilter>,
    filter_category: Entity<FacetedFilter>,

    // DateRangeFilter instances
    filter_created_at: Entity<DateRangeFilter>,
    filter_updated_at: Entity<DateRangeFilter>,
    filter_due_date: Entity<DateRangeFilter>,

    _subscription: Subscription,
}

impl gpui_storybook::Story for FilterShowcaseStory {
    fn title() -> String {
        FilterShowcase::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for FilterShowcaseStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

/// Check if a text field matches a filter (case-insensitive contains)
fn text_matches(value: &str, filter: &str) -> bool {
    filter.is_empty() || value.to_lowercase().contains(&filter.to_lowercase())
}

/// Check if a number is within a range filter
fn number_in_range<T: Into<f64> + Copy>(value: T, range: &(Option<f64>, Option<f64>)) -> bool {
    let v: f64 = value.into();
    match range {
        (None, None) => true,
        (Some(min), None) => v >= *min,
        (None, Some(max)) => v <= *max,
        (Some(min), Some(max)) => v >= *min && v <= *max,
    }
}

/// Check if a Decimal is within a range filter
fn decimal_in_range(value: &Decimal, range: &(Option<f64>, Option<f64>)) -> bool {
    let v: f64 = value.to_string().parse().unwrap_or(0.0);
    match range {
        (None, None) => true,
        (Some(min), None) => v >= *min,
        (None, Some(max)) => v <= *max,
        (Some(min), Some(max)) => v >= *min && v <= *max,
    }
}

/// Check if a faceted value matches the filter set (empty set = no filter)
fn facet_matches(value: &str, filter: &HashSet<String>) -> bool {
    filter.is_empty() || filter.contains(value)
}

/// Check if a date is within a date range filter
fn date_in_range(
    value: &chrono::DateTime<chrono::Utc>,
    range: &(Option<NaiveDate>, Option<NaiveDate>),
) -> bool {
    let date = value.date_naive();
    match range {
        (None, None) => true,
        (Some(start), None) => date >= *start,
        (None, Some(end)) => date <= *end,
        (Some(start), Some(end)) => date >= *start && date <= *end,
    }
}

impl FilterShowcaseStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    /// Apply all filters and update the table rows
    fn apply_filters(&mut self, cx: &mut Context<Self>) {
        self.table.update(cx, |table, _cx| {
            let filters = &table.delegate().filters;

            let filtered: Vec<FilterShowcase> = self
                .all_rows
                .iter()
                .filter(|row| {
                    // Text filters
                    text_matches(&row.name, &filters.name)
                        && text_matches(&row.email, &filters.email)
                        && text_matches(&row.company, &filters.company)
                        && text_matches(&row.description, &filters.description)
                        // Number range filters
                        && number_in_range(row.age, &filters.age)
                        && number_in_range(row.score, &filters.score)
                        && decimal_in_range(&row.amount, &filters.amount)
                        && number_in_range(row.quantity, &filters.quantity)
                        // Faceted filters (bool)
                        && facet_matches(&row.active.to_string(), &filters.active)
                        && facet_matches(&row.verified.to_string(), &filters.verified)
                        // Faceted filters (enum) - use variant name
                        && facet_matches(&format!("{:?}", row.priority), &filters.priority)
                        && facet_matches(&format!("{:?}", row.category), &filters.category)
                        // Date range filters
                        && date_in_range(&row.created_at, &filters.created_at)
                        && date_in_range(&row.updated_at, &filters.updated_at)
                        && date_in_range(&row.due_date, &filters.due_date)
                })
                .cloned()
                .collect();

            table.delegate_mut().rows = filtered;
        });
        cx.notify();
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Generate 200 rows of sample data
        let all_rows: Vec<FilterShowcase> = (0..200).map(|_| fake::Faker.fake()).collect();

        let delegate = FilterShowcaseTableDelegate::new(all_rows.clone());
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // TextFilter: name
        let table_entity = table.clone();
        let filter_name = TextFilter::build(
            "Name",
            String::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.name = value;
                    cx.notify();
                });
            },
            cx,
        );

        // TextFilter: email
        let table_entity = table.clone();
        let filter_email = TextFilter::build(
            "Email",
            String::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.email = value;
                    cx.notify();
                });
            },
            cx,
        );

        // TextFilter: company
        let table_entity = table.clone();
        let filter_company = TextFilter::build(
            "Company",
            String::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.company = value;
                    cx.notify();
                });
            },
            cx,
        );

        // TextFilter: description
        let table_entity = table.clone();
        let filter_description = TextFilter::build(
            "Description",
            String::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.description = value;
                    cx.notify();
                });
            },
            cx,
        );

        // NumberRangeFilter: age
        let table_entity = table.clone();
        let filter_age = NumberRangeFilter::build(
            "Age",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.age = value;
                    cx.notify();
                });
            },
            cx,
        );

        // NumberRangeFilter: score
        let table_entity = table.clone();
        let filter_score = NumberRangeFilter::build(
            "Score",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.score = value;
                    cx.notify();
                });
            },
            cx,
        );

        // NumberRangeFilter: amount
        let table_entity = table.clone();
        let filter_amount = NumberRangeFilter::build(
            "Amount",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.amount = value;
                    cx.notify();
                });
            },
            cx,
        );

        // NumberRangeFilter: quantity
        let table_entity = table.clone();
        let filter_quantity = NumberRangeFilter::build(
            "Quantity",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.quantity = value;
                    cx.notify();
                });
            },
            cx,
        );

        // FacetedFilter: active (boolean)
        let table_entity = table.clone();
        let filter_active = FacetedFilter::build_for::<bool>(
            || "Active".to_string(),
            HashSet::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.active = value;
                    cx.notify();
                });
            },
            cx,
        );

        // FacetedFilter: verified (boolean)
        let table_entity = table.clone();
        let filter_verified = FacetedFilter::build_for::<bool>(
            || "Verified".to_string(),
            HashSet::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.verified = value;
                    cx.notify();
                });
            },
            cx,
        );

        // FacetedFilter: priority
        let table_entity = table.clone();
        let filter_priority = FacetedFilter::build_for::<Priority>(
            || FilterShowcaseLabelKvFtl::Priority.to_fluent_string(),
            HashSet::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.priority = value;
                    cx.notify();
                });
            },
            cx,
        );

        // FacetedFilter: category
        let table_entity = table.clone();
        let filter_category = FacetedFilter::build_for::<Category>(
            || FilterShowcaseLabelKvFtl::Category.to_fluent_string(),
            HashSet::new(),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.category = value;
                    cx.notify();
                });
            },
            cx,
        );

        // DateRangeFilter: created_at
        let table_entity = table.clone();
        let filter_created_at = DateRangeFilter::build(
            "Created At",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.created_at = value;
                    cx.notify();
                });
            },
            cx,
        );

        // DateRangeFilter: updated_at
        let table_entity = table.clone();
        let filter_updated_at = DateRangeFilter::build(
            "Updated At",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.updated_at = value;
                    cx.notify();
                });
            },
            cx,
        );

        // DateRangeFilter: due_date
        let table_entity = table.clone();
        let filter_due_date = DateRangeFilter::build(
            "Due Date",
            (None, None),
            move |value, _window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.due_date = value;
                    cx.notify();
                });
            },
            cx,
        );

        let _subscription = cx.observe(&table, |this, _, cx| {
            this.apply_filters(cx);
        });

        Self {
            table,
            all_rows,
            filter_name,
            filter_email,
            filter_company,
            filter_description,
            filter_age,
            filter_score,
            filter_amount,
            filter_quantity,
            filter_active,
            filter_verified,
            filter_priority,
            filter_category,
            filter_created_at,
            filter_updated_at,
            filter_due_date,
            _subscription,
        }
    }
}

impl Render for FilterShowcaseStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            // Text Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Text Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_name.clone())
                            .child(self.filter_email.clone())
                            .child(self.filter_company.clone())
                            .child(self.filter_description.clone()),
                    ),
            )
            // Number Range Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Number Range Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_age.clone())
                            .child(self.filter_score.clone())
                            .child(self.filter_amount.clone())
                            .child(self.filter_quantity.clone()),
                    ),
            )
            // Faceted Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Faceted Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_active.clone())
                            .child(self.filter_verified.clone())
                            .child(self.filter_priority.clone())
                            .child(self.filter_category.clone()),
                    ),
            )
            // Date Range Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Date Range Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_created_at.clone())
                            .child(self.filter_updated_at.clone())
                            .child(self.filter_due_date.clone()),
                    ),
            )
            // Status bar
            .child(
                h_flex()
                    .gap_4()
                    .child(format!(
                        "Showing: {} / {} rows",
                        delegate.rows.len(),
                        self.all_rows.len()
                    ))
                    .child(if delegate.eof {
                        "All data loaded"
                    } else {
                        "More data available"
                    }),
            )
            // Table
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
