use std::time;

use super::{ChangeSize, OpenDetail};
use fake::Fake;
use gpui::{
    Action, App, AppContext, Context, Entity, Focusable, InteractiveElement, ParentElement, Render,
    Styled, Subscription, Task, Timer, Window, prelude::FluentBuilder as _,
};
use gpui_component::{
    Selectable, Sizable as _, Size,
    button::Button,
    checkbox::Checkbox,
    h_flex,
    menu::DropdownMenu,
    table::{Table, TableDelegate, TableEvent, TableState},
    v_flex,
};
use serde::Deserialize;
use some_lib::structs::item::{Item, ItemFtl, ItemTableDelegate};

#[gpui_storybook::story]
pub struct ItemTableStory {
    table: Entity<TableState<ItemTableDelegate>>,
    stripe: bool,
    size: Size,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for ItemTableStory {
    fn title() -> String {
        ItemFtl::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }

    fn closable() -> bool {
        false
    }
}

impl Focusable for ItemTableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl ItemTableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut delegate = ItemTableDelegate::new(vec![]);
        for _ in 0..100 {
            delegate.rows.push(fake::Faker.fake());
        }

        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        let _subscriptions = vec![cx.subscribe_in(&table, window, Self::on_table_event)];

        Self {
            table,
            stripe: false,
            size: Size::default(),
            _subscriptions,
        }
    }

    fn toggle_stripe(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        self.stripe = *checked;
        cx.notify();
    }

    fn on_change_size(&mut self, a: &ChangeSize, _: &mut Window, cx: &mut Context<Self>) {
        self.size = a.0;
        cx.notify();
    }

    fn on_table_event(
        &mut self,
        _: &Entity<TableState<ItemTableDelegate>>,
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

impl Render for ItemTableStory {
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
                h_flex().items_center().gap_3().flex_wrap().child(
                    Checkbox::new("stripe")
                        .label("Stripe")
                        .selected(self.stripe)
                        .on_click(cx.listener(Self::toggle_stripe)),
                ),
            )
            .child(
                h_flex().gap_2().child(
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
                                .child(format!("Total Rows: {}", rows_count)),
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
