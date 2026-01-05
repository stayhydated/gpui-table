use es_fluent::{EsFluentKv, EsFluentThis};
use fake::decimal::PositiveDecimal;
use fake::faker::{chrono::en::DateTime, internet::en::SafeEmail, name::en::Name};
use fake::uuid::UUIDv4;
use fake::{Fake, Faker};
use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
use gpui_table::{Filterable, GpuiTable, TableCell};
use rust_decimal::Decimal;
use std::time::Duration;

#[derive(
    Clone, Debug, Eq, Hash, fake::Dummy, es_fluent::EsFluent, Filterable, PartialEq, TableCell,
)]
#[filter(fluent)]
pub enum UserStatus {
    #[filter(icon = IconName::Check)]
    Active,
    #[filter(icon = IconName::CircleX)]
    Suspended,
    #[filter(icon = IconName::Moon)]
    Offline,
}

#[derive(Clone, fake::Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label", filters)]
#[gpui_table(load_more = "Self::load_more_users")]
#[gpui_table(load_more_threshold = 20)]
pub struct User {
    #[gpui_table(skip)]
    #[dummy(faker = "UUIDv4")]
    #[allow(dead_code)]
    pub id: uuid::Uuid,

    #[gpui_table(sortable, width = 150., filter(text()))]
    #[dummy(faker = "Name()")]
    pub name: String,

    #[gpui_table(sortable, width = 80., filter(number_range()))]
    #[dummy(faker = "18..67")]
    pub age: u8,

    #[gpui_table(sortable, width = 150., filter(number_range()))]
    #[dummy(faker = "PositiveDecimal")]
    pub debt: Decimal,

    #[gpui_table(width = 200., filter(text()))]
    #[dummy(faker = "SafeEmail()")]
    pub email: String,

    #[gpui_table(width = 70., filter(faceted()))]
    pub active: bool,

    #[gpui_table(width = 100., filter(faceted()))]
    pub status: UserStatus,

    #[gpui_table(sortable, width = 300., filter(date_range()))]
    #[dummy(faker = "DateTime()")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl UserTableDelegate {
    /// Load more users with fake data generation.
    pub fn load_more_users(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;

            // Generate fake data
            let new_rows: Vec<User> = (0..50).map(|_| Faker.fake()).collect();

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();
                    delegate.rows.extend(new_rows);
                    delegate.loading = false;

                    // Stop after 500 rows for demo purposes
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
