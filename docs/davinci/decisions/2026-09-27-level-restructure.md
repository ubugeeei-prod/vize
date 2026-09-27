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

Tracking issues: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826) (restructure), [#6827](https://github.com/ubugeeei-prod/vize/issues/6827) (products), [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) (legacy deletion), [#6829](https://github.com/ubugeeei-prod/vize/issues/6829) (multi-framework), [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) (CI). [SSR option compatibility](./2026-09-27-ssr-source-compatibility.md).

## Levels and naming

- The stages are renamed **L0–L4** (levels). The rename is in progress.
- **Crates are named by level only**: `vize_l0` … `vize_l4`, and conversion crates `vize_l1_to_l2` and `vize_l2_to_l3`. The one exception is `vize_l0_derive`, because proc-macro crates must be separate.
- There are no functional crate names (no `vize_folio`, `vize_dialect`, …). Functional concerns are modules inside a level crate.
- **Codenames are aliases only.** Davinci, Sinopia, Disegno, Ricalco, Impeto, Folio, Spolvero and the rest may appear in crate docs (`//! L3 — reactivity IR (codename: Impeto)`) and in `docs/`. They never appear in crate, directory, file, module or type names, or in serialized strings. Ordinary terms such as `lattice` and `ledger` are not codenames.
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

- Davinci has no external users yet, so its crates, APIs, dump formats, serialized strings and feature names change without a compatibility period. Legacy products keep strict output compatibility; the [#6898 SSR slot scope correction](./2026-09-27-ssr-slot-scope.md) adds its own regression corpus.
- Product crates with art names (croquis, patina, glyph, maestro, canon, carton, …) keep their names. `docs/davinci/` keeps its name as the program name.
- The migration does not freeze other work. Move-only commits keep git rename detection working for in-flight fixes, renames are scripted (on a conflict, re-run the script on `main`), and PRs stay small.

## Skeleton-first (2026-09-28)

Decided by the maintainer on 2026-09-28 ([#6826 comment](https://github.com/ubugeeei-prod/vize/issues/6826#issuecomment-5857956448)).

- **The whole L0–L4 picture is laid down as a compiling skeleton first.** Unimplemented bodies are `todo!()` inside modules that start with `#![expect(clippy::todo, reason = "skeleton: #NNNN")]`; the workspace still denies `clippy::todo` and `#[allow]` everywhere else. Skeleton code is not reachable from any product path.
- **A CI ratchet counts those expectations per crate.** `tools/commands/ci/check-skeleton-todos.rs --check` (clippy job and `check:rust`) compares them with `tools/config/skeleton-todos.toml`; counts only go down, and a PR that fills a module lowers its line (`--write`).
- **Priority:** L0 (`vize_l0`, [#6833](https://github.com/ubugeeei-prod/vize/issues/6833)), the L1 lexer (`vize_l1::markup`), L4 (`vize_l4`).
- **`vize_l0` starts as its own package with no dependents.** Its modules are `id`, `side_table`, `key`, `dump` (trait, values, runtime), `diag` (diagnostic, witness), `pass` (manager; remarks, fusion and timing observers), `fact`, `level` (L0–L4 and the `vize_l1_to_l2`/`vize_l2_to_l3` conversions), `span`, `source`, `arena` and a doc-only `extension` placeholder (the neutral wire stays in `vize_carton::extension` per [#6976](https://github.com/ubugeeei-prod/vize/pull/6976) and moves with [#6834](https://github.com/ubugeeei-prod/vize/issues/6834)). It is `publish = false` until it replaces the `vize_l0` workspace alias, which still points at `vize_carton` until #6834.
- **"Lightweight Davinci" means runtime cost and process weight.** Runtime: instruction counts, zero-cost unobserved paths, no reparses or serialization between levels ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868), [#6869](https://github.com/ubugeeei-prod/vize/issues/6869)). Process: no committed whole-repo ledgers, shorter PR-tier CI, short PR bodies and issue comments, no hash- or evidence-heavy verifiers where a normal test does.
- **The dump/naming stack #6906 → #6907 → #6943 → #6944 → #6945 → #6946 → #6947 is collapsed into one PR** on `main`; the seven PRs are closed in its favour.

## Level dump naming

The L3 package identity, stage feed types, level dump API, versioned dump protocols and `vize dump --roundtrip` are recorded in [level dump naming](./2026-09-28-level-dump-naming.md).

## `vize_davinci` is deleted

Tracked in [#6833](https://github.com/ubugeeei-prod/vize/issues/6833) and [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

- **`vize_l0`** is the foundation level. It holds source, arena, span, ids (`NodeId`, `AnalysisId`), side tables, artifact keys, the dump trait and runtime, diagnostics and witnesses, the pass and fact managers, and the level registry. It is carved out of `vize_carton`, which keeps config, i18n, LSP, resolver and profiler code.
- Some code moves to the crate that uses it:
  - `summary` → `vize_l2::summary`
  - `render` → the `vize` CLI
  - the Croquis dump page → croquis
  - the stage feed → curator
  - the repro page → the CLI
  - `legacy_plan` → test support
- The accepted trade-off is that `vize_l0` becomes large. It stays readable through its modules.

## Dependency direction

Tracked in [#6831](https://github.com/ubugeeei-prod/vize/issues/6831) and [#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

- Level crates never take normal dependencies on legacy crates: `vize_armature`, `vize_relief`, `vize_atelier_*`, `vize_croquis`, `vize_croquis_cf`. Legacy may depend on levels. Dev-dependencies used as differential oracles are fine.
- **Croquis counts as legacy.** Script analysis is rebuilt natively in the levels ([#6844](https://github.com/ubugeeei-prod/vize/issues/6844)). No adapter presents legacy output as Davinci facts.
- A `cargo metadata` gate enforces the rule. It starts with a shrinking allowlist and later covers product crates too.
- The [declaration ratchet](./2026-09-27-foundation-stack-replay.md#dependency-gate)
  records shrinking #6831 permissions, enforcement and remaining scope.

## L1: what the text _is_

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835), [#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and [#6837](https://github.com/ubugeeei-prod/vize/issues/6837). The detailed design is in the [#6836 design comment](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).

[Authored formatter tag forms](./2026-09-27-formatter-tag-forms.md) records
#6846's resolution-free audit and public byte corpus; source-built Actions
and the actual queue remain required before closing the issue.

- **Boundary rule:** L1 is what the text _is_ (the concrete syntax of every grammar, lossless). L2 is what it _does_.
- **Markup** is a grammar × profile matrix:
  - Grammars: Vue now; Svelte, Angular and others later.
  - Profiles: `document` (HTML and in-DOM rules, used by petite-vue) and `component` (SFC template rules).
  - One shared lexer, `Lexer<P: Profile>`, with static dispatch.
  - The template tokenizer moves from armature into `vize_l1::markup`, so armature depends on L1 and not the other way round.
- **Container** is the file-format layer. SFC block splitting moves here out of Croquis, laid out so Svelte, Analog and TSRX containers fit later.
- **Dialect syntax hooks** decompose directive names (`v-on:click.stop`, `@click`, `#default`, `:[dyn]`). They play the role of MLIR custom assembly formats.
- **Typed embeds.** Attribute values, mustaches and dynamic arguments become `Embed { grammar, source }` with `Grammar = Shape × Lang`:
  - `Shape` is the role inside the markup: `Expr`, `HandlerBody`, `ForHead`, `SlotParams`, `FilterChain`, …
  - `Lang` is the host language, resolved once per file: JS and TS now; Flow, MoonBit and others later.
  - Composite shapes (`ForHead`, `FilterChain`) are built by the dialect from language pieces.
- **Embed trees** are a separate L1 artifact keyed by embed id:
  - The JS/TS tree is the oxc AST plus spans; the source bytes are authoritative.
  - A broken expression becomes a hole in its own tree and never breaks the markup tree.
  - Embedding works in both directions and nests (markup inside script for JSX, TSRX and Angular inline templates).
- **Entity decoding** happens when an embed is built. The embed carries the decoded text and a decode map, so language providers never see HTML.
- **`v-pre`** is handled in L1, because it switches the lexing mode of its children.
- **Language providers** are bundles of optional capabilities, one per level (like MLIR interfaces):
  - L1 syntax entry points
  - L2 semantics (today's `ExprDialect`)
  - L4 emission, the type-check projection and the checker host
  - the formatter printer

  A missing capability falls back to the opaque path or a diagnostic. There is one dynamic dispatch per file; everything after it is static. Template expressions follow `<script lang>`, and a language mismatch between `<script>` and `<script setup>` is a diagnostic.

- The [markup and container skeleton](./2026-09-28-l1-markup-skeleton.md) records the `Profile`, `Sink`, lex error and directive-hook shapes.
- Two decisions from the same design comment belong to the next section: the `const` pattern table of `fn` pointers, and parsing each expression once (L4 rewrites from the L2 identifier-resolution table).

[The explicit L1 embed source skeleton](./2026-09-28-l1-embed-source-skeleton.md) records the maintainer's code-first request for #6836; source preparation remains `todo!()`, with typed trees, language resolution and dialect hooks unfinished.

## L1→L2 and L2

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and [#6838](https://github.com/ubugeeei-prod/vize/issues/6838).

- L1→L2 is a **table of conversion patterns**: each dialect exposes a `const` slice of `fn` pointers. Unconverted input becomes a diagnostic, so lowering stays total.
- **Expressions are parsed once, in L1.** L2 records identifier resolution (setup, props, ctx, local, …) in a side table. L4 rewrites by span from that table. The DOM emitter's re-parses for `_ctx.` prefixing go away.
- `vize_l2` owns the canonical artifact: ops plus semantic side tables. Transform passes move into `vize_l2`. Dialect-derived tables (`wrappers`, `legacy`) are absorbed by legalization and do not appear in the artifact.

## L3 is the decision layer

See the [l3 is the decision layer decisions](./2026-09-27-level-restructure-designs.md#l3-is-the-decision-layer) in the companion record.

The [native decision skeleton](../plan/native-decision-skeleton.md) records
[#6839's paired decision](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5857255718) records the 2026-09-28 architecture-first follow-up:
L3 owns L2-node decision types and target policy identities; the borrowed
L2→L3 producer is an explicit `todo!()`, without a production caller.
Current analysis, flat-program demand splitting and L4 extraction remain
unfinished; no product coverage or performance acceptance is claimed.
The first source Actions run typechecked the skeleton and passed all four
Rust test workers; tooling had 4,648 passes, 12 skips and one missing storage
inventory row. The reviewed new row records one owned `Vec<NodeId>` import
and one bound use; all prior inventory rows, runtime budgets and gates stay
unchanged. The fresh source and merge-queue checks remain required.

## L4: emission

See the [l4: emission decisions](./2026-09-27-level-restructure-designs.md#l4-emission) in the companion record.

[The `vize_l4` skeleton](./2026-09-28-l4-skeleton.md) records the crate layout (`write`, `expr`, `runtime`, `module`, `targets/{dom,ssr,vapor,ts}`), the compile-time link sink, and the L4 codename Stesura; target, expression and module bodies remain `todo!()`.

[Compiler fix-history byte references](./2026-09-27-compiler-fix-history-pins.md) records [#6880](https://github.com/ubugeeei-prod/vize/issues/6880): three immutable complete SFC Results, including #1416 diagnostics. The earlier ten-profile archive has eight fixes and two feature controls; whole history/native acceptance remain unfinished and fresh Actions are required. Publication uses ordinary shared test modules, a documented unused-helper expectation verified by strict Clippy, and a generated SFC consumer shard after Actions found policy drift.

[SSR and Vapor history witness audit](./2026-09-27-compiler-target-history-audit.md) records 19 SSR and 20 Vapor inspected fixes, original option boundaries and raw byte gaps. Preserve normalized complete Pkl outputs and all shape/runtime checks alongside new raw pins.

[Complete SSR fix-history capture](./2026-09-27-ssr-fix-history-captures.md) records nine repeated outputs for #990/#3701/#2487 with source/executable identity and exact options. The frozen capture source is preserved on its provenance branch; fresh Actions, whole history and native acceptance remain pending. The SSR consumer shard is regenerated for the new test helper.

[Complete Vapor fix-history capture](./2026-09-27-vapor-fix-history-captures.md) records nine repeated complete results for seven fixes with exact options and source/executable identity. Its frozen source is preserved on a provenance branch; whole history, native acceptance and fresh Actions remain pending. Publication uses ordinary test/example modules and the generated Vapor consumer shard.

[SFC map and diagnostic history references](./2026-09-27-sfc-map-diagnostic-history.md) records two original #3604/#750 inputs and three repeated source/executable-bound Results, with a populated map and distinct relative/document diagnostic locations. The complete test and strict Clippy pass locally; fresh Actions, whole history and native acceptance remain TODO. Its reviewed frozen source is preserved on a provenance branch, and the bounded SFC consumer shard records the new helper references.

[Existing compiler stack acceptance](./2026-09-27-sfc-map-diagnostic-history.md#existing-stack-merge-acceptance) records actual CI4 main, the sole inspector binding, fresh exact-head checks and native queue/actual-merge requirements for the existing five PRs; 23 compiler inputs and 15 fix links leave whole history/native acceptance unfinished.

## Script side

See the [script side decisions](./2026-09-27-level-restructure-designs.md#script-side) in the companion record.

## Dialects, languages, frameworks

See the [dialects, languages, frameworks decisions](./2026-09-27-level-restructure-designs.md#dialects-languages-frameworks) in the companion record.

[Legacy formatter Vue 2 filters](./2026-09-27-glyph-vue2-filters.md) records [#6845](https://github.com/ubugeeei-prod/vize/issues/6845), explicit version selection and the native FilterChain follow-up in #6836.

## JSX semantics

See the [JSX semantics decisions](./2026-09-27-level-restructure-designs.md#jsx-semantics) in the companion record.

## Products on the levels

Tracked in [#6827](https://github.com/ubugeeei-prod/vize/issues/6827) and [#6845](https://github.com/ubugeeei-prod/vize/issues/6845)–[#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

- One parse per file. Every product consumes the same artifacts.
- **Formatter:** L1 only. A rewrite whose safety depends on L2 facts (for example component-dependent self-closing) is a linter autofix instead.

  [Formatter fix-history output fixtures](./2026-09-27-formatter-fix-history.md) records [#6882](https://github.com/ubugeeei-prod/vize/issues/6882); public script and CSS byte fixtures retain the actual invalid-CSS error separately. Full-history audit and shared corpus registration remain pending.
  Formatter history asset and binary-reference bytes disable Git text conversion, including on CRLF checkouts. Binary references retain authored trailing whitespace and EOF spaces.
  Legacy internal single-pass outputs remain API observations; native formatting does not inherit that mechanism. Product compatibility separately compares CLI check verdicts and streams.
  Formatter byte snapshots replace duplicated partial checks in new history tests; original semantic regressions and measured golden/capture bytes remain unchanged.
  Formatter slot/root/raw-close fixtures retain public dynamic barriers and final-CR/LF behavior, with bound-slot helper classification checked separately; shared corpus registration remains pending. The glyph import inventory row travels with that fixture slice.
  Formatter raw/entity fixtures preserve deleted authored inputs and reuse duplicate witnesses. Legacy internal check-mode intermediate output does not prescribe a native pipeline stage; compare CLI verdict/streams separately.
  Formatter SFC layout fixtures retain six exact original inputs/defaults and compare full first-pass bytes alongside the existing three-pass fixed-point checks.
  Formatter directive layout fixtures retain six original inputs/options and semantic constraints; complete existing template-literal witnesses are reused without duplicate credit.

  Existing formatter regressions gain twenty-four full byte references while retaining their semantic/fixed-point assertions; opaque CRLF templates and script identity keep exact output bytes. Shared registration remains pending.
  The shared public formatter API observer builds in T1 from the actual checkout, rejects untracked Rust product sources, retains receipt/stream artifacts through one shared composite upload action, counts four default byte/fixed-point cases and two legacy internal observations separately, and keeps native credit zero; other history and CLI checks remain pending.

- **Linter:** syntax rules on L1, semantic rules on L2 and facts. Diagnostics go through L0, and autofixes are L1 span edits.

  [#6881 linter history inventory](./2026-09-27-linter-history-inventory.md) pins full Git history and candidate snapshot blobs; enumeration never counts as fixture coverage or native acceptance.

  [Selected current linter history oracles](./2026-09-27-lint-history-current-api.md) record [#6881](https://github.com/ubugeeei-prod/vize/issues/6881): complete public API results and actual independent fixes, with whole-history closure and native comparisons still pending.

  [Static-class edit boundaries](./2026-09-27-static-class-fix-boundary.md) record [#6920](https://github.com/ubugeeei-prod/vize/issues/6920): use the end-exclusive binding span; preserve the original bad capture as evidence, not accepted coverage.

  [#6881](https://github.com/ubugeeei-prod/vize/issues/6881) queue validation exposed a 1,053,102-byte tracked filename stream exceeding Node's default 1 MiB buffer. The v-on and WASM-cache inspections share an 8 MiB bounded Git reader that rejects child errors and nonzero exits, preserves NUL-delimited filenames, and checks a real index above 1 MiB plus overflow and command failure. All historical captures and assertions remain unchanged; the remaining three PRs still require fresh source and full queue checks before actual merge.

  [Actual historical linter reports](./2026-09-27-lint-report-history.md) record [#6881](https://github.com/ubugeeei-prod/vize/issues/6881): complete JSON/Text bytes and unchanged requery controls, with the existing linter witness shard refreshed; full history and native proof remain open.

  [NextTick arrow history oracles](./2026-09-27-next-tick-history.md) record [#6881](https://github.com/ubugeeei-prod/vize/issues/6881): ten exact private source/public API observations, including Unicode/CRLF SFC framing, complete diagnostics and unchanged requery, with one linter witness row refreshed; full history and native proof remain open.

- **Type checker:** the virtual-TS projection is an L4 target mapped back through `EmitDocument` links.
- **LSP:** holds level artifacts incrementally and never parses by itself.

## Type check

See the [type check decisions](./2026-09-27-level-restructure-designs.md#type-check) in the companion record.

[Type-checker history slices](./2026-09-27-typechecker-history-slices.md) record
focused required T1 diagnostics, including component tuples, for #6879;
whole-history and native admission remain unfinished; the companion preserves every row.

## Legacy deletion criteria

See the [legacy deletion criteria decisions](./2026-09-27-level-restructure-history.md#legacy-deletion-criteria)
in the companion record and the [LSP fix-history response-fixture installment](./2026-09-27-lsp-fix-history.md)
for the pinned review ledger, complete document-link and CRLF on-type response contracts and
unfinished history obligations in #6883.
The same installment preserves dated source-runtime receipts separately from
historical binary evidence; later heads still require their own Actions proof.

## Multi-framework

Tracked in [#6829](https://github.com/ubugeeei-prod/vize/issues/6829) and [#6855](https://github.com/ubugeeei-prod/vize/issues/6855)–[#6860](https://github.com/ubugeeei-prod/vize/issues/6860).

- This supersedes charter row #1. Other frameworks are in-tree and include the compiler, with parity against each reference compiler.
- The order is **TSRX → Solid → others** (Svelte, Angular/Analog).
- Five independent axes: container, markup, profile, lang, framework.
- L2 keeps neutral core ops plus framework dialect ops.
- L3 gets a neutral reactivity vocabulary (signal, derived, effect, ordering).
- L4 gets one target per framework runtime.

## Vue Fes Japan (2026-10-24)

- **In scope:** Vue (templates and every Vue dialect), JS/TS, JSX/TSX.
- **TSRX:** not a priority; nice to have if something runs.
- **Other frameworks:** only the neutral types and names are designed before the talk. Flow is not in scope at all.
- **Goal (A):** for the in-scope inputs, all five products run on the shared L1/L2 structure as far as native work goes, with per-product native-only acceptance rates.
- **Deletion (B)** follows later, under the criteria above.
- Unfinished work is reported as unfinished. There are no legacy-backed shortcuts.

## Performance

The toolchain aims to be extremely fast. The layering must not add pipelines or serialization cost.

- **Levels are type boundaries, not runtime boundaries.**
  - There is no serialization between levels; dumps exist only for `vize dump` and observers.
  - All artifacts share one arena and use dense `u32` ids.
  - Passes are fused into single walks where possible.
  - Anything unobserved costs nothing (generic observers; entity decoding only when `&` is present; incremental keys only in resident mode).
- **Regressions are stopped in the merge queue** with instruction-count measurements per stage ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868)). The wall-clock envelope runs nightly.
- **Per-stage budgets ratchet from current measurements.** Today all 102 `wall_p50_ns` entries in `plan/budgets.toml` are unset.
- The [instruction-count gate record](./2026-09-27-instruction-counts.md) defines measured-only ceilings and immutable-base ratchets for #6868. Independent clean Actions builds (run 36307058591, attempts 1 and 2) match all 100 probes in three executions each under the fixed guest method. Required `test-report` aggregates queue measurement and strict ceilings. Separate queue tests and test-inventory collection preserve its check name and satisfy the source-length ratchet. Exact queue verification is pending.
- **`SideTable` stays `FxHashMap`** ([#6869](https://github.com/ubugeeei-prod/vize/issues/6869), measured 2026-09-28).
  - Density over the repo's 1,419 `.vue` templates (18,283 L2 ops): `static_facts` holds 0.33 facts per op. Every other table holds under 0.05, and `if_facts`/`model_faults` under 0.001.
  - A lookup costs about 2.9 ns hashed and 1.4 ns in a dense `Vec`. The DOM emitter's static-fact reads would save well under 1% of a compile.
  - No table meets the densification trigger in `side_table.rs` (majority occupancy). Re-measure if a table's occupancy passes one half.

## Toolchain practice

[External report attribution](../../../CONTRIBUTING.md#fix-requests) prioritizes reports
through regression, verified co-authors, actual CI/merge and public release verification.
[Release acknowledgements](../../../.github/release-notes/drop-in-scope.md#reporter-acknowledgements)
credit nine reports already shipped in v0.429.1 without rewriting fix/tag history;
the four documented workspace-only experiments remain unpublished.

Vize follows language-toolchain practice, not compiler-only practice. It stays lightweight and fast. It avoids the heaviness of rust-analyzer-style designs: fine-grained per-node queries, per-node reference-counted trees and whole-workspace resident state.

- **Two tiers stay as they are.** Long-lived processes (LSP, check server, watch modes) use the salsa-based resident tier. The one-shot CLI uses the fused non-salsa pipeline.
- **One semantic query API over L2** serves every product ([#6871](https://github.com/ubugeeei-prod/vize/issues/6871)). See the [semantic query API design](./2026-09-27-level-restructure-designs.md#semantic-query-api-6871) in the companion record.
- **LSP state stays coarse** ([#6872](https://github.com/ubugeeei-prod/vize/issues/6872)):
  - Queries are per SFC block and per expression embed, not per node.
  - Node references never survive an edit. A position is resolved to a node on the latest snapshot through a span-sorted index.
  - Diagnostics and code actions carry a range and a document version, and they are recomputed when stale.
  - Memory holds artifacts only for open files, plus `SfcSummary` for every file.
- **Stale requests are cancelled on edit** ([#6873](https://github.com/ubugeeei-prod/vize/issues/6873)).
- **The CLI and the LSP share one project model** ([#6874](https://github.com/ubugeeei-prod/vize/issues/6874)).
- **The formatter keeps its Doc IR separate from its printer** ([#6875](https://github.com/ubugeeei-prod/vize/issues/6875)).
- **Edits have one representation.** Diagnostic fixes, code actions and lint autofixes are all L1 span edits tagged with a document version ([#6876](https://github.com/ubugeeei-prod/vize/issues/6876)).

[Inspector comparison transport](./2026-09-27-inspector-compare-transport.md)
preserves authoritative child failures when Node exits before input delivery.

## CI tiers

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and [#6861](https://github.com/ubugeeei-prod/vize/issues/6861)–[#6867](https://github.com/ubugeeei-prod/vize/issues/6867). [First-publish control repair](./2026-09-27-sdk-bootstrap-control.md) is tracked in [#6895](https://github.com/ubugeeei-prod/vize/issues/6895).

| Tier           | Runs              | Target                   | Content                                                                                                                                                                            |
| -------------- | ----------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T0 PR          | every push        | p50 ≤ 3 min, p90 ≤ 6 min | fmt, title-policy, clippy and tests for affected crates (nextest archive + shards), input-selected tooling tests                                                                   |
| T1 merge queue | once before merge | —                        | full workspace and tooling tests, differential corpus and acceptance gates, instruction-count performance gates, playground ([#6865 execution](./2026-09-27-merge-queue-gates.md)) |
| T2 nightly     | schedule          | —                        | E2E, real-project matrix, fuzz, miri, benchmarks, editor conformance, resource budgets                                                                                             |
| T3 release     | release           | —                        | everything, semver checks, release preflight                                                                                                                                       |

- zizmor runs only for external contributors, releases and PRs that touch `.github/**`. [Audit selection](./2026-09-27-ci-security-selection.md) and [inherited finding repair](./2026-09-27-workflow-security-refresh.md) record #6866.
- VRT, tsgo-required tests and ledger checks leave the PR tier. [Inventory tier implementation](./2026-09-27-ci-tier-inventories.md) records #6864.
- Whole-repo generated ledgers stop being committed. [Artifact generation](./2026-09-27-generated-ledgers.md) records #6867.
- [JS native preparation](./2026-09-27-js-native-preparation.md) keeps package coverage and reuses the root test build ([#6830](https://github.com/ubugeeei-prod/vize/issues/6830)).
- [UI check tiers](./2026-09-27-ui-check-ci-tiers.md) retain Fresco declarations/types in T0 and require UI acceptance in T1 ([#6864](https://github.com/ubugeeei-prod/vize/issues/6864)).

[Affected Rust selection](./2026-09-27-affected-rust-ci.md) records the fail-closed dependency plan and shell-free package execution for #6862.

[Tooling input selection](./2026-09-27-tooling-input-selection.md) records the shared task inputs, explicit T1 runtime inventory and conservative T0 fallback for [#6863](https://github.com/ubugeeei-prod/vize/issues/6863) and [#6864](https://github.com/ubugeeei-prod/vize/issues/6864).

[Full tooling LSP source identity](./2026-09-27-lsp-source-binding.md) binds
required runtime proof to the receipted current CLI without cached fallback.
Use test-step environment variables to retain the plain VP command and compose
with the formatter's always-upload corpus evidence without changing build setup.

[Stacked PR checks](./2026-09-27-stacked-pr-checks.md) run on every PR base. [Rust PR archive and shard execution](./2026-09-27-rust-nextest-shards.md) records #6862's runner, doctest, resource and archive identity decisions.

[Rust source wiring](./2026-09-27-ci-rust-core.md) preserves full queue execution, proves the tested comparison base and records the intermediate scope.

[CI baseline decisions](./2026-09-27-ci-baseline-decisions.md) preserve the Rust timings, first formatter path and stack replay evidence.

[Tooling tier wiring](./2026-09-27-ci-tooling-tier-core.md) records the shared comparison base, queue-only VRT and preserved complete tooling task.

[Heavy tooling inputs](./2026-09-27-heavy-tooling-inputs.md) records Cargo corpus,
Moon/benchmark dependencies and the immutable queue source-length base for #6863.

[Rust cache backends](./2026-09-27-rust-cache-backends.md) records #6830's bounded provider namespaces and actual partial trusted seed receipts; [nested post paths](./2026-09-27-rust-cache-backends.md#nested-cache-post-paths) records the reviewed missing-target repair. Nested target saving and later restore proof remain pending. Reviewed save eligibility for the two observed oversized roles keeps full validation and fixed provider mounts; actual full-run post omission remains pending.

## Order of work

See the [order of work decisions](./2026-09-27-level-restructure-order.md#order-of-work) in the companion record.

[Nuxt critical CSS module identity](./2026-09-27-nuxt-critical-css.md) records [#6897](https://github.com/ubugeeei-prod/vize/issues/6897); compiler migration and SSR slot scope remain separate.

- **Full queue Rust execution (#6830, #6861):** reuse one bound workspace archive and four unfiltered workers; keep required TSGO, doctests and all 11 feature recipes. Runtime evidence and acceptance conditions are in [the full Rust shard decision](./2026-09-27-ci-full-rust-shards.md).

[Options API computed regressions](./2026-09-27-options-computed-regressions.md)
record #6921 owner history, shared parse facts, setter guards and registered typechecker fixtures for #6879.

[Canon slot outlet regression preparation](./2026-09-27-canon-slot-outlet-union.md)
records #6922 owner history, unbounded inferred payloads, string widening and required full diagnostic proof.

[Extension wire ownership](./2026-09-28-extension-wire-ownership.md) starts the
actual L0 neutral-code move for #6833; retiring the three old packages remains unfinished.
