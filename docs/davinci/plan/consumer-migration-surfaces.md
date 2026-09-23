<!-- GENERATED FILE - do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write
     Verify:     rust-script tools/commands/davinci/consumer-migration-surfaces.rs --check
     Totals:     rust-script tools/commands/davinci/consumer-migration-surfaces.rs --summary
     Generator:  tools/support/compat/davinci/consumer-migration-surfaces.mjs -->

# Consumer migration surfaces

This inventory records where the user-facing consumers that must eventually
sit on Davinci/S0/S1/S2 still name stage crates, legacy AST/parser/Croquis
crates, or raw OXC crates directly on current `origin/main`. It is an
observational guard for planning only. It does not change rollout state.

## Resolution method

- Rust comments and string literals are stripped before matching; Cargo
  comments are stripped while dependency keys remain visible.
- Matches are lexical crate/surface names, not type-resolved imports. A row
  means "this file directly names this surface", not necessarily that every
  mention is a runtime dependency edge.
- Stage names are split into preferred physical names and compatibility
  code-name aliases so S0/S1/S2 migration work is measurable without changing
  rollout state.
- `source/manifest` includes production Rust files plus crate manifests.
  `test/dev` includes crate `tests`, `benches`, `tests.rs`,
  `*_tests.rs`, and Rust sites after the first `#[cfg(test)]` in a file.
- Content-mapper files under Canon are reported separately from the broader
  typechecker row so that protocol work can move in smaller PRs.

## Surface legend

| surface          | group | matched name classes                                                                                                                               |
| ---------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Davinci          | stage | preferred: `vize_davinci`                                                                                                                          |
| S0               | stage | preferred: `vize_s0`<br>compat/code-name: `vize_carton`                                                                                            |
| S1               | stage | preferred: `vize_s1`<br>compat/code-name: `vize_sinopia`                                                                                           |
| S2               | stage | preferred: `vize_s2`<br>compat/code-name: `vize_disegno`                                                                                           |
| S1->S2           | stage | preferred: `vize_s1_to_s2`<br>compat/code-name: `vize_ricalco`                                                                                     |
| old AST/parser   | old   | legacy: `vize_relief`, `vize_armature`                                                                                                             |
| Croquis analysis | old   | legacy: `vize_croquis`, `vize_croquis_cf`                                                                                                          |
| raw OXC          | raw   | raw: `oxc_allocator`, `oxc_ast`, `oxc_ast_visit`, `oxc_codegen`, `oxc_formatter`, `oxc_formatter_core`, `oxc_parser`, `oxc_semantic`, `oxc_syntax` |

## Consumers

- **Compiler** (`compiler`): build command plus atelier compiler crates.
- **Linter** (`linter`): lint command plus Patina rule engine.
- **Typechecker** (`typechecker`): check command plus Canon, excluding dedicated content-mapper files.
- **Typechecker content-mapper** (`typechecker-content-mapper`): content-mapper command plus Canon content-mapper protocol files.
- **Formatter** (`formatter`): fmt command, Glyph formatter crate, and LSP format handler.
- **LSP** (`lsp`): lsp/ide commands plus Maestro editor/server crate.

## Shards

Every row lives in exactly one TSV shard per (consumer, crate) under
`docs/davinci/plan/consumer-migration-surfaces/`: one row per file x class x surface x matched name,
columns `consumer_id`, `consumer`, `class` (`source`, `manifest`,
`test/dev`), `file`, `first_line`, `surface_id`, `surface`,
`surface_group`, `matched_name`, `name_kind`, `sites`. The shard set is
fixed by the consumer scopes (table below), so it only changes when a scope does.

Cross-file aggregates — per-consumer and per-surface totals and the top files
by site count — are deliberately **not committed**: they changed with every PR
and made every open PR conflict. They are pure sums over the shards; print
them with `rust-script tools/commands/davinci/consumer-migration-surfaces.rs --summary`. The staleness check (TS-12)
byte-compares this page, every shard, and the shard set itself.

