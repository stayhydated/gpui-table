use es_fluent::EsFluentKv;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use fake::{Dummy, Fake, Faker};
use gpui::{AsyncWindowContext, Context, Window};
use gpui_component::table::TableState;
use gpui_table::NamedTableRow;
use std::time::Duration;

#[derive(Clone, Debug, Dummy, NamedTableRow, EsFluentKv)]
#[fluent_kv(this, keys = ["description", "label"])]
#[table(load_more = "Self::load_more_data")]
#[table(fluent = "label")]
pub struct InfiniteRow {
    #[dummy(faker = "1..10000")]
    #[table(width = 80.)]
    pub id: u64,

    #[dummy(faker = "Name()")]
    #[table(sortable)]
    pub name: String,

    #[dummy(faker = "Sentence(3..6)")]
    #[table(width = 300.)]
    pub description: String,
}

impl InfiniteRowTableDelegate {
    pub fn load_more_data(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;

            // Generate fake data
            let new_rows: Vec<InfiniteRow> = (0..20).map(|_| Faker.fake()).collect();

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.rows.extend(new_rows);
                    delegate.loading = false;

                    // Stop after 500 rows
                    if delegate.rows.len() >= 500 {
                        delegate.eof = true;
                    }

                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }
}
