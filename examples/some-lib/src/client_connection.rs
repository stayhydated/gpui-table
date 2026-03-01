use crate::module_bindings::DbConnection;
use log::{info, warn};
use spacetimedb_sdk::DbContext as _;
use std::sync::{Arc, OnceLock};

static CONNECTION: OnceLock<Arc<DbConnection>> = OnceLock::new();
const DEFAULT_SPACETIMEDB_URI: &str = "http://127.0.0.1:3000";
const DEFAULT_SPACETIMEDB_DB_NAME: &str = "gpui-table-some-lib";
const SPACETIME_EVENT_SUBSCRIPTION: &str = "SELECT * FROM spacetime_event";

pub fn init(conn: DbConnection) -> Result<Arc<DbConnection>, String> {
    let arc = Arc::new(conn);
    CONNECTION
        .set(arc.clone())
        .map_err(|_| "DbConnection already initialized".to_string())?;
    Ok(arc)
}

pub fn init_from_env() -> Result<Arc<DbConnection>, String> {
    if let Some(existing) = CONNECTION.get().cloned() {
        return Ok(existing);
    }

    let uri =
        std::env::var("SPACETIMEDB_URI").unwrap_or_else(|_| DEFAULT_SPACETIMEDB_URI.to_string());
    let database_name = std::env::var("SPACETIMEDB_DB_NAME")
        .unwrap_or_else(|_| DEFAULT_SPACETIMEDB_DB_NAME.to_string());
    let token = std::env::var("SPACETIMEDB_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty());

    info!(
        "Connecting to SpaceTimeDB uri={} database={}",
        uri, database_name
    );

    let conn = DbConnection::builder()
        .with_uri(uri.as_str())
        .with_database_name(database_name.as_str())
        .with_token(token)
        .on_disconnect(|_, error| {
            if let Some(error) = error {
                warn!("SpaceTimeDB disconnected with error: {}", error);
            } else {
                info!("SpaceTimeDB disconnected");
            }
        })
        .build()
        .map_err(|error| format!("Failed to create DbConnection: {}", error))?;

    let _ = conn
        .subscription_builder()
        .subscribe(SPACETIME_EVENT_SUBSCRIPTION);
    let _ = conn.run_threaded();

    init(conn)
}

pub fn get() -> Result<Arc<DbConnection>, String> {
    CONNECTION
        .get()
        .cloned()
        .ok_or_else(|| "DbConnection not initialized".to_string())
}
