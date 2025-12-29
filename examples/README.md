# gpui-table Examples

```sh
cargo run
```

## Quick Start

Use `just` for common tasks:

```bash
just --list          # See all available commands
just run             # Run the storybook
```

## Database Examples

### SeaORM + PostgreSQL

Demonstrates real database integration with SeaORM entities and PostgreSQL.

```bash
# One-command setup (starts postgres, runs migrations, seeds data)
just seaorm-setup

# Or step by step:
just postgres-start   # Start PostgreSQL in Docker
just seaorm-migrate   # Run migrations
just seaorm-seed 100  # Seed 100 test orders

# Run with database connection
just run-seaorm
```

**Crates:**
- `seaorm-migration/` - Database migrations
- `some-lib/` - Entity definitions with `#[derive(DeriveEntityModel)]` and shared enums

### SpacetimeDB

Demonstrates real-time data with SpacetimeDB server module.

```bash
# Start SpacetimeDB (in a separate terminal)
just spacetime-start

# Setup (publishes module, seeds data)
just spacetime-setup

# Or step by step:
just spacetime-publish    # Publish the server module
just spacetime-seed 100   # Seed 100 test players
just spacetime-generate   # Generate client bindings
```

**Crates:**
- `spacetimedb-server/` - Server module with `#[table]` and `#[reducer]` definitions

## Project Structure

```
examples/
├── i18n/                    # i18n resources (fluent files)
├── seaorm-migration/        # SeaORM migrations
│   └── src/
│       ├── lib.rs
│       └── m..._*.rs   # Migration: create orders table
├── spacetimedb-server/      # SpacetimeDB server module
│   └── src/
│       └── lib.rs           # Player table, reducers
├── some-lib/                # Shared types and table definitions
│   └── src/
│       └── structs/
│           ├── product.rs           # DummyJSON API example
│           ├── seaorm_order.rs      # SeaORM + gpui-table integration
│           ├── spacetimedb_player.rs # SpacetimeDB + gpui-table integration
│           └── user.rs              # Fake data example
├── some-lib-tables/         # Storybook UI for table examples
│   └── src/
│       ├── main.rs          # Storybook app entry
│       └── tables/          # Story implementations
├── prototyping/             # Code generation tool for forms
├── justfile                 # Task runner commands
└── README.md
```

## Examples Overview

| Example | Data Source | Filters | Features |
|---------|-------------|---------|----------|
| **User** | Fake data | Text, Number, Faceted, Date | Infinite scroll, client-side |
| **Product** | DummyJSON API | Text, Faceted | Real API, server-side filtering |
| **SeaORM Order** | PostgreSQL | Text, Number, Faceted, Date | Real DB, pagination |
| **SpacetimeDB Player** | SpacetimeDB | Text, Number, Faceted | Real-time subscriptions |

## Development

```bash
# Check all crates compile
just check

# Clean up databases
just clean
```
