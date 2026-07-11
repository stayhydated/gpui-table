set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features --exclude some-lib --exclude some-lib-tables

check:
    cargo check --workspace --all-features --exclude some-lib --exclude some-lib-tables

test:
    cargo test --workspace --all-features

cov:
    cargo llvm-cov --workspace --all-features --all-targets --exclude prototyping --exclude some-lib-tables

test-publish:
    cargo publish --workspace --dry-run --allow-dirty
