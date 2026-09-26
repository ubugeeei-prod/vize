# Decision Record — Level Restructure (2026-09-27)

> [!NOTE]
> This record and its linked companion pages collect the decisions from
> the maintainer's 2026-09-27 design session. It is the working source of truth for crate layout, naming, level
> responsibilities and CI tiers. Where it conflicts with older pages (S0–S4
> naming, the `vize_davinci` substrate crate, Folio naming, S4 placement,
> charter rows #1, #5 and #11), this record wins until those pages are
> rewritten. Each section links to the issue that tracks the work.
> The [Open Questions entry](../open-questions.md#level-restructure-2026-09-27)
> records what changed against the charter.

Tracking issues: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826)
(restructure), [#6827](https://github.com/ubugeeei-prod/vize/issues/6827)
(products), [#6828](https://github.com/ubugeeei-prod/vize/issues/6828)
(legacy deletion), [#6829](https://github.com/ubugeeei-prod/vize/issues/6829)
(multi-framework), [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)
(CI).

## Levels and naming

- The stages are renamed **L0–L4** (levels). The rename is in progress.
- **Crates are named by level only**: `vize_l0` … `vize_l4`, and conversion
  crates `vize_l1_to_l2` and `vize_l2_to_l3`. The one exception is
  `vize_l0_derive`, because proc-macro crates must be separate.
- There are no functional crate names (no `vize_folio`, `vize_dialect`, …).
  Functional concerns are modules inside a level crate.
- **Codenames are aliases only.** Davinci, Sinopia, Disegno, Ricalco, Impeto,
  Folio, Spolvero and the rest may appear in crate docs
  (`//! L3 — reactivity IR (codename: Impeto)`) and in `docs/`. They never
  appear in crate, directory, file, module or type names, or in serialized
  strings. Ordinary terms such as `lattice` and `ledger` are not codenames.
- Examples of the mapping ([#6832](https://github.com/ubugeeei-prod/vize/issues/6832)):

  | Now                                   | After                                                                     |
  | ------------------------------------- | ------------------------------------------------------------------------- |
  | `folio`, `trait Folio`, `FolioValue`  | `dump`, `trait Dump`, `DumpValue`                                         |
  | `FolioOp`, `FolioIf`, …               | `vize_l2::dump::{Op, If, …}`                                              |
  | `vize_impeto`                         | `vize_l3`                                                                 |
  | `impeto.set-prop`                     | `l3.set-prop`                                                             |
  | `davinci.s2_dom.*` counters           | `l2.dom.*`                                                                |
  | `SpolveroFeed`, `inspector::spolvero` | `StageFeed`, `inspector::stages`                                          |
  | feature `davinci-differential`        | `legacy-differential`                                                     |
  | `davinci-opt`                         | `vize dump` options (one generator shared with the playground stage view) |

- Davinci has no external users yet, so its crates, APIs, dump formats,
  serialized strings and feature names change without a compatibility
  period. Legacy products keep strict output compatibility.
- Product crates with art names (croquis, patina, glyph, maestro, canon,
  carton, …) keep their names. `docs/davinci/` keeps its name as the program
  name.
- The migration does not freeze other work. Move-only commits keep git
  rename detection working for in-flight fixes, renames are scripted (on a
  conflict, re-run the script on `main`), and PRs stay small.

## `vize_davinci` is deleted

Tracked in [#6833](https://github.com/ubugeeei-prod/vize/issues/6833) and
[#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

- **`vize_l0`** is the foundation level. It holds source, arena, span, ids
  (`NodeId`, `AnalysisId`), side tables, artifact keys, the dump trait and
  runtime, diagnostics and witnesses, the pass and fact managers, and the
  level registry. It is carved out of `vize_carton`, which keeps config,
  i18n, LSP, resolver and profiler code.
- Some code moves to the crate that uses it:
  - `summary` → `vize_l2::summary`
  - `render` → the `vize` CLI
  - the Croquis dump page → croquis
  - the stage feed → curator
  - the repro page → the CLI
  - `legacy_plan` → test support
- The accepted trade-off is that `vize_l0` becomes large. It stays readable
  through its modules.

## Dependency direction

Tracked in [#6831](https://github.com/ubugeeei-prod/vize/issues/6831) and
[#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

- Level crates never take normal dependencies on legacy crates:
  `vize_armature`, `vize_relief`, `vize_atelier_*`, `vize_croquis`,
  `vize_croquis_cf`. Legacy may depend on levels. Dev-dependencies used as
  differential oracles are fine.
- **Croquis counts as legacy.** Script analysis is rebuilt natively in the
  levels ([#6844](https://github.com/ubugeeei-prod/vize/issues/6844)). No
  adapter presents legacy output as Davinci facts.
- A `cargo metadata` gate enforces the rule. It starts with a shrinking
  allowlist and later covers product crates too.

## L1: what the text _is_

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835),
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and
[#6837](https://github.com/ubugeeei-prod/vize/issues/6837). The detailed
design is in the
[#6836 design comment](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).

- **Boundary rule:** L1 is what the text _is_ (the concrete syntax of every
  grammar, lossless). L2 is what it _does_.
- **Markup** is a grammar × profile matrix:
  - Grammars: Vue now; Svelte, Angular and others later.
  - Profiles: `document` (HTML and in-DOM rules, used by petite-vue) and
    `component` (SFC template rules).
  - One shared lexer, `Lexer<P: Profile>`, with static dispatch.
  - The template tokenizer moves from armature into `vize_l1::markup`, so
    armature depends on L1 and not the other way round.
- **Container** is the file-format layer. SFC block splitting moves here out
  of Croquis, laid out so Svelte, Analog and TSRX containers fit later.
- **Dialect syntax hooks** decompose directive names (`v-on:click.stop`,
  `@click`, `#default`, `:[dyn]`). They play the role of MLIR custom
  assembly formats.
- **Typed embeds.** Attribute values, mustaches and dynamic arguments become
  `Embed { grammar, source }` with `Grammar = Shape × Lang`:
  - `Shape` is the role inside the markup: `Expr`, `HandlerBody`, `ForHead`,
    `SlotParams`, `FilterChain`, …
  - `Lang` is the host language, resolved once per file: JS and TS now;
    Flow, MoonBit and others later.
  - Composite shapes (`ForHead`, `FilterChain`) are built by the dialect from
    language pieces.
- **Embed trees** are a separate L1 artifact keyed by embed id:
  - The JS/TS tree is the oxc AST plus spans; the source bytes are
    authoritative.
  - A broken expression becomes a hole in its own tree and never breaks the
    markup tree.
  - Embedding works in both directions and nests (markup inside script for
    JSX, TSRX and Angular inline templates).
- **Entity decoding** happens when an embed is built. The embed carries the
  decoded text and a decode map, so language providers never see HTML.
- **`v-pre`** is handled in L1, because it switches the lexing mode of its
  children.
- **Language providers** are bundles of optional capabilities, one per level
  (like MLIR interfaces):
  - L1 syntax entry points
  - L2 semantics (today's `ExprDialect`)
  - L4 emission, the type-check projection and the checker host
  - the formatter printer

  A missing capability falls back to the opaque path or a diagnostic. There
  is one dynamic dispatch per file; everything after it is static. Template
  expressions follow `<script lang>`, and a language mismatch between
  `<script>` and `<script setup>` is a diagnostic.

- Two decisions from the same design comment belong to the next section:
  the `const` pattern table of `fn` pointers, and parsing each expression
  once (L4 rewrites from the L2 identifier-resolution table).

## L1→L2 and L2

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838).

- L1→L2 is a **table of conversion patterns**: each dialect exposes a
  `const` slice of `fn` pointers. Unconverted input becomes a diagnostic, so
  lowering stays total.
- **Expressions are parsed once, in L1.** L2 records identifier resolution
  (setup, props, ctx, local, …) in a side table. L4 rewrites by span from
  that table. The DOM emitter's re-parses for `_ctx.` prefixing go away.
- `vize_l2` owns the canonical artifact: ops plus semantic side tables.
  Transform passes move into `vize_l2`. Dialect-derived tables (`wrappers`,
  `legacy`) are absorbed by legalization and do not appear in the artifact.

## L3 is the decision layer

See the [l3 is the decision layer decisions](./2026-09-27-level-restructure-designs.md#l3-is-the-decision-layer)
in the companion record.

## L4: emission

See the [l4: emission decisions](./2026-09-27-level-restructure-designs.md#l4-emission)
in the companion record.

## Script side

See the [script side decisions](./2026-09-27-level-restructure-designs.md#script-side)
in the companion record.

## Dialects, languages, frameworks

See the [dialects, languages, frameworks decisions](./2026-09-27-level-restructure-designs.md#dialects-languages-frameworks)
in the companion record.

## JSX semantics

See the [JSX semantics decisions](./2026-09-27-level-restructure-designs.md#jsx-semantics)
in the companion record.

## Products on the levels

Tracked in [#6827](https://github.com/ubugeeei-prod/vize/issues/6827) and
[#6845](https://github.com/ubugeeei-prod/vize/issues/6845)–[#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

- One parse per file. Every product consumes the same artifacts.
- **Formatter:** L1 only. A rewrite whose safety depends on L2 facts (for
  example component-dependent self-closing) is a linter autofix instead.
- **Linter:** syntax rules on L1, semantic rules on L2 and facts. Diagnostics
  go through L0, and autofixes are L1 span edits.
- **Type checker:** the virtual-TS projection is an L4 target mapped back
  through `EmitDocument` links.
- **LSP:** holds level artifacts incrementally and never parses by itself.

## Type check

See the [type check decisions](./2026-09-27-level-restructure-designs.md#type-check)
in the companion record.

## Legacy deletion criteria

See the [legacy deletion criteria decisions](./2026-09-27-level-restructure-history.md#legacy-deletion-criteria)
in the companion record.

## Multi-framework

Tracked in [#6829](https://github.com/ubugeeei-prod/vize/issues/6829) and
[#6855](https://github.com/ubugeeei-prod/vize/issues/6855)–[#6860](https://github.com/ubugeeei-prod/vize/issues/6860).

- This supersedes charter row #1. Other frameworks are in-tree and include
  the compiler, with parity against each reference compiler.
- The order is **TSRX → Solid → others** (Svelte, Angular/Analog).
- Five independent axes: container, markup, profile, lang, framework.
- L2 keeps neutral core ops plus framework dialect ops.
- L3 gets a neutral reactivity vocabulary (signal, derived, effect,
  ordering).
- L4 gets one target per framework runtime.

## Vue Fes Japan (2026-10-24)

- **In scope:** Vue (templates and every Vue dialect), JS/TS, JSX/TSX.
- **TSRX:** not a priority; nice to have if something runs.
- **Other frameworks:** only the neutral types and names are designed before
  the talk. Flow is not in scope at all.
- **Goal (A):** for the in-scope inputs, all five products run on the shared
  L1/L2 structure as far as native work goes, with per-product
  native-only acceptance rates.
- **Deletion (B)** follows later, under the criteria above.
- Unfinished work is reported as unfinished. There are no legacy-backed
  shortcuts.

## Performance

The toolchain aims to be extremely fast. The layering must not add
pipelines or serialization cost.

- **Levels are type boundaries, not runtime boundaries.**
  - There is no serialization between levels; dumps exist only for
    `vize dump` and observers.
  - All artifacts share one arena and use dense `u32` ids.
  - Passes are fused into single walks where possible.
  - Anything unobserved costs nothing (generic observers; entity decoding
    only when `&` is present; incremental keys only in resident mode).
- **Regressions are stopped in the merge queue** with instruction-count
  measurements per stage
  ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868)). The
  wall-clock envelope runs nightly.
- **Per-stage budgets ratchet from current measurements.** Today all 102
  `wall_p50_ns` entries in `plan/budgets.toml` are unset.
- Whether `SideTable` changes from `FxHashMap` to dense `Vec` storage is
  decided after measuring table density and lookup cost
  ([#6869](https://github.com/ubugeeei-prod/vize/issues/6869)).

## Toolchain practice

Vize follows language-toolchain practice, not compiler-only practice. It
stays lightweight and fast. It avoids the heaviness of rust-analyzer-style
designs: fine-grained per-node queries, per-node reference-counted trees and
whole-workspace resident state.

- **Two tiers stay as they are.** Long-lived processes (LSP, check server,
  watch modes) use the salsa-based resident tier. The one-shot CLI uses the
  fused non-salsa pipeline.
- **One semantic query API over L2** serves every product
  ([#6871](https://github.com/ubugeeei-prod/vize/issues/6871)). See the
  [semantic query API design](./2026-09-27-level-restructure-designs.md#semantic-query-api-6871)
  in the companion record.
- **LSP state stays coarse**
  ([#6872](https://github.com/ubugeeei-prod/vize/issues/6872)):
  - Queries are per SFC block and per expression embed, not per node.
  - Node references never survive an edit. A position is resolved to a node
    on the latest snapshot through a span-sorted index.
  - Diagnostics and code actions carry a range and a document version, and
    they are recomputed when stale.
  - Memory holds artifacts only for open files, plus `SfcSummary` for every
    file.
- **Stale requests are cancelled on edit**
  ([#6873](https://github.com/ubugeeei-prod/vize/issues/6873)).
- **The CLI and the LSP share one project model**
  ([#6874](https://github.com/ubugeeei-prod/vize/issues/6874)).
- **The formatter keeps its Doc IR separate from its printer**
  ([#6875](https://github.com/ubugeeei-prod/vize/issues/6875)).
- **Edits have one representation.** Diagnostic fixes, code actions and lint
  autofixes are all L1 span edits tagged with a document version
  ([#6876](https://github.com/ubugeeei-prod/vize/issues/6876)).

## CI tiers

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6861](https://github.com/ubugeeei-prod/vize/issues/6861)–[#6867](https://github.com/ubugeeei-prod/vize/issues/6867).

| Tier           | Runs              | Target                   | Content                                                                                                                     |
| -------------- | ----------------- | ------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| T0 PR          | every push        | p50 ≤ 3 min, p90 ≤ 6 min | fmt, title-policy, clippy and tests for affected crates (nextest archive + shards), input-selected tooling tests            |
| T1 merge queue | once before merge | —                        | full workspace and tooling tests, differential corpus and acceptance gates, instruction-count performance gates, playground |
| T2 nightly     | schedule          | —                        | E2E, real-project matrix, fuzz, miri, benchmarks, editor conformance, resource budgets                                      |
| T3 release     | release           | —                        | everything, semver checks, release preflight                                                                                |

- zizmor runs only for external contributors, releases and PRs that touch
  `.github/**`.
- VRT, tsgo-required tests and ledger checks leave the PR tier.
- Whole-repo generated ledgers stop being committed.

## Order of work

See the [order of work decisions](./2026-09-27-level-restructure-order.md#order-of-work)
in the companion record.
