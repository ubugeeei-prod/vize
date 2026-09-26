# Decision Record — Level Restructure (2026-09-27)

> [!NOTE]
> This record collects the decisions from the maintainer's 2026-09-27 design
> session. It is the working source of truth for crate layout, naming, level
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

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839). The
detailed design is in the
[#6839 design comment](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5847890775).

- **L3 decides, L4 encodes.**
  - L3 owns the decisions: the static level of each L2 node, the set of
    dynamic bindings per element, `Hoist`/`Cache` placement, and control
    regions (if, for, slot).
  - L4 owns the encoding: DOM patch flags, `dynamicProps`, `_cache[n]`,
    stringification and the block tree; Vapor templates and effects; SSR
    strings.
- **Target-specific criteria are L3 policies**, one per target:
  `l3::policy::{dom, vapor, ssr}`.
- **L3 artifacts are split in two:**
  - decision tables keyed by L2 `NodeId`, always built;
  - the flat program, built only on demand (for Vapor).
- DOM and SSR read the L2 tree plus the L3 decision tables. Vapor reads the
  L3 program. This replaces charter row #9's "SSR thin path".
- **The work moves; it does not grow.** The L3 decision computation replaces
  the DOM `pass/hoist` analysis. Its instruction count must not exceed the
  current DOM lane
  ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868)).
- **Migration:** tests compare the old DOM analysis with the L3 decisions.
  Once they agree, the old analysis is deleted. The comparison never runs in
  production.
- Remarks are recorded only while an observer is active.
- **Risk:** `_cache[n]` numbering must follow Vue's order. That is an L4
  encoding concern, not an L3 decision.

## L4: emission

