default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt

clippy:
    cargo clippy --workspace --all-features --exclude some-lib --exclude some-lib-tables

check:
    cargo check --workspace --all-features

test:
    cargo test --workspace --all-features

update_crate_paths:
    crates-paths -c gpui -o crates/gpui-table-derive/__crate_paths/gpui.rs
    crates-paths -c gpui-component -o crates/gpui-table-derive/__crate_paths/gpui_component.rs

test-publish:
    cargo publish --workspace --dry-run --allow-dirty
