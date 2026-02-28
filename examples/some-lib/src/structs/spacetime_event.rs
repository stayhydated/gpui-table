use es_fluent::{EsFluentThis, EsFluentVariants};
use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
use gpui_table::{Filterable, GpuiTable, TableCell, TableLoader};
use gpui_tokio::Tokio;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Eq, Hash, PartialEq, es_fluent::EsFluent, Filterable, TableCell)]
#[filter(fluent)]
pub enum SpacetimeMutation {
    #[filter(icon = IconName::ArrowUp)]
    Insert,
    #[filter(icon = IconName::Settings)]
    Update,
    #[filter(icon = IconName::CircleX)]
    Delete,
}

#[derive(Clone, Debug, EsFluentThis, EsFluentVariants, GpuiTable)]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_table(fluent = "label", filters, load_more)]
pub struct SpacetimeEvent {
    #[gpui_table(sortable, width = 120., filter(text()))]
    pub table_name: String,

    #[gpui_table(sortable, width = 260.)]
    pub sender: spacetimedb::Identity,

    #[gpui_table(width = 120.)]
    pub connection_id: Option<spacetimedb::ConnectionId>,

    #[gpui_table(width = 120., filter(faceted()))]
    pub mutation: SpacetimeMutation,

    #[gpui_table(sortable, width = 220., filter(date_range()))]
    pub committed_at: spacetimedb::Timestamp,

    #[gpui_table(width = 240., filter(text()))]
    pub reducer: String,
}

const DEFAULT_STDB_URI: &str = "http://localhost:3000";
const DEFAULT_PAGE_SIZE: usize = 50;
const DEFAULT_STDB_SQL: &str = "SELECT table_name, sender, connection_id, mutation, committed_at, reducer FROM spacetime_event ORDER BY committed_at DESC";

#[derive(Clone, Debug)]
struct SpacetimeSqlConfig {
    uri: String,
    database: String,
    auth_token: Option<String>,
    sql_template: String,
    page_size: usize,
}

impl SpacetimeSqlConfig {
    fn from_env() -> Result<Self, String> {
        let database = std::env::var("GPUI_TABLE_SPACETIMEDB_DATABASE").map_err(|_| {
            "missing GPUI_TABLE_SPACETIMEDB_DATABASE env var (database name or identity)"
                .to_string()
        })?;
        let uri = std::env::var("GPUI_TABLE_SPACETIMEDB_URI")
            .unwrap_or_else(|_| DEFAULT_STDB_URI.to_string());
        let auth_token = std::env::var("GPUI_TABLE_SPACETIMEDB_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty());
        let sql_template = std::env::var("GPUI_TABLE_SPACETIMEDB_SQL")
            .unwrap_or_else(|_| DEFAULT_STDB_SQL.to_string());
        let page_size = std::env::var("GPUI_TABLE_SPACETIMEDB_PAGE_SIZE")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_PAGE_SIZE);

        Ok(Self {
            uri,
            database,
            auth_token,
            sql_template,
            page_size,
        })
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/v1/database/sql/{}",
            self.uri.trim_end_matches('/'),
            urlencoding::encode(&self.database),
        )
    }

    fn paged_query(&self, offset: usize) -> String {
        let base = self.sql_template.trim().trim_end_matches(';');
        if base.contains("{limit}") || base.contains("{offset}") {
            return base
                .replace("{limit}", &self.page_size.to_string())
                .replace("{offset}", &offset.to_string());
        }

        format!("{base} LIMIT {} OFFSET {}", self.page_size, offset)
    }
}

#[derive(Debug, Serialize)]
struct SpacetimeSqlRequest<'a> {
    query: &'a str,
}

