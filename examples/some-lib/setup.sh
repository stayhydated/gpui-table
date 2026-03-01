#!/usr/bin/env bash
set -euo pipefail

DB_NAME="${SPACETIMEDB_DB_NAME:-gpui-table-some-lib}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
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

echo "SpacetimeDB CLI: $($SPACETIME --version)"

echo "Building module (db)..."
cd "$ROOT_DIR"
cargo build --release -p some-lib --features db

echo "Checking SpaceTimeDB server status..."
if ! "$SPACETIME" server status 2>/dev/null | grep -qi "running"; then
    echo "Starting SpaceTimeDB server..."
    "$SPACETIME" start &
    sleep 3
fi

echo "Publishing module '$DB_NAME'..."
"$SPACETIME" publish --server local --module-path "$MODULE_PATH" --build-options='--features db' --delete-data=always -y "$DB_NAME" || {
    echo "Retrying publish without --delete-data..."
    "$SPACETIME" publish --server local --module-path "$MODULE_PATH" --build-options='--features db' -y "$DB_NAME"
}

echo "Generating Rust client bindings..."
rm -rf "$BINDINGS_DIR"
mkdir -p "$BINDINGS_DIR"
"$SPACETIME" generate --lang rust \
    --out-dir "$BINDINGS_DIR" \
    --module-path "$MODULE_PATH" \
    --build-options='--features db'

echo "Setup complete."
echo "Try: cargo run -p some-lib-tables --features client"
echo "Seed rows: cargo run -p some-lib --features seed-bin --bin seed_spacetime_events -- 10000"
