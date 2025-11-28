use fake::Fake;
use gpui::{
    App, AppContext, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    Sizable as _, Size, h_flex,
    table::{Table, TableDelegate, TableEvent, TableState},
    v_flex,
};
use some_lib::structs::item::{Item, ItemTableDelegate};

#[gpui_storybook::story]
pub struct ItemTableStory {
    table: Entity<TableState<ItemTableDelegate>>,
    stripe: bool,
    size: Size,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for ItemTableStory {
    fn title() -> String {
        Item::this_ftl()
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

        v_flex()
            .size_full()
            .text_sm()
            .gap_4()
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
