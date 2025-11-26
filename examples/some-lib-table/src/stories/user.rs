use std::{
    ops::Range,
    sync::LazyLock,
    time::{self, Duration},
};

use fake::Fake;
use gpui::{
    Action, AnyElement, App, AppContext, ClickEvent, Context, Div, Entity, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Subscription, Task, TextAlign, Timer, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, Selectable, Sizable as _, Size, StyleSized as _, StyledExt,
    button::Button,
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    menu::{DropdownMenu, PopupMenu},
    spinner::Spinner,
    table::{Column, ColumnFixed, ColumnSort, Table, TableDelegate, TableEvent, TableState},
    v_flex,
};
use serde::{Deserialize, Serialize};
use some_lib::user::{User, UserTableDelegate};

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = table_story, no_json)]
struct ChangeSize(Size);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = table_story, no_json)]
struct OpenDetail(usize);

pub struct UserTableStory {
    table: Entity<TableState<UserTableDelegate>>,
    num_stocks_input: Entity<InputState>,
    stripe: bool,
    refresh_data: bool,
    size: Size,

    _subscriptions: Vec<Subscription>,
    _load_task: Task<()>,
}

impl gpui_storybook::Story for UserTableStory {
    fn title() -> String {
        User::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }

    fn closable() -> bool {
        false
    }
}

impl Focusable for UserTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl UserTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {}

    fn toggle_loop_selection(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.loop_selection = *checked;
            cx.notify();
        });
    }

    fn toggle_col_resize(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.col_resizable = *checked;
            cx.notify();
        });
    }

    fn toggle_col_order(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.col_movable = *checked;
            cx.notify();
        });
    }

    fn toggle_col_sort(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.sortable = *checked;
            cx.notify();
        });
    }

    fn toggle_col_fixed(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.col_fixed = *checked;
            cx.notify();
        });
    }

    fn toggle_col_selection(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            table.col_selectable = *checked;
            cx.notify();
        });
    }

    fn toggle_stripe(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.stripe = *checked;
        cx.notify();
    }

    fn on_change_size(&mut self, a: &ChangeSize, _: &mut Window, cx: &mut Context<Self>) {
        self.size = a.0;
        cx.notify();
    }

    fn toggle_refresh_data(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.refresh_data = *checked;
        cx.notify();
    }

    fn on_table_event(
        &mut self,
        _: &Entity<TableState<UserTableDelegate>>,
        event: &TableEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::ColumnWidthsChanged(col_widths) => {
                println!("Column widths changed: {:?}", col_widths)
            },
            TableEvent::SelectColumn(ix) => println!("Select col: {}", ix),
            TableEvent::DoubleClickedRow(ix) => println!("Double clicked row: {}", ix),
            TableEvent::SelectRow(ix) => println!("Select row: {}", ix),
            TableEvent::MoveColumn(origin_idx, target_idx) => {
                println!("Move col index: {} -> {}", origin_idx, target_idx);
            },
        }
    }
}

impl Render for UserTableStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = &self.table.read(cx);
        let delegate = table.delegate();
        let rows_count = delegate.rows_count(cx);
        let size = self.size;

        v_flex()
            .on_action(cx.listener(Self::on_change_size))
            .size_full()
            .text_sm()
            .gap_4()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    .child(
                        Checkbox::new("loop-selection")
                            .label("Loop Selection")
                            .selected(table.loop_selection)
                            .on_click(cx.listener(Self::toggle_loop_selection)),
                    )
                    .child(
                        Checkbox::new("col-resize")
                            .label("Column Resize")
                            .selected(table.col_resizable)
                            .on_click(cx.listener(Self::toggle_col_resize)),
                    )
                    .child(
                        Checkbox::new("col-order")
                            .label("Column Order")
                            .selected(table.col_movable)
                            .on_click(cx.listener(Self::toggle_col_order)),
                    )
                    .child(
                        Checkbox::new("col-sort")
                            .label("Sortable")
                            .selected(table.sortable)
                            .on_click(cx.listener(Self::toggle_col_sort)),
                    )
                    .child(
                        Checkbox::new("col-selection")
                            .label("Column Selectable")
                            .selected(table.col_selectable)
                            .on_click(cx.listener(Self::toggle_col_selection)),
                    )
                    .child(
                        Checkbox::new("fixed")
                            .label("Column Fixed")
                            .selected(table.col_fixed)
                            .on_click(cx.listener(Self::toggle_col_fixed)),
                    )
                    .child(
                        Checkbox::new("stripe")
                            .label("Stripe")
                            .selected(self.stripe)
                            .on_click(cx.listener(Self::toggle_stripe)),
                    )
                    .child(
                        Checkbox::new("loading")
                            .label("Loading")
                            .checked(self.table.read(cx).delegate().full_loading)
                            .on_click(cx.listener(|this, check: &bool, _, cx| {
                                this.table.update(cx, |this, cx| {
                                    this.delegate_mut().full_loading = *check;
                                    cx.notify();
                                })
                            })),
                    )
                    .child(
                        Checkbox::new("refresh-data")
                            .label("Refresh Data")
                            .selected(self.refresh_data)
                            .on_click(cx.listener(Self::toggle_refresh_data)),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("size")
                            .outline()
                            .small()
                            .label(format!("size: {:?}", self.size))
                            .dropdown_menu(move |menu, _, _| {
                                menu.menu_with_check(
                                    "Large",
                                    size == Size::Large,
                                    Box::new(ChangeSize(Size::Large)),
                                )
                                .menu_with_check(
                                    "Medium",
                                    size == Size::Medium,
                                    Box::new(ChangeSize(Size::Medium)),
                                )
                                .menu_with_check(
                                    "Small",
                                    size == Size::Small,
                                    Box::new(ChangeSize(Size::Small)),
                                )
                                .menu_with_check(
                                    "XSmall",
                                    size == Size::XSmall,
                                    Box::new(ChangeSize(Size::XSmall)),
                                )
                            }),
                    )
                    .child(
                        Button::new("scroll-top")
                            .outline()
                            .small()
                            .child("Scroll to Top")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.table.update(cx, |table, cx| {
                                    table.scroll_to_row(0, cx);
                                })
                            })),
                    )
                    .child(
                        Button::new("scroll-bottom")
                            .outline()
                            .small()
                            .child("Scroll to Bottom")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.table.update(cx, |table, cx| {
                                    table.scroll_to_row(table.delegate().rows_count(cx) - 1, cx);
                                })
                            })),
                    ),
            )
            .child(
                h_flex().items_center().gap_2().child(
                    h_flex()
                        .w_full()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            h_flex()
                                .gap_2()
                                .flex_1()
                                .child(Label::new("Number of Stocks:"))
                                .child(
                                    h_flex()
                                        .min_w_32()
                                        .child(Input::new(&self.num_stocks_input).small())
                                        .into_any_element(),
                                )
                                .when(delegate.loading, |this| {
                                    this.child(
                                        h_flex().gap_1().child(Spinner::new()).child("Loading..."),
                                    )
                                }),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(format!("Total Rows: {}", rows_count))
                                .child(format!("Visible Rows: {:?}", delegate.visible_rows))
                                .child(format!("Visible Cols: {:?}", delegate.visible_cols))
                                .when(delegate.eof, |this| this.child("All data loaded.")),
                        ),
                ),
            )
            .child(
                Table::new(&self.table)
                    .with_size(self.size)
                    .stripe(self.stripe),
            )
    }
}
