use es_fluent::{EsFluentKv, EsFluentThis};
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use fake::{Dummy, Fake, Faker};
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::GpuiTable;
use std::time::Duration;

#[derive(Clone, Debug, Dummy, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
#[gpui_table(load_more = "Self::load_more_data")]
#[gpui_table(load_more_threshold = 30)]
pub struct InfiniteRow {
    #[dummy(faker = "1..10000")]
    #[gpui_table(width = 80., resizable = false, movable = false)]
    pub id: u64,

    #[dummy(faker = "Name()")]
    // Use the component type directly - no strings!
    #[gpui_table(sortable, ascending, filter(text()))]
    pub name: String,

    #[dummy(faker = "Sentence(3..6)")]
    // Both short form (TextFilter) and full path work
    #[gpui_table(width = 300., filter(text()))]
    pub description: String,
}

impl InfiniteRowTableDelegate {
    pub fn load_more_data(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        // Type-safe access to filter values via the filters struct
        let name_filter = self.filters.name.clone();
        let description_filter = self.filters.description.clone();

        // Log active filters (in a real app, these would be sent to an API)
        if !name_filter.is_empty() {
            println!("Fetching with name filter: {}", name_filter);
        }
        if !description_filter.is_empty() {
            println!("Fetching with description filter: {}", description_filter);
        }

        self.loading = true;
        cx.notify();

        cx.spawn(async move |view, cx| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;

            // Generate fake data - in a real app, this would be an API call
            // that includes the filter values as query parameters
            let new_rows: Vec<InfiniteRow> = (0..50)
                .map(|_| Faker.fake())
                .filter(|row: &InfiniteRow| {
                    // Apply client-side filtering for demo purposes
                    // In production, filtering would happen server-side
                    let name_match = name_filter.is_empty()
                        || row
                            .name
                            .to_lowercase()
                            .contains(&name_filter.to_lowercase());
                    let desc_match = description_filter.is_empty()
                        || row
                            .description
                            .to_lowercase()
                            .contains(&description_filter.to_lowercase());
                    name_match && desc_match
                })
                .collect();

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