| consumer                   | shard                                                                                                                  | scanned roots                                                                                            |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Compiler                   | [`compiler/vize.tsv`](./consumer-migration-surfaces/compiler/vize.tsv)                                                 | `crates/vize/src/commands/build.rs`<br>`crates/vize/src/commands/build`                                  |
| Compiler                   | [`compiler/vize_atelier_core.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_core.tsv)                       | crate `vize_atelier_core` (`Cargo.toml`, `src`, `tests`, `benches`)                                      |
| Compiler                   | [`compiler/vize_atelier_dom.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_dom.tsv)                         | crate `vize_atelier_dom` (`Cargo.toml`, `src`, `tests`, `benches`)                                       |
| Compiler                   | [`compiler/vize_atelier_jsx.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_jsx.tsv)                         | crate `vize_atelier_jsx` (`Cargo.toml`, `src`, `tests`, `benches`)                                       |
| Compiler                   | [`compiler/vize_atelier_sfc.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_sfc.tsv)                         | crate `vize_atelier_sfc` (`Cargo.toml`, `src`, `tests`, `benches`)                                       |
| Compiler                   | [`compiler/vize_atelier_ssr.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_ssr.tsv)                         | crate `vize_atelier_ssr` (`Cargo.toml`, `src`, `tests`, `benches`)                                       |
| Compiler                   | [`compiler/vize_atelier_vapor.tsv`](./consumer-migration-surfaces/compiler/vize_atelier_vapor.tsv)                     | crate `vize_atelier_vapor` (`Cargo.toml`, `src`, `tests`, `benches`)                                     |
| Linter                     | [`linter/vize.tsv`](./consumer-migration-surfaces/linter/vize.tsv)                                                     | `crates/vize/src/commands/lint.rs`<br>`crates/vize/src/commands/lint`                                    |
| Linter                     | [`linter/vize_patina.tsv`](./consumer-migration-surfaces/linter/vize_patina.tsv)                                       | crate `vize_patina` (`Cargo.toml`, `src`, `tests`, `benches`)                                            |
| Typechecker                | [`typechecker/vize.tsv`](./consumer-migration-surfaces/typechecker/vize.tsv)                                           | `crates/vize/src/commands/check.rs`<br>`crates/vize/src/commands/check`                                  |
| Typechecker                | [`typechecker/vize_canon.tsv`](./consumer-migration-surfaces/typechecker/vize_canon.tsv)                               | crate `vize_canon` (`Cargo.toml`, `src`, `tests`, `benches`) (filtered, see scope)                       |
| Typechecker content-mapper | [`typechecker-content-mapper/vize.tsv`](./consumer-migration-surfaces/typechecker-content-mapper/vize.tsv)             | `crates/vize/src/commands/content_mapper.rs`<br>`crates/vize/src/commands/content_mapper`                |
| Typechecker content-mapper | [`typechecker-content-mapper/vize_canon.tsv`](./consumer-migration-surfaces/typechecker-content-mapper/vize_canon.tsv) | `crates/vize_canon/src/batch/virtual_project` (filtered, see scope)                                      |
| Formatter                  | [`formatter/vize.tsv`](./consumer-migration-surfaces/formatter/vize.tsv)                                               | `crates/vize/src/commands/fmt.rs`<br>`crates/vize/src/commands/fmt`                                      |
| Formatter                  | [`formatter/vize_glyph.tsv`](./consumer-migration-surfaces/formatter/vize_glyph.tsv)                                   | crate `vize_glyph` (`Cargo.toml`, `src`, `tests`, `benches`)                                             |
| Formatter                  | [`formatter/vize_maestro.tsv`](./consumer-migration-surfaces/formatter/vize_maestro.tsv)                               | `crates/vize_maestro/src/server/format.rs`                                                               |
| LSP                        | [`lsp/vize.tsv`](./consumer-migration-surfaces/lsp/vize.tsv)                                                           | `crates/vize/src/commands/lsp.rs`<br>`crates/vize/src/commands/ide.rs`<br>`crates/vize/src/commands/ide` |
| LSP                        | [`lsp/vize_maestro.tsv`](./consumer-migration-surfaces/lsp/vize_maestro.tsv)                                           | crate `vize_maestro` (`Cargo.toml`, `src`, `tests`, `benches`)                                           |

## Independently mergeable no-rollout slices

1. `test(davinci): pin consumer migration surfaces` - this artifact and its
   drift test. It makes the current dependency shape reviewable without
   changing command routing or defaults.
2. `refactor(compiler): introduce stage-named compiler boundary adapters` -
   add S0/S1/S2 adapter entrypoints inside the atelier crates while continuing
   to feed the existing Relief/Croquis pipeline. Guard with compiler fixture
   parity and keep the `vize build` path unchanged.
3. `refactor(linter): add template analysis facade` - move rule code toward
   a stable analysis contract while the facade is still backed by
   Relief/Croquis. Guard with lint divergence and rule fixture snapshots; no
   default linter backend switch.
4. `refactor(typechecker): add virtual document boundary` - introduce a
   narrow S0/S1 input contract for virtual TS generation and adapt current
   callers into it. Guard with the existing typecheck fixture matrix and
   real-project rows.
5. `test(content-mapper): pin stage-neutral mapping protocol fixtures` -
   expand content-mapper protocol fixtures around spans, virtual extensions,
   package routes, and declaration-map lookups. Keep the external tsgo protocol
   byte-compatible.
6. `refactor(formatter): isolate region formatting plan` - keep Glyph/OXC
   output unchanged, but put region extraction and script formatting behind a
   stage-neutral formatting plan. Guard with idempotence and range-formatting
   fixtures.
7. `refactor(lsp): add current-backend adapter boundary` - route Maestro
   document/virtual-code feature inputs through a backend trait whose first
   implementation delegates to the current Armature/Croquis/Canon stack. Guard
   hover, definition, diagnostics, semantic tokens, and formatting with
   existing LSP e2e tests.
8. `refactor(davinci): align physical layer names with s0/s1/s2` - migrate
   public internal module/crate references toward S0/S1/S2 naming in small
   aliasing steps. Keep code names only as compatibility aliases until all
   consumers have moved.

Rollout remains explicitly out of scope for these slices: none should switch
user-visible defaults, command dispatch, package exports, editor activation, or
protocol behavior.

## Regeneration

```sh
rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write
rust-script tools/commands/davinci/consumer-migration-surfaces.rs --check
rust-script tools/commands/davinci/consumer-migration-surfaces.rs --summary
```
