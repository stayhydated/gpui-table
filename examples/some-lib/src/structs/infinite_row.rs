use es_fluent::{EsFluentKv, EsFluentThis};
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use fake::Dummy;
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::GpuiTable;

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
    #[gpui_table(sortable, ascending, filter(text()))]
    pub name: String,

    #[dummy(faker = "Sentence(3..6)")]
    #[gpui_table(width = 300., filter(text()))]
    pub description: String,
}

impl InfiniteRowTableDelegate {

    /// Initialize with a pre-generated data pool
    pub fn with_data_pool(all_data: Vec<InfiniteRow>) -> Self {
        let rows = all_data.clone();
        Self {
            rows,
            columns: <InfiniteRow as gpui_table::TableRowMeta>::table_columns(),
            visible_rows: Default::default(),
            visible_cols: Default::default(),
            eof: true, // All data is already loaded
            loading: false,
            full_loading: false,
        }
    }

    /// Apply filters to the data pool and update displayed rows
    pub fn apply_filters(
        &mut self,
        all_data: &[InfiniteRow],
        name_filter: &str,
        description_filter: &str,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        log::info!("Applying filters - name: '{}', description: '{}'", name_filter, description_filter);
        
        let name_filter_lower = name_filter.to_lowercase();
        let description_filter_lower = description_filter.to_lowercase();
        
        self.rows = all_data
            .iter()
            .filter(|row| {
                let name_matches = name_filter.is_empty() 
                    || row.name.to_lowercase().contains(&name_filter_lower);
                let desc_matches = description_filter.is_empty() 
                    || row.description.to_lowercase().contains(&description_filter_lower);
                name_matches && desc_matches
            })
            .cloned()
            .collect();
        
        log::info!("Filtered to {} rows from {} total", self.rows.len(), all_data.len());
        cx.notify();
    }

    /// Load more data with filter values for server-side filtering
    /// (For compatibility - but this example now uses client-side filtering)
    pub fn load_more_with_filters(
        &mut self,
        _name_filter: String,
        _description_filter: String,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        // No-op: all data is pre-loaded, filtering is done client-side
    }

    /// Load more data (without filters - for initial load)
    pub fn load_more_data(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        // No-op: all data is pre-loaded
    }

    /// Reset and reload data with new filter values
    /// (For compatibility - delegates to apply_filters in the story)
    pub fn reset_and_reload_with_filters(
        &mut self,
        _name_filter: String,
        _description_filter: String,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        // This is now handled by the story calling apply_filters directly
        // with access to the data pool
    }
}