Tracked in [#6840](https://github.com/ubugeeei-prod/vize/issues/6840). The
detailed design is in the
[#6840 design comment](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963).

- One crate, `vize_l4`, laid out as:
  - `write/`: the writer, built on `EmitDocument` and source maps. Legacy
    codegen depends on it.
  - `expr/`: expression rewriting
  - `runtime/`: the runtime helper vocabulary
  - `module/`: module assembly
  - `target/{dom, ssr, vapor}`, later `ts` (the type-check projection) and
    other frameworks
- **L4 writes text directly.** There is no JS AST plus codegen step.
- **One append-only writer serves every target:**
  - span links cost nothing when not recording;
  - it handles indentation and tracks the set of used helpers;
  - the preamble is assembled last, so nothing is `insert_str`-ed into the
    middle of a string.

  This gives DOM structural source maps and removes DOM's source-map
  fallback to legacy.

- SSR runs natively, without legacy codegen or Croquis.
- **Vapor generates JavaScript directly from the L3 program, with no legacy
  IR.** The port keeps the current emission order. The existing L3→legacy IR
  adapter stays as the test oracle until parity, then is deleted.
  Architectural soundness takes priority over reuse, and output stays
  byte-identical.
- **Module assembly lives in L4 `module/`.** `vize_atelier_sfc` becomes
  lane selection only, and the other `vize_atelier_*` crates become thin
  shells that select a lane and fall back to legacy.

## Dialects, languages, frameworks

Tracked in [#6841](https://github.com/ubugeeei-prod/vize/issues/6841),
[#6842](https://github.com/ubugeeei-prod/vize/issues/6842) and
[#6843](https://github.com/ubugeeei-prod/vize/issues/6843).

- Each level holds variant code in modules: `dialect/` (Vue variants: vue3,
  vue2, vue1, vue0, petite, quirks), `lang/`, `framework/` and `markup/`.
  Core modules never reference them, and a gate test enforces this.
- One descriptor per file and one capability derivation replace
  `LegacyDialectCapabilities` and `LegacyCaps`.
- Quirks is a dialect. Vue 0.x and 1.x are implemented on Davinci.
- petite-vue gets L1/L2 support (document profile plus hooks), and lint and
  LSP read it. There are no L3/L4 targets for petite-vue.
- `vize_dialect_moonbit` dissolves into per-level `lang/moonbit` modules.

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

## Legacy deletion criteria

Tracked in [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) and
[#6852](https://github.com/ubugeeei-prod/vize/issues/6852)–[#6854](https://github.com/ubugeeei-prod/vize/issues/6854).

1. Every legacy fix adds its input to the differential corpus. The merge
   queue checks that the Davinci lane matches.
2. Zero fallbacks across the corpus, every dialect and the real-project
   corpus.
3. Rules 1 and 2 hold for a stability period before deletion.

The criteria apply per product: compiler output, lint diagnostics, formatter
output, type-check diagnostics and LSP snapshots. Acceptance rates are
published for native Davinci work only.

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
  ([#6871](https://github.com/ubugeeei-prod/vize/issues/6871)).
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

Every issue body carries an `**Order:**` line; this table mirrors those lines. Pick the lowest-stage open issue whose "Start after" issues are all closed. Rows in the same stage with no mutual dependency can run in parallel.

| Stage          | Issues                                                                                                                                                                                                                                                                                                                                                                                                                             | Work                                       | Start after                                                                                                                                                                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0 Foundation   | [#6861](https://github.com/ubugeeei-prod/vize/issues/6861), [#6862](https://github.com/ubugeeei-prod/vize/issues/6862), [#6863](https://github.com/ubugeeei-prod/vize/issues/6863), [#6864](https://github.com/ubugeeei-prod/vize/issues/6864), [#6865](https://github.com/ubugeeei-prod/vize/issues/6865), [#6866](https://github.com/ubugeeei-prod/vize/issues/6866), [#6867](https://github.com/ubugeeei-prod/vize/issues/6867) | CI tiers                                   | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6868](https://github.com/ubugeeei-prod/vize/issues/6868)                                                                                                                                                                                                                                                                                                                                                                         | Merge-queue instruction-count gate         | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6831](https://github.com/ubugeeei-prod/vize/issues/6831)                                                                                                                                                                                                                                                                                                                                                                         | Dependency gate (report mode)              | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6852](https://github.com/ubugeeei-prod/vize/issues/6852), [#6853](https://github.com/ubugeeei-prod/vize/issues/6853), [#6854](https://github.com/ubugeeei-prod/vize/issues/6854)                                                                                                                                                                                                                                                 | Legacy-deletion gates                      | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6832](https://github.com/ubugeeei-prod/vize/issues/6832)                                                                                                                                                                                                                                                                                                                                                                         | Level rename (maintainer in progress)      | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6845](https://github.com/ubugeeei-prod/vize/issues/6845), [#6846](https://github.com/ubugeeei-prod/vize/issues/6846)                                                                                                                                                                                                                                                                                                             | glyph filter bug, self-closing audit       | —                                                                                                                                                                                                                                              |
| 0 Foundation   | [#6874](https://github.com/ubugeeei-prod/vize/issues/6874)                                                                                                                                                                                                                                                                                                                                                                         | Shared project model                       | —                                                                                                                                                                                                                                              |
| 1 Structure    | [#6833](https://github.com/ubugeeei-prod/vize/issues/6833)                                                                                                                                                                                                                                                                                                                                                                         | Dissolve `vize_davinci`                    | [#6832](https://github.com/ubugeeei-prod/vize/issues/6832)                                                                                                                                                                                     |
| 1 Structure    | [#6834](https://github.com/ubugeeei-prod/vize/issues/6834)                                                                                                                                                                                                                                                                                                                                                                         | Carve `vize_l0` out of carton              | [#6833](https://github.com/ubugeeei-prod/vize/issues/6833)                                                                                                                                                                                     |
| 1 Structure    | [#6835](https://github.com/ubugeeei-prod/vize/issues/6835), [#6837](https://github.com/ubugeeei-prod/vize/issues/6837), [#6841](https://github.com/ubugeeei-prod/vize/issues/6841)                                                                                                                                                                                                                                                 | L1 markup, L1 container, dialect modules   | [#6832](https://github.com/ubugeeei-prod/vize/issues/6832)                                                                                                                                                                                     |
| 1 Structure    | [#6869](https://github.com/ubugeeei-prod/vize/issues/6869)                                                                                                                                                                                                                                                                                                                                                                         | Measure `SideTable` density                | — (anytime)                                                                                                                                                                                                                                    |
| 1 Structure    | [#6843](https://github.com/ubugeeei-prod/vize/issues/6843)                                                                                                                                                                                                                                                                                                                                                                         | petite-vue                                 | [#6835](https://github.com/ubugeeei-prod/vize/issues/6835), [#6841](https://github.com/ubugeeei-prod/vize/issues/6841)                                                                                                                         |
| 2 IR contracts | [#6836](https://github.com/ubugeeei-prod/vize/issues/6836)                                                                                                                                                                                                                                                                                                                                                                         | Typed embeds and pattern table             | [#6835](https://github.com/ubugeeei-prod/vize/issues/6835), [#6841](https://github.com/ubugeeei-prod/vize/issues/6841)                                                                                                                         |
| 2 IR contracts | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838)                                                                                                                                                                                                                                                                                                                                                                         | L2 canonical artifact                      | [#6836](https://github.com/ubugeeei-prod/vize/issues/6836)                                                                                                                                                                                     |
| 2 IR contracts | [#6871](https://github.com/ubugeeei-prod/vize/issues/6871)                                                                                                                                                                                                                                                                                                                                                                         | Semantic query API                         | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838)                                                                                                                                                                                     |
| 2 IR contracts | [#6855](https://github.com/ubugeeei-prod/vize/issues/6855), [#6856](https://github.com/ubugeeei-prod/vize/issues/6856), [#6857](https://github.com/ubugeeei-prod/vize/issues/6857)                                                                                                                                                                                                                                                 | Neutral-vocabulary designs                 | — (anytime)                                                                                                                                                                                                                                    |
| 2 IR contracts | [#6844](https://github.com/ubugeeei-prod/vize/issues/6844)                                                                                                                                                                                                                                                                                                                                                                         | Native script analysis                     | [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), [#6837](https://github.com/ubugeeei-prod/vize/issues/6837)                                                                                                                         |
| 3 Backends     | [#6839](https://github.com/ubugeeei-prod/vize/issues/6839)                                                                                                                                                                                                                                                                                                                                                                         | L3 decision layer                          | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838), [#6868](https://github.com/ubugeeei-prod/vize/issues/6868)                                                                                                                         |
| 3 Backends     | [#6842](https://github.com/ubugeeei-prod/vize/issues/6842)                                                                                                                                                                                                                                                                                                                                                                         | Vue 0.x/1.x and quirks                     | [#6841](https://github.com/ubugeeei-prod/vize/issues/6841), [#6836](https://github.com/ubugeeei-prod/vize/issues/6836)                                                                                                                         |
| 3 Backends     | [#6840](https://github.com/ubugeeei-prod/vize/issues/6840) `write/`, `module/`                                                                                                                                                                                                                                                                                                                                                     | L4 writer and module assembly              | [#6832](https://github.com/ubugeeei-prod/vize/issues/6832)                                                                                                                                                                                     |
| 3 Backends     | [#6840](https://github.com/ubugeeei-prod/vize/issues/6840) DOM move                                                                                                                                                                                                                                                                                                                                                                | DOM target                                 | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838)                                                                                                                                                                                     |
| 3 Backends     | [#6840](https://github.com/ubugeeei-prod/vize/issues/6840) SSR                                                                                                                                                                                                                                                                                                                                                                     | SSR target                                 | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838), [#6839](https://github.com/ubugeeei-prod/vize/issues/6839)                                                                                                                         |
| 3 Backends     | [#6840](https://github.com/ubugeeei-prod/vize/issues/6840) Vapor                                                                                                                                                                                                                                                                                                                                                                   | Vapor target                               | [#6839](https://github.com/ubugeeei-prod/vize/issues/6839)                                                                                                                                                                                     |
| Products       | [#6847](https://github.com/ubugeeei-prod/vize/issues/6847), [#6875](https://github.com/ubugeeei-prod/vize/issues/6875), [#6876](https://github.com/ubugeeei-prod/vize/issues/6876)                                                                                                                                                                                                                                                 | Formatter from L1, Doc IR, span edits      | [#6836](https://github.com/ubugeeei-prod/vize/issues/6836)                                                                                                                                                                                     |
| Products       | [#6848](https://github.com/ubugeeei-prod/vize/issues/6848)                                                                                                                                                                                                                                                                                                                                                                         | Linter on L1/L2                            | [#6871](https://github.com/ubugeeei-prod/vize/issues/6871)                                                                                                                                                                                     |
| Products       | [#6850](https://github.com/ubugeeei-prod/vize/issues/6850), [#6872](https://github.com/ubugeeei-prod/vize/issues/6872)                                                                                                                                                                                                                                                                                                             | LSP on level artifacts, LSP state          | [#6838](https://github.com/ubugeeei-prod/vize/issues/6838), [#6871](https://github.com/ubugeeei-prod/vize/issues/6871)                                                                                                                         |
| Products       | [#6873](https://github.com/ubugeeei-prod/vize/issues/6873)                                                                                                                                                                                                                                                                                                                                                                         | Cancellation                               | [#6872](https://github.com/ubugeeei-prod/vize/issues/6872)                                                                                                                                                                                     |
| Products       | [#6849](https://github.com/ubugeeei-prod/vize/issues/6849)                                                                                                                                                                                                                                                                                                                                                                         | Type checker via L4 `ts`                   | [#6840](https://github.com/ubugeeei-prod/vize/issues/6840)                                                                                                                                                                                     |
| Products       | [#6851](https://github.com/ubugeeei-prod/vize/issues/6851)                                                                                                                                                                                                                                                                                                                                                                         | Dependency gate for products               | [#6847](https://github.com/ubugeeei-prod/vize/issues/6847), [#6848](https://github.com/ubugeeei-prod/vize/issues/6848), [#6849](https://github.com/ubugeeei-prod/vize/issues/6849), [#6850](https://github.com/ubugeeei-prod/vize/issues/6850) |
| After Vue Fes  | [#6858](https://github.com/ubugeeei-prod/vize/issues/6858)                                                                                                                                                                                                                                                                                                                                                                         | TSRX (nice to have before; not a priority) | —                                                                                                                                                                                                                                              |
| After Vue Fes  | [#6859](https://github.com/ubugeeei-prod/vize/issues/6859)                                                                                                                                                                                                                                                                                                                                                                         | Solid                                      | —                                                                                                                                                                                                                                              |
| After Vue Fes  | [#6860](https://github.com/ubugeeei-prod/vize/issues/6860)                                                                                                                                                                                                                                                                                                                                                                         | Flow                                       | —                                                                                                                                                                                                                                              |
