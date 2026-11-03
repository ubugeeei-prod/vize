# Davinci completion gates

Davinci is complete when every gate below passes on `main`. The gates restate the
roadmaps [#6826](https://github.com/ubugeeei-prod/vize/issues/6826) (levels),
[#6827](https://github.com/ubugeeei-prod/vize/issues/6827) (products),
[#6828](https://github.com/ubugeeei-prod/vize/issues/6828) (legacy deletion),
[#6829](https://github.com/ubugeeei-prod/vize/issues/6829) (frameworks) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830) (CI) as checks.

```sh
node tools/commands/davinci/completion-gates.ts            # measured gates
node tools/commands/davinci/completion-gates.ts --issues   # plus issue state via gh
```

A **measured** gate is computed from the repository by the tool. An **issue** gate passes
when all of its issues are closed; each issue's body is its acceptance criterion. Counts
are not committed here; the tool prints them.

## Rules

- Only native Davinci work counts. A legacy-backed result never passes a gate (#6828).
- A definition changes only by a maintainer decision recorded in this file. The tool and
  the tables change together; `tests/tooling/davinci-completion-gates.test.ts` keeps their
  ids identical.
- Tiers are the dependency order. A tier's work needs the earlier gates it consumes; the
  `**Order:**` line of each issue in #6826 gives the finer edges. Structural slices may
  still merge in parallel under the
  [2026-09-28 decision](https://github.com/ubugeeei-prod/vize/issues/6826#issuecomment-5871526916).
  A product's default route switches only after `history.fixtures` and
  `harness.differential` pass for that product.

## T0: foundation

| Gate                   | Passes when                                                                                                                                                                                                                                                                                              | Kind     | Issues                                                 |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------------------------------------------------ |
| `levels.crates`        | The only crates under `davinci/` are `vize_l0`, `vize_l0_derive`, `vize_l1`, `vize_l1_to_l2`, `vize_l2`, `vize_l2_to_l3`, `vize_l3` and `vize_l4`.                                                                                                                                                       | measured | #6833                                                  |
| `levels.dependencies`  | No level crate has a normal or build dependency on a legacy crate: `docs/davinci/plan/level-dependency-allowlist.json` is empty.                                                                                                                                                                         | measured | #6831                                                  |
| `levels.skeleton`      | No `#![expect(clippy::todo, reason = "skeleton: …")]` module remains.                                                                                                                                                                                                                                    | measured | #6826                                                  |
| `levels.codenames`     | No codename (Sinopia, Disegno, Ricalco, Impeto, Stesura, Folio, Spolvero) or `davinci-opt` appears in a path, a line or a serialized id, and no serialized id starts with `davinci.`. Exempt: `docs/`, `CHANGELOG.md`, lines that say "codename" (the crate-doc alias) and captured output of past runs. | measured | #6832                                                  |
| `formal.rust-only`     | No Lean source, Lake file or Lean workflow remains; the laws run as Kani harnesses or Rust oracles.                                                                                                                                                                                                      | measured | #10825                                                 |
| `ci.tiers`             | PR checks meet p50 ≤ 3 min and p90 ≤ 6 min, with the full suites in the merge queue.                                                                                                                                                                                                                     | issue    | #6830                                                  |
| `harness.differential` | One differential harness runs every product, dialect coverage has no gaps, real-project differences reduce into fixtures, and both JSX oracles have value-sensitive rows.                                                                                                                                | issue    | #6891, #6892, #6893, #6884, #6890                      |
| `history.fixtures`     | Every behavior fix in each product's history is a fixture in that product's differential corpus.                                                                                                                                                                                                         | issue    | #6879, #6880, #6881, #6883 (formatter #6882 is closed) |

## T1: level structure

| Gate               | Passes when                                                                                                                                                 | Kind  | Issues       |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | ------------ |
| `levels.markup`    | The template lexer and tree builder live in `vize_l1::markup` with document and component profiles, and petite-vue runs on L1/L2.                           | issue | #6835, #6843 |
| `levels.container` | SFC block splitting is `vize_l1::container`; nothing reads the Croquis SFC parser.                                                                          | issue | #6837        |
| `levels.dialects`  | Each level keeps dialects in its own modules, resolved from one per-file descriptor; core modules don't reference them; Vue 0.x, 1.x and quirks are native. | issue | #6841, #6842 |

## T2: IR contracts

| Gate            | Passes when                                                             | Kind  | Issues |
| --------------- | ----------------------------------------------------------------------- | ----- | ------ |
| `levels.l2`     | L2 has one canonical artifact and owns the former S2 passes.            | issue | #6838  |
| `levels.query`  | Products read semantics through one L2 query API.                       | issue | #6871  |
| `levels.script` | Native script analysis replaces Croquis.                                | issue | #6844  |
| `levels.jsx`    | Vue JSX lowers to the same L2 semantics as babel TSX and vue-jsx-vapor. | issue | #6885  |

## T3: backends

| Gate        | Passes when                                                | Kind  | Issues |
| ----------- | ---------------------------------------------------------- | ----- | ------ |
| `levels.l3` | L3 makes the shared decisions for every backend.           | issue | #6839  |
| `levels.l4` | DOM, SSR, Vapor and virtual-TS emission live in `vize_l4`. | issue | #6840  |

## T4: products

| Gate                    | Passes when                                                                                                                                                                    | Kind     | Issues       |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------- | ------------ |
| `products.formatter`    | Glyph formats from L1 only.                                                                                                                                                    | issue    | #6847        |
| `products.linter`       | Patina runs syntax rules on L1 and semantic rules on L2.                                                                                                                       | issue    | #6848        |
| `products.typechecker`  | Canon type-checks through the `vize_l4` TS projection.                                                                                                                         | issue    | #6849        |
| `products.lsp`          | Maestro serves from incremental level artifacts and never parses on its own.                                                                                                   | issue    | #6850, #6872 |
| `products.project`      | The CLI and the LSP share one project model.                                                                                                                                   | issue    | #6874        |
| `products.fixes`        | Diagnostic fixes, code actions and lint autofixes are one mechanism that edits L1 spans.                                                                                       | issue    | #6876        |
| `products.dependencies` | Glyph, Patina, Canon and Maestro have no normal dependency, direct or inherited, on a legacy crate: every file in `docs/davinci/plan/product-dependency-allowlists/` is empty. | measured | #6851        |

## T5: legacy deletion

| Gate                   | Passes when                                                                                                                                                                                              | Kind     | Issues |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------ |
| `legacy.zero-fallback` | Every product's native-only verifier reports zero legacy fallbacks over the differential corpus, every dialect listed in `docs/davinci/plan/deletion-readiness-policy.json` and the real-project corpus. | issue    | #6854  |
| `legacy.acceptance`    | Native-only acceptance rates are published per product.                                                                                                                                                  | issue    | #6853  |
| `legacy.stability`     | `deletion-readiness-policy.json` sets `minimumStableDays`, and `legacy.zero-fallback` has held for that long. The tool measures the policy; the #6854 verifier owns the held period.                     | measured | #6828  |
| `legacy.deleted`       | No legacy crate remains in the workspace: `vize_armature`, `vize_relief`, `vize_croquis`, `vize_croquis_cf` and every `vize_atelier_*`.                                                                  | measured | #6828  |

## T6: frameworks

| Gate               | Passes when                                                                              | Kind  | Issues |
| ------------------ | ---------------------------------------------------------------------------------------- | ----- | ------ |
| `frameworks.tsrx`  | TSRX runs on the levels, compiler included, with parity against the reference compiler.  | issue | #6858  |
| `frameworks.solid` | Solid runs on the levels, compiler included, with parity against the reference compiler. | issue | #6859  |

## Roadmap closure

- #6826 closes when every `levels.*` gate and `formal.rust-only` pass.
- #6827 closes when every `products.*` gate passes.
- #6828 closes when `history.fixtures`, `harness.differential` and every `legacy.*` gate pass.
- #6829 closes when every `frameworks.*` gate passes.
- #6830 is the `ci.tiers` gate.

## Pending maintainer decisions

1. **Stability period** (`legacy.stability`). `minimumStableDays` is `null`.
   Options: (a) 14 days, (b) 30 days, (c) a number of consecutive releases.
   Releases ship per fix, several a day, so a release count would pass within hours.
2. **Framework scope** (`frameworks.*`). #6829 lists TSRX and Solid as children; Svelte,
   Angular and Analog have no issues. Options: (a) complete means TSRX and Solid, and the
   others are a later program; (b) add Svelte; (c) add all three.
3. **The `davinci` name.** #6826 lists Davinci among the codenames but keeps `docs/davinci/`
   as the program name. Since then the level crates moved under `davinci/`, and about 4,600
   tracked paths contain `davinci` (`davinci/`, `davinci_harness`, `davinci_test_support`,
   `davinci_*.rs` tests, `tests/tooling/davinci-*`). Options: (a) Davinci is the program
   name, allowed in paths and tool crates, and only serialized `davinci.*` ids are banned
   (the current gate); (b) it is a codename everywhere except `docs/davinci/`, so those
   paths are renamed too.
