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
    cargo llvm-cov --workspace --all-features --all-targets \
        --exclude prototyping \
        --exclude some-lib-tables \
        --exclude xtask \
        --exclude web

test-publish:
    cargo publish --workspace --dry-run --allow-dirty

book:
    mdbook serve book

gpui-demo-build:
    cargo xtask build gpui-demo

web-build: gpui-demo-build
    cargo xtask build book
    cargo xtask build llms-txt
    cargo xtask build web

web: web-build
    dx serve --package web

web-preview: web-build
    cargo xtask preview web
