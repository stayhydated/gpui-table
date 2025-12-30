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
