#[cfg(feature = "db")]
use spacetimedb::Table as _;
#[cfg(feature = "client")]
use spacetimedb_sdk::Table as _;

#[derive(Clone, Debug, Eq, Hash, PartialEq, spacetimedb::SpacetimeType)]
#[cfg_attr(
    feature = "client",
    derive(es_fluent::EsFluent, gpui_table::Filterable, gpui_table::TableCell)
)]
#[cfg_attr(feature = "client", filter(fluent))]
pub enum SpacetimeMutation {
    #[cfg_attr(feature = "client", filter(icon = gpui_component::IconName::ArrowUp))]
    Insert,
    #[cfg_attr(feature = "client", filter(icon = gpui_component::IconName::Settings))]
    Update,
    #[cfg_attr(feature = "client", filter(icon = gpui_component::IconName::CircleX))]
    Delete,
}

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "client",
    derive(
        es_fluent::EsFluentThis,
        es_fluent::EsFluentVariants,
        gpui_table::GpuiTable
    )
)]
#[cfg_attr(feature = "db", spacetimedb::table(accessor = spacetime_event, public))]
#[cfg_attr(feature = "client", fluent_this(origin, variants))]
#[cfg_attr(feature = "client", fluent_variants(keys = ["description", "label"]))]
#[cfg_attr(feature = "client", gpui_table(fluent = "label", filters, load_more))]
pub struct SpacetimeEvent {
    #[cfg_attr(feature = "db", primary_key)]
    #[cfg_attr(feature = "db", auto_inc)]
    #[cfg_attr(feature = "client", gpui_table(skip))]
    pub id: u64,

    #[cfg_attr(feature = "client", gpui_table(sortable, width = 120., filter(text())))]
    pub table_name: String,

    #[cfg_attr(feature = "client", gpui_table(sortable, width = 260.))]
    pub sender: spacetimedb::Identity,

    #[cfg_attr(feature = "client", gpui_table(width = 120.))]
    pub connection_id: Option<spacetimedb::ConnectionId>,

    #[cfg_attr(feature = "client", gpui_table(width = 120., filter(faceted())))]
    pub mutation: SpacetimeMutation,

    #[cfg_attr(
        feature = "client",
        gpui_table(sortable, width = 130., filter(number_range(min = 1., step = 1.)))
    )]
    pub rows_touched: u32,

    #[cfg_attr(
        feature = "client",
        gpui_table(sortable, width = 220., filter(date_range()))
    )]
    pub committed_at: spacetimedb::Timestamp,

    #[cfg_attr(feature = "client", gpui_table(width = 240., filter(text())))]
    pub reducer: String,
}

#[cfg(feature = "db")]
fn seeded_timestamp(now_micros: i64, row: u32, count: u32) -> spacetimedb::Timestamp {
    let spacing_micros: i64 = 30 * 1_000_000;
    let remaining = i64::from(count.saturating_sub(row));
    let delta = remaining.saturating_mul(spacing_micros);
    spacetimedb::Timestamp::from_micros_since_unix_epoch(now_micros.saturating_sub(delta))
}

#[cfg(feature = "db")]
#[spacetimedb::reducer]
pub fn seed_spacetime_events(ctx: &spacetimedb::ReducerContext, count: u32) -> Result<(), String> {
    let table = ctx.db.spacetime_event();
    if count == 0 {
        return Ok(());
    }

    let sender = ctx.sender();
    let connection_id = ctx.connection_id();
    let now_micros = ctx.timestamp.to_micros_since_unix_epoch();
    // Mirrors entities commonly used in SpaceTimeDB docs and quickstart examples.
    let table_names = [
        "player",
        "message",
        "inventory",
        "match_state",
        "leaderboard",
    ];
    let reducers = [
        "set_name",
        "send_message",
        "spawn_player",
        "submit_score",
        "collect_loot",
        "complete_quest",
        "equip_item",
        "respawn_player",
    ];

    for row in 0..count {
        let mutation = match row % 3 {
            0 => SpacetimeMutation::Insert,
            1 => SpacetimeMutation::Update,
            _ => SpacetimeMutation::Delete,
        };
        let rows_touched = match row % 3 {
            0 => 1 + (row % 4),
            1 => 2 + ((row * 3) % 32),
            _ => 1 + ((row * 5) % 10),
        };
        let table_name = table_names[row as usize % table_names.len()];
        let reducer = reducers[row as usize % reducers.len()];

        table
            .try_insert(SpacetimeEvent {
                id: 0,
                table_name: table_name.to_string(),
                sender,
                connection_id,
                mutation,
                rows_touched,
                committed_at: seeded_timestamp(now_micros, row, count),
                reducer: reducer.to_string(),
            })
            .map_err(|error| format!("Failed to insert spacetime_event row {row}: {error}"))?;
    }

    Ok(())
}

