default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt

test-publish:
  cargo publish --workspace --dry-run --allow-dirty