#[derive(Debug, Deserialize)]
struct SpacetimeSqlStatement<Row> {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct SpacetimeSqlRow {
    table_name: Value,
    sender: Value,
    #[serde(default)]
    connection_id: Value,
    mutation: Value,
    committed_at: Value,
    reducer: Value,
}

impl SpacetimeMutation {
    fn from_label(label: &str) -> Option<Self> {
        match label.trim().to_ascii_lowercase().as_str() {
            "insert" => Some(Self::Insert),
            "update" => Some(Self::Update),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }

    fn from_sql_value(value: &Value) -> Option<Self> {
        if let Some(label) = value.as_str() {
            return Self::from_label(label);
        }

        if let Some(tag) = value.as_u64() {
            return match tag {
                0 => Some(Self::Insert),
                1 => Some(Self::Update),
                2 => Some(Self::Delete),
                _ => None,
            };
        }

        if let Some(items) = value.as_array() {
            if items.len() == 1 {
                return Self::from_sql_value(&items[0]);
            }

            if items.len() == 2 {
                return Self::from_sql_value(&items[0]).or_else(|| Self::from_sql_value(&items[1]));
            }
        }

        if let Some(object) = value.as_object() {
            for name in object.keys() {
                if let Some(parsed) = Self::from_label(name) {
                    return Some(parsed);
                }
            }
            if let Some(tag) = object.get("tag") {
                return Self::from_sql_value(tag);
            }
        }

        None
    }
}

impl SpacetimeEvent {
    fn from_sql_row(row: SpacetimeSqlRow) -> Result<Self, String> {
        Ok(Self {
            table_name: parse_sql_string("table_name", &row.table_name)?,
            sender: parse_sql_identity("sender", &row.sender)?,
            connection_id: parse_sql_connection_id("connection_id", &row.connection_id)?,
            mutation: SpacetimeMutation::from_sql_value(&row.mutation)
                .ok_or_else(|| format!("unsupported mutation payload: {}", row.mutation))?,
            committed_at: parse_sql_timestamp("committed_at", &row.committed_at)?,
            reducer: parse_sql_string("reducer", &row.reducer)?,
        })
    }
}

async fn fetch_spacetime_rows(
    config: SpacetimeSqlConfig,
    offset: usize,
) -> Result<Vec<SpacetimeEvent>, String> {
    let query = config.paged_query(offset);
    let endpoint = config.endpoint();
    debug!("POST {} with query {}", endpoint, query);

    let client = reqwest::Client::new();
    let mut request = client
        .post(endpoint)
        .json(&SpacetimeSqlRequest { query: &query });
    if let Some(token) = &config.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .map_err(|err| format!("failed to query SpaceTimeDB: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_else(|_| String::new());
        return Err(format!("SpaceTimeDB query failed ({status}): {body}"));
    }

    let statements: Vec<SpacetimeSqlStatement<SpacetimeSqlRow>> = response
        .json()
        .await
        .map_err(|err| format!("failed to decode SQL response: {err}"))?;

    let mut rows = Vec::new();
    for (statement_ix, statement) in statements.into_iter().enumerate() {
        for (row_ix, row) in statement.rows.into_iter().enumerate() {
            match SpacetimeEvent::from_sql_row(row) {
                Ok(event) => rows.push(event),
                Err(err) => warn!(
                    "Skipping SpaceTimeDB row (statement={}, row={}): {}",
                    statement_ix, row_ix, err
                ),
            }
        }
    }

    Ok(rows)
}

fn parse_sql_string(field: &str, value: &Value) -> Result<String, String> {
    if let Some(text) = value.as_str() {
        return Ok(text.to_string());
    }

    if let Some(array) = value.as_array() {
        if array.len() == 1 {
            return parse_sql_string(field, &array[0]);
        }
    }

    Err(format!("invalid {field} value: {value}"))
}

fn parse_sql_identity(field: &str, value: &Value) -> Result<spacetimedb::Identity, String> {
    if let Some(hex) = value.as_str() {
        return spacetimedb::Identity::from_hex(hex)
            .map_err(|err| format!("invalid {field} identity '{hex}': {err}"));
    }

    if let Some(array) = value.as_array() {
        if array.len() == 1 {
            return parse_sql_identity(field, &array[0]);
        }
    }

    if let Some(object) = value.as_object() {
        if let Some(inner) = object.get("__identity__") {
            return parse_sql_identity(field, inner);
        }
        if let Some(inner) = object.get("some") {
            return parse_sql_identity(field, inner);
        }
    }

    Err(format!("invalid {field} value: {value}"))
}

fn parse_sql_connection_id(
    field: &str,
    value: &Value,
) -> Result<Option<spacetimedb::ConnectionId>, String> {
    if value.is_null() {
        return Ok(None);
    }

    if let Some(hex) = value.as_str() {
        if hex.eq_ignore_ascii_case("null") {
            return Ok(None);
        }

        return spacetimedb::ConnectionId::from_hex(hex)
            .map(Some)
            .or_else(|_| {
                hex.parse::<u128>()
                    .map(spacetimedb::ConnectionId::from_u128)
                    .map(Some)
            })
            .map_err(|err| format!("invalid {field} connection id '{hex}': {err}"));
    }

    if let Some(raw) = value.as_u64() {
        return Ok(Some(spacetimedb::ConnectionId::from_u128(raw as u128)));
    }

    if let Some(array) = value.as_array() {
        if array.is_empty() {
            return Ok(None);
        }
        if array.len() == 1 {
            return parse_sql_connection_id(field, &array[0]);
        }
    }

    if let Some(object) = value.as_object() {
        if let Some(inner) = object.get("__connection_id__") {
            return parse_sql_connection_id(field, inner);
        }
        if let Some(inner) = object.get("some") {
            return parse_sql_connection_id(field, inner);
        }
    }

    Err(format!("invalid {field} value: {value}"))
}

fn parse_sql_timestamp(field: &str, value: &Value) -> Result<spacetimedb::Timestamp, String> {
    if let Some(raw) = value.as_i64() {
        return Ok(spacetimedb::Timestamp::from_micros_since_unix_epoch(raw));
    }

    if let Some(text) = value.as_str() {
        return spacetimedb::Timestamp::parse_from_rfc3339(text)
            .map_err(|err| format!("invalid {field} timestamp '{text}': {err}"));
    }

    if let Some(array) = value.as_array() {
        if array.len() == 1 {
            return parse_sql_timestamp(field, &array[0]);
        }
    }

    if let Some(object) = value.as_object() {
        if let Some(inner) = object.get("__timestamp_micros_since_unix_epoch") {
            return parse_sql_timestamp(field, inner);
        }
        if let Some(inner) = object.get("some") {
            return parse_sql_timestamp(field, inner);
        }
    }

    Err(format!("invalid {field} value: {value}"))
}

#[gpui_table::gpui_table_impl]
impl TableLoader for SpacetimeEventTableDelegate {
    const THRESHOLD: usize = 20;

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        let config = match SpacetimeSqlConfig::from_env() {
            Ok(config) => config,
            Err(err) => {
                warn!("SpaceTimeDB story is not configured: {}", err);
                self.loading = false;
                self.eof = true;
                cx.notify();
                return;
            },
        };

        self.loading = true;
        cx.notify();

        let offset = self.rows.len();
        info!(
            "Querying SpaceTimeDB database={} offset={} limit={}",
            config.database, offset, config.page_size
        );

        let tokio_task = Tokio::spawn(cx, async move {
            let limit = config.page_size;
            let rows = fetch_spacetime_rows(config, offset).await;
            (rows, limit)
        });

        cx.spawn(async move |view, cx| match tokio_task.await {
            Ok((result, limit)) => {
                cx.update(|cx| {
                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        match result {
                            Ok(rows) => {
                                let fetched_count = rows.len();
                                delegate.rows.extend(rows);
                                if fetched_count < limit {
                                    delegate.eof = true;
                                }
                            },
                            Err(err) => {
                                warn!("SpaceTimeDB query error: {}", err);
                                delegate.eof = true;
                            },
                        }

                        delegate.loading = false;
                        cx.notify();
                    })
                    .unwrap();
                });
            },
            Err(err) => {
                warn!("SpaceTimeDB query task failed: {:?}", err);
                cx.update(|cx| {
                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        delegate.loading = false;
                        delegate.eof = true;
                        cx.notify();
                    })
                    .unwrap();
                });
            },
        })
        .detach();
    }
}