#[cfg(feature = "client")]
static SPACETIME_FILTER_STATE: std::sync::OnceLock<std::sync::RwLock<SpacetimeEventFilterValues>> =
    std::sync::OnceLock::new();

#[cfg(feature = "client")]
fn filter_state() -> &'static std::sync::RwLock<SpacetimeEventFilterValues> {
    SPACETIME_FILTER_STATE
        .get_or_init(|| std::sync::RwLock::new(SpacetimeEventFilterValues::default()))
}

#[cfg(feature = "client")]
pub fn set_spacetime_event_table_filters(filters: SpacetimeEventFilterValues) {
    if let Ok(mut state) = filter_state().write() {
        *state = filters;
    }
}

#[cfg(feature = "client")]
fn current_filters() -> SpacetimeEventFilterValues {
    match filter_state().read() {
        Ok(state) => state.clone(),
        Err(_) => SpacetimeEventFilterValues::default(),
    }
}

#[cfg(feature = "client")]
impl From<crate::module_bindings::SpacetimeMutation> for SpacetimeMutation {
    fn from(value: crate::module_bindings::SpacetimeMutation) -> Self {
        match value {
            crate::module_bindings::SpacetimeMutation::Insert => Self::Insert,
            crate::module_bindings::SpacetimeMutation::Update => Self::Update,
            crate::module_bindings::SpacetimeMutation::Delete => Self::Delete,
        }
    }
}

#[cfg(feature = "client")]
impl From<crate::module_bindings::SpacetimeEvent> for SpacetimeEvent {
    fn from(value: crate::module_bindings::SpacetimeEvent) -> Self {
        Self {
            id: value.id,
            table_name: value.table_name,
            sender: value.sender,
            connection_id: value.connection_id,
            mutation: value.mutation.into(),
            rows_touched: value.rows_touched,
            committed_at: value.committed_at,
            reducer: value.reducer,
        }
    }
}

#[cfg(feature = "client")]
fn fetch_page_from_bindings(
    filters: SpacetimeEventFilterValues,
    offset: usize,
    limit: usize,
) -> Result<(Vec<SpacetimeEvent>, usize), String> {
    use crate::module_bindings::spacetime_event_table::SpacetimeEventTableAccess as _;

    let conn = crate::client_connection::get()?;
    let table = conn.db.spacetime_event();

    let mut rows: Vec<SpacetimeEvent> = table.iter().map(Into::into).collect();

    rows.sort_by(|left, right| right.id.cmp(&left.id));

    if gpui_table::filter::FilterValuesExt::has_active_filters(&filters) {
        rows.retain(|row| gpui_table::filter::Matchable::matches_filters(row, &filters));
    }

    let total_count = rows.len();
    let page = rows.into_iter().skip(offset).take(limit).collect();

    Ok((page, total_count))
}

#[cfg(feature = "client")]
#[gpui_table::gpui_table_impl]
impl gpui_table::TableLoader for SpacetimeEventTableDelegate {
    const THRESHOLD: usize = 50;

    fn load_more(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        let offset = self.rows.len();
        let limit = Self::THRESHOLD;
        let filters = current_filters();

        log::debug!(
            "Loading SpacetimeDB rows via generated bindings: offset={}, limit={}",
            offset,
            limit
        );

        cx.spawn(async move |view, cx| {
            let result = fetch_page_from_bindings(filters, offset, limit);

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();

                    match result {
                        Ok((rows, total_count)) => {
                            delegate.rows.extend(rows);

                            delegate.eof = delegate.rows.len() >= total_count;
                            log::info!(
                                "SpacetimeDB page loaded: visible={}, total={}",
                                delegate.rows.len(),
                                total_count
                            );
                        },
                        Err(err) => {
                            log::warn!("SpacetimeDB bindings query failed: {}", err);
                            delegate.eof = true;
                        },
                    }

                    delegate.loading = false;
                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }
}
