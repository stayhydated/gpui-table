#[cfg(feature = "client")]
use es_fluent::{EsFluentThis, EsFluentVariants};
#[cfg(feature = "client")]
use gpui::{Context, Window};
#[cfg(feature = "client")]
use gpui_component::IconName;
#[cfg(feature = "client")]
use gpui_component::table::TableState;
#[cfg(feature = "client")]
use gpui_table::filter::{FilterValuesExt as _, Matchable as _};
#[cfg(feature = "client")]
use gpui_table::{Filterable, GpuiTable, TableCell, TableLoader};
#[cfg(feature = "client")]
use log::{debug, info, warn};
#[cfg(feature = "client")]
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug, Eq, Hash, PartialEq, spacetimedb::SpacetimeType)]
#[cfg_attr(feature = "client", derive(es_fluent::EsFluent, Filterable, TableCell))]
#[cfg_attr(feature = "client", filter(fluent))]
pub enum SpacetimeMutation {
    #[cfg_attr(feature = "client", filter(icon = IconName::ArrowUp))]
    Insert,
    #[cfg_attr(feature = "client", filter(icon = IconName::Settings))]
    Update,
    #[cfg_attr(feature = "client", filter(icon = IconName::CircleX))]
    Delete,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "client", derive(EsFluentThis, EsFluentVariants, GpuiTable))]
#[cfg_attr(not(feature = "db"), derive(spacetimedb::SpacetimeType))]
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
        gpui_table(sortable, width = 220., filter(date_range()))
    )]
    pub committed_at: spacetimedb::Timestamp,

    #[cfg_attr(feature = "client", gpui_table(width = 240., filter(text())))]
    pub reducer: String,
}

#[cfg(feature = "client")]
static SPACETIME_FILTER_STATE: OnceLock<RwLock<SpacetimeEventFilterValues>> = OnceLock::new();

#[cfg(feature = "client")]
fn filter_state() -> &'static RwLock<SpacetimeEventFilterValues> {
    SPACETIME_FILTER_STATE.get_or_init(|| RwLock::new(SpacetimeEventFilterValues::default()))
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
        use crate::module_bindings::SpacetimeMutation as ModuleMutation;

        match value {
            ModuleMutation::Insert => Self::Insert,
            ModuleMutation::Update => Self::Update,
            ModuleMutation::Delete => Self::Delete,
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
    use spacetimedb_sdk::Table as _;

    let conn = crate::client_connection::get()?;

    let mut rows: Vec<SpacetimeEvent> = conn
        .db
        .spacetime_event()
        .iter()
        .map(|row| row.clone())
        .map(Into::into)
        .collect();

    rows.sort_by(|left, right| right.id.cmp(&left.id));

    if filters.has_active_filters() {
        rows.retain(|row| row.matches_filters(&filters));
    }

    let total_count = rows.len();
    let page = rows.into_iter().skip(offset).take(limit).collect();

    Ok((page, total_count))
}

#[cfg(feature = "client")]
#[gpui_table::gpui_table_impl]
impl TableLoader for SpacetimeEventTableDelegate {
    const THRESHOLD: usize = 50;

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        let offset = self.rows.len();
        let limit = Self::THRESHOLD;
        let filters = current_filters();

        debug!(
            "Loading SpaceTimeDB rows via generated bindings: offset={}, limit={}",
            offset, limit
        );

        cx.spawn(async move |view, cx| {
            let result = fetch_page_from_bindings(filters, offset, limit);

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();

                    match result {
                        Ok((rows, total_count)) => {
                            let mut existing_ids = delegate
                                .rows
                                .iter()
                                .map(|row| row.id)
                                .collect::<std::collections::HashSet<_>>();

                            for row in rows {
                                if existing_ids.insert(row.id) {
                                    delegate.rows.push(row);
                                }
                            }

                            delegate.eof = delegate.rows.len() >= total_count;
                            info!(
                                "SpaceTimeDB page loaded: visible={}, total={}",
                                delegate.rows.len(),
                                total_count
                            );
                        },
                        Err(err) => {
                            warn!("SpaceTimeDB bindings query failed: {}", err);
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
