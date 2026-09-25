# Working in gpui-table

Start with `crates/gpui-table` for application-facing API changes,
`examples/README.md` for runnable examples, and `just --list` for local commands.

## Where changes belong

| Surface | Ownership |
|---|---|
| `crates/gpui-table` | Public facade, macro re-exports, and feature gates |
| `crates/gpui-table-component` | Built-in filter widgets, adapters, reset controls, and `TableStatusBar` |
| `crates/gpui-table-core` | UI-neutral typed filter values and matching semantics |
| `crates/gpui-table-runtime` | GPUI row, cell, loading, filter-shape, and generated-filter contracts |
| `crates/gpui-table-schema` | UI-neutral filter metadata and table-shape registry consumed by tooling |
| `crates/gpui-table-derive` | Table, cell, filter, and MCP macro expansion and diagnostics |
| `crates/gpui-table-mcp` | Public query contracts, schemas, registration, and server composition |
| `crates/gpui-table-prototyping-core` | Syntax-tree generation from registered table metadata |
| `examples/some-lib` | Canonical row models, localization, loading, and context-menu examples |
| `examples/some-lib-tables` | Native and Wasm Storybook views that compose those rows and filters |
| `examples/mcp-query` | Runnable stdio MCP query example |
| `examples/prototyping` | Generator and checked-in table-story output |
| `book/src`, `skills/` | User workflows and application-agent guidance |
| `web/src/lib.rs`, `xtask` | Public catalog and book, demo, and Pages build orchestration |

The default `cargo run` target is `some-lib-tables`. Its
`examples/demo.rs` entry point discovers the same stories on native and Wasm
targets. Built-in widget previews live in `crates/gpui-table-component/src/stories`.

## Keep related surfaces aligned

- When derive syntax, feature gates, generated filter behavior, MCP queries,
  localization, or another public contract changes, update the owning source
  and affected rustdocs, root and crate READMEs, `book/src` chapters, examples,
  and repository skills. Update `web/src/lib.rs` when its catalog description
  changes and this guide when routing or validation changes.
- Use `skills/use-gpui-table` for table composition guidance and
  `skills/use-gpui-table-component-shapes` for adapter and custom-shape guidance.
- When generated contracts change, check the facade, derive, runtime, schema,
  and affected component or MCP crates together. Keep the corresponding
  `crates/gpui-table/tests/ui` fixtures and
  `crates/gpui-table/tests/snapshots` expectations aligned. Review `.stderr`
  and snapshot diffs before accepting them.
- When built-in filters or query values change, update their component stories
  and affected `some-lib` row models and `some-lib-tables` views.
- When Fluent behavior changes, update the affected `i18n.toml`, `i18n/*.ftl`,
  localized examples, and user guidance together.
- Keep internal contracts near their source, tests, fixtures, and generator
  inputs. Use the book and skills for the application workflows they describe.

## Generated output and build ownership

Regenerate `examples/prototyping/output` with `cargo run -p prototyping` when
inventory, code generation, table-shape metadata, `ComponentShapeUse`,
`ColumnVariant`, `FilterVariant`, or the prototyping layout changes. Edit the
owning metadata or generator instead of the generated files.

Build the published book, `llms.txt` files, GPUI demo, and catalog through
`cargo xtask`. Their sources are `book/src`, `web/src`, and
`examples/some-lib-tables`; the build writes into `web/public` and `web/dist`.

Linux setup has separate owners: `.cargo/config.toml` controls local environment
settings, `.github/actions/install-linux-deps` installs GPUI system packages,
and `.github/workflows/ci.yml` defines CI jobs. The Pages pipeline is in
`.github/workflows/gh-pages.yml`. Change the surface that owns the build requirement.

## Validate the edited surface

- Choose the affected crate, example, or documentation check from `just --list`.
  `just check` and `just clippy` exclude `some-lib` and `some-lib-tables`; use
  direct Cargo commands when those examples need validation.
- Run built-in widget previews with
  `cargo run -p gpui-table-component --bin story --features story`.
- For book edits, run `cargo xtask build book` and
  `cargo xtask build llms-txt`. For the nightly Wasm demo, use
  `cargo xtask build gpui-demo`; for the catalog, use `cargo xtask build web`.
  `just web-build` runs the complete publication pipeline.
- `just test` runs workspace tests with all features. `just cov` measures all
  targets and features except the prototyping generator, GUI application, and
  publication tooling.
- CI also checks formatting, Clippy, rustdocs, package contents, coverage,
  Fluent resources, and unused dependencies. Read `.github/workflows/ci.yml`
  when matching its full checks; local recipes intentionally cover different
  scopes.

Report the commands actually run, their results, and any checks left unrun.
