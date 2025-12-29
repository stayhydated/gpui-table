//! SpacetimeDB Example - Player Management

use es_fluent::{EsFluentKv, EsFluentThis};
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::GpuiTable;
use gpui_tokio::Tokio;
use log::info;
use spacetimedb::{Identity, Timestamp, table};
use std::sync::atomic::{AtomicU64, Ordering};

// Use enums from the local enums module
use super::enums::{Guild, PlayerStatus};

// ============================================================================
// SpacetimeDB Table + GPUI Table Definition (combined)
// ============================================================================

/// Player table - public so all clients can subscribe, also used for gpui-table display
#[derive(Clone, Debug, EsFluentKv, EsFluentThis, GpuiTable)]
#[table(name = player, public)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
#[gpui_table(load_more = "Self::load_more")]
#[gpui_table(load_more_threshold = 20)]
pub struct SpacetimedbPlayer {
    #[primary_key]
    #[auto_inc]
    #[gpui_table(sortable, width = 60., resizable = false, movable = false)]
    pub id: u64,

    #[unique]
    #[gpui_table(skip)]
    pub identity: Identity,

    #[unique]
    #[gpui_table(sortable, width = 150., filter(text()))]
    pub username: String,

    #[gpui_table(sortable, width = 70., filter(number_range(min = 1., max = 100.)))]
    pub level: u32,

    #[gpui_table(sortable, width = 100.)]
    pub experience: u64,

    #[gpui_table(width = 100., filter(faceted()))]
    pub guild: Guild,

    #[gpui_table(width = 90., filter(faceted()))]
    pub status: PlayerStatus,

    #[gpui_table(sortable, width = 80.)]
    pub score: u32,

    #[gpui_table(sortable, width = 80.)]
    pub games_played: u32,

    #[gpui_table(sortable, width = 80.)]
    pub win_rate: f32,

    #[gpui_table(skip)]
    pub created_at: Timestamp,

    #[gpui_table(skip)]
    pub last_seen: Timestamp,
}

// ============================================================================
// Mock data for demo (replace with real SpacetimeDB subscription)
// ============================================================================

static PLAYER_OFFSET: AtomicU64 = AtomicU64::new(0);
static PLAYER_SEED: AtomicU64 = AtomicU64::new(42);

fn generate_mock_players(offset: u64, count: usize, seed: u64) -> Vec<SpacetimedbPlayer> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let guilds = [
        Guild::None,
        Guild::Warriors,
        Guild::Mages,
        Guild::Defenders,
        Guild::Rogues,
        Guild::Healers,
    ];
    let statuses = [
        PlayerStatus::Online,
        PlayerStatus::Away,
        PlayerStatus::Offline,
        PlayerStatus::InGame,
    ];
    let name_prefixes = [
        "Shadow", "Dragon", "Storm", "Fire", "Ice", "Thunder", "Dark", "Light", "Swift", "Iron",
    ];
    let name_suffixes = [
        "Knight", "Mage", "Hunter", "Warrior", "Slayer", "Master", "Lord", "King", "Blade", "Heart",
    ];

    (0..count)
        .map(|i| {
            let id = offset + i as u64;
            let mut hasher = DefaultHasher::new();
            (id, seed).hash(&mut hasher);
            let hash = hasher.finish();

            let prefix = name_prefixes[(hash % 10) as usize];
            let suffix = name_suffixes[((hash >> 8) % 10) as usize];
            let number = (hash >> 16) % 1000;

            SpacetimedbPlayer {
                id,
                identity: Identity::__dummy(),
                username: format!("{}{}{}", prefix, suffix, number),
                level: ((hash >> 24) % 100 + 1) as u32,
                experience: (hash >> 32) % 1_000_000,
                guild: guilds[((hash >> 40) % 6) as usize],
                status: statuses[((hash >> 44) % 4) as usize],
                score: ((hash >> 48) % 10000) as u32,
                games_played: ((hash >> 52) % 500) as u32,
                win_rate: ((hash >> 56) % 100) as f32,
                created_at: Timestamp::UNIX_EPOCH,
                last_seen: Timestamp::UNIX_EPOCH,
            }
        })
        .collect()
}

// ============================================================================
// Table Delegate Implementation
// ============================================================================

impl SpacetimedbPlayerTableDelegate {
    pub fn load_more_with_filters(
        &mut self,
        filters: SpacetimedbPlayerFilterValues,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        let offset = PLAYER_OFFSET.load(Ordering::SeqCst);
        let seed = PLAYER_SEED.load(Ordering::SeqCst);
        let batch_size = 30;

        info!(
            "SpacetimeDB: SELECT * FROM player WHERE username LIKE '%{}%' ...",
            filters.username
        );

        let tokio_task = Tokio::spawn(cx, async move {
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;

            let mut players = generate_mock_players(offset, batch_size, seed);

            players.retain(|p| {
                filters.username.matches(&p.username)
                    && filters.level.matches(&(p.level as f64))
                    && filters.guild.matches(&p.guild)
                    && filters.status.matches(&p.status)
            });

            (players, batch_size)
        });

        cx.spawn(async move |view, cx| {
            if let Ok((new_players, batch_size)) = tokio_task.await {
                _ = cx.update(|cx| {
                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        PLAYER_OFFSET.fetch_add(batch_size as u64, Ordering::SeqCst);

                        info!("SpacetimeDB: Received {} players", new_players.len());
                        delegate.rows.extend(new_players);

                        if delegate.rows.len() >= 300 {
                            delegate.eof = true;
                        }

                        delegate.loading = false;
                        cx.notify();
                    })
                    .ok();
                });
            }
        })
        .detach();
    }

    pub fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
        self.load_more_with_filters(SpacetimedbPlayerFilterValues::default(), window, cx);
    }

    pub fn reset_and_reload_with_filters(
        &mut self,
        filters: SpacetimedbPlayerFilterValues,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        info!("SpacetimeDB: Resubscribing with new filters");
        self.rows.clear();
        self.eof = false;
        self.loading = false;
        PLAYER_OFFSET.store(0, Ordering::SeqCst);
        PLAYER_SEED.fetch_add(1, Ordering::SeqCst);

        self.load_more_with_filters(filters, window, cx);
    }
}
