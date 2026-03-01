use std::process::Command;

const DEFAULT_SPACETIMEDB_SERVER: &str = "local";
const DEFAULT_SPACETIMEDB_DB_NAME: &str = "gpui-table-some-lib";
const DEFAULT_SEED_COUNT: u32 = 10_000;

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let count = parse_count()?;
    let server = std::env::var("SPACETIMEDB_SERVER")
        .or_else(|_| std::env::var("SPACETIMEDB_URI"))
        .unwrap_or_else(|_| DEFAULT_SPACETIMEDB_SERVER.to_string());
    let database_name = std::env::var("SPACETIMEDB_DB_NAME")
        .unwrap_or_else(|_| DEFAULT_SPACETIMEDB_DB_NAME.to_string());
    let spacetime = resolve_spacetime_cli()?;

    let status = Command::new(&spacetime)
        .arg("call")
        .arg("--server")
        .arg(server)
        .arg("--anonymous")
        .arg("-y")
        .arg(&database_name)
        .arg("seed_spacetime_events")
        .arg(count.to_string())
        .status()
        .map_err(|err| format!("Failed to run `{spacetime} call`: {err}"))?;

    if !status.success() {
        return Err(format!(
            "Seeding command failed with status {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

fn parse_count() -> Result<u32, String> {
    match std::env::args().nth(1) {
        Some(value) => value
            .parse::<u32>()
            .map_err(|err| format!("Invalid row count `{value}`: {err}")),
        None => Ok(DEFAULT_SEED_COUNT),
    }
}

fn resolve_spacetime_cli() -> Result<String, String> {
    if let Ok(path) = std::env::var("SPACETIME_BIN") {
        if !path.trim().is_empty() {
            return Ok(path);
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let local = format!("{home}/.local/bin/spacetime");
        if std::path::Path::new(&local).is_file() {
            return Ok(local);
        }
    }

    if Command::new("spacetime")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return Ok("spacetime".to_string());
    }

    Err("SpacetimeDB CLI not found. Install it or set SPACETIME_BIN.".to_string())
}
