#!/usr/bin/env bash
set -euo pipefail

DB_NAME="${SPACETIMEDB_DB_NAME:-gpui-table-some-lib}"
SERVER="${SPACETIMEDB_SERVER:-local}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
MODULE_PATH="$SCRIPT_DIR"
BINDINGS_DIR="$MODULE_PATH/src/module_bindings"

SPACETIME="$HOME/.local/bin/spacetime"
if [ ! -x "$SPACETIME" ]; then
    if command -v spacetime >/dev/null 2>&1; then
        SPACETIME="spacetime"
    else
        echo "SpacetimeDB CLI not found."
        echo "Install it with: curl -sSf https://install.spacetimedb.com | sh"
        exit 1
    fi
fi

supports_flag() {
    local subcommand="$1"
    local flag="$2"
    "$SPACETIME" "$subcommand" --help 2>&1 | grep -q -- "$flag"
}

echo "SpacetimeDB CLI: $($SPACETIME --version)"

if [ "$SERVER" = "local" ]; then
    echo "Checking local SpacetimeDB server status..."
    if ! "$SPACETIME" server status 2>/dev/null | grep -qi "running"; then
        echo "Starting local SpacetimeDB server..."
        "$SPACETIME" start &
        sleep 3
    fi
else
    echo "Using SpacetimeDB server '$SERVER' (skipping local startup)."
fi

echo "Building module with SpacetimeDB CLI..."
if supports_flag build "--build-options"; then
    "$SPACETIME" build --module-path "$MODULE_PATH" --build-options='--features db'
else
    "$SPACETIME" build --module-path "$MODULE_PATH"
fi

echo "Publishing module '$DB_NAME'..."
"$SPACETIME" publish --server "$SERVER" --module-path "$MODULE_PATH" --build-options='--features db' --delete-data -y "$DB_NAME" || {
    echo "Retrying publish without --delete-data..."
    "$SPACETIME" publish --server "$SERVER" --module-path "$MODULE_PATH" --build-options='--features db' -y "$DB_NAME"
}

echo "Generating Rust client bindings..."
rm -rf "$BINDINGS_DIR"
mkdir -p "$BINDINGS_DIR"
"$SPACETIME" generate --lang rust \
    --out-dir "$BINDINGS_DIR" \
    --module-path "$MODULE_PATH" \
    --build-options='--features db'

echo "Setup complete."
echo "Try: cargo run -p some-lib-tables"
echo "Seed rows: cargo run -p some-lib --features seed-bin --bin seed_spacetime_events -- 10000"
