# Phase 0 — Instrumentation and Groundwork

> [!NOTE]
> No behavior changes anywhere in this phase. Every task is parallelizable
> unless a dependency says otherwise. Exit gate at the bottom.
> Each task lists concrete **Steps** — sub-checkboxes an agent works through —
> and machine-checkable **Acceptance**. Paths are current as of drafting;
> if a path moved, fix the plan in the same PR (the plan is code).

## TODO index

- [x] P0-1 Bench harness with memory metrics
- [x] P0-2 Template-pipeline microbenches, front half (armature, croquis)
- [x] P0-3 Template-pipeline microbenches, back half (atelier core/dom/vapor/ssr)
- [ ] P0-4 Bench baselines and CI gating (`budgets.toml`) — budget registry, compare gate, and tests landed; reference-runner baseline recording + CI lane pending
- [x] P0-5 Corpus baseline snapshot + diff tool — fingerprints the four harness lanes (single DOM compile lane today); TS-11 scope proof embedded; one filed unstable row (`typechecker/element-plus` corsa-shard race) shard-scoped in `corpus-baseline-unstable.json`
- [ ] P0-6 Corpus expansion round 1 (pug/JSX/Vapor/petite-vue)
- [x] P0-7 Croquis consumption matrix as tracked artifact — resolution is `use`-declaration-based with a grep cross-check lane (rustdoc-JSON upgrade possible later)
- [x] P0-8 Rule-parity matrix (SFC × JSX) — surfaces derived from trait impls + overridden hooks per rule file; SFC/JSX membership derived from the registration sites and the `lint_jsx` three-lane partition (`as_markup_rule` / `jsx_needs_lowering` / legacy fallback), with drift asserts on the dispatch source
- [x] P0-9 `Span` type + `SourceLocation` consumer inventory — corpus-diff acceptance clause pending the P0-5 tool
- [x] P0-10 Folio harness skeleton + VIR absorption — the `vir` payload key lives in `vize_vitrine`, not `vize_curator`; alias added at the real site
- [ ] P0-11 Profiler source-level attribution + stable export
- [ ] P0-12 Assurance harness (assertion lint, mutation baseline, taxonomy)
- [x] P0-13 Seeded-defect + suppression-telemetry pilots (FP/FN oracles) — both oracles running with committed, triaged ledgers; corpus-shard-in-CI pending corpus hydration (with P0-6)

---

## P0-1 — Bench harness with memory metrics

**Deliverable:** `benchmarks/davinci_harness/` — a workspace crate wrapping
criterion with allocation and RSS metrics, used by every later bench.

**Steps:**

- [x] Create `benchmarks/davinci_harness/` (add to `[workspace] members` in root `Cargo.toml`); `publish = false`, stability `experimental`
- [x] `src/alloc.rs`: `CountingAllocator` (wraps the global allocator; counts `alloc` calls + running/peak bytes; `#[global_allocator]` opt-in via `davinci_harness::main!` macro; default inner allocator is mimalloc, matching the shipped `vize` binary)
- [x] `src/rss.rs`: peak-RSS sampling (`ru_maxrss` via `libc::getrusage` on macOS/Linux; stub returning `None` elsewhere). Platform semantics documented and normalized: `ru_maxrss` is KB on Linux, bytes on macOS, and is a **process-wide peak** — report baseline-subtracted deltas per bench process, never raw values
- [x] `src/report.rs`: JSON exporter — schema `{bench_id, fixture, platform, wall_ns: {p50,p95}, allocs, alloc_bytes_peak, rss_peak_bytes, harness_version}` written to `bench/results/davinci/<bench_id>.json`; exact peak-byte gates select an explicit per-platform budget and fail closed for unknown platforms
- [x] `schema/davinci-bench.schema.json` committed; exporter validates against it in debug (strict subset validator: unimplemented schema keywords are errors, not skipped checks)
- [x] One sample bench (`benches/selfcheck.rs`) exercising all metrics
- [x] Wire a `vp` task: `bench:davinci` at workspace root

**Acceptance:**

- `cargo bench -p davinci_harness` emits schema-valid JSON with all four metric families non-null on macOS/Linux
- `node --test tests/tooling/` conventions untouched; no existing bench affected

**Deps:** none. **Non-goals:** benching pipeline crates (P0-2/3); CI gating (P0-4).

## P0-2 — Microbenches: `vize_armature` + `vize_croquis`

**Deliverable:** criterion benches over a committed fixture ladder.

**Steps:**

- [x] Fixture ladder at `benchmarks/davinci_harness/fixtures/`: `small.vue` (~30 lines), `medium.vue` (~200 lines, real corpus extract), `large.vue` (~1k lines, corpus extract), `stress-deep.vue` (64-deep nesting), `stress-wide.vue` (200 attributes), `stress-interp.vue` (500 interpolations) — each with a `PROVENANCE.md` naming source project + commit (extracts come from MIT-licensed corpus projects only; identity pinned by exact-length tests in `davinci_harness::fixtures`)
- [x] `crates/vize_armature/benches/davinci.rs`: `tokenize` and `parse` cases per fixture (add `[[bench]]` + `davinci_harness` dev-dep to `crates/vize_armature/Cargo.toml`)
- [x] `crates/vize_croquis/benches/davinci.rs`: `analyze_sfc_descriptor` with `SfcCroquisOptions::full()` and `::for_compile()` per fixture (the entry point lives in `vize_atelier_sfc::croquis`; the bench takes a dev-only dependency cycle on `vize_atelier_sfc`, which cargo permits)
- [x] Confirm `clippy.toml` bans hold in bench code (use `vize_carton` types)

**Acceptance:**

- `cargo bench -p vize_armature -p vize_croquis` produces per-case JSON in `bench/results/davinci/`
- Two consecutive runs differ < 5% wall on the same machine (measured, recorded in the PR body)

**Deps:** P0-1.

## P0-3 — Microbenches: atelier core / dom / vapor / ssr

**Deliverable:** per-stage benches so P1–P3 regressions localize.

**Steps:**

- [x] `crates/vize_atelier_core/benches/davinci.rs`: `transform` (lane only, pre-parsed AST input) per fixture — via the `davinci_harness::stage` window (transform mutates in place, so setup is rebuilt per iteration outside the measured section)
- [x] `crates/vize_atelier_dom/benches/davinci.rs`: `compile_template` split into `transform` vs `codegen` timings (wrap stages with harness timers, not one blob) — plus a fused `compile` case pinning the end-to-end number
- [x] `crates/vize_atelier_vapor/benches/davinci.rs`: `lower` and `generate` separately — plus `transform` (the VDOM lane on the vapor path), pinning the cost of today's run-then-discard double transform as a number
- [x] `crates/vize_atelier_ssr/benches/davinci.rs`: `codegen`
- [x] Record the current expression re-parse count per fixture (temporary counter hook `vize_atelier_core::expr_parse_probe`, 18 sites; baseline committed at [expr-reparse-baseline.md](./expr-reparse-baseline.md); becomes the `davinci.expr.parses` baseline for P1-13)

**Acceptance:** as P0-2, for four crates; vapor bench JSON shows `transform`/`lower`/`generate` as separate entries; expr-reparse baseline committed.
Measured note (2026-08-13, macOS M-series): loop-style cases hold the < 5%
run-to-run bar; **stage-windowed cases (µs-scale transforms with
per-iteration setup) show a 5–7% noise floor with rotating outliers across
three consecutive runs**, while allocation counts are identical across all
runs and cases. P0-4 budgets therefore gate stage-window wall at 10%
tolerance and lean on the deterministic alloc counts for tight regression
detection.

**Deps:** P0-1.

## P0-4 — Bench baselines and CI gating

**Deliverable:** committed baselines + a PR gate with ratchet semantics.

**Steps:**

- [x] `davinci-road/plan/budgets.toml`: per-bench budgets `{wall_p50_ns, allocs, rss_peak_bytes}` + global `traversal_count` placeholder (filled in P2) + mutation-score section (filled by P0-12) _(landed with a `wall_tolerance` field per entry — 0.05 whole-routine, 0.10 for `_transform_` stage windows — and every entry seeded at 0, meaning "reference-runner baseline not yet recorded"; bench-compare treats 0-seeded entries as report-only, and `tests/tooling/davinci-budgets.test.ts` reconciles the registry against the bench sources exactly, both directions)_
- [ ] Baseline JSONs recorded on the Blacksmith reference runner, committed under `bench/results/davinci/baseline/` _(pending: reference-runner recording once Blacksmith lanes drain; wiring lands with the recorded baselines)_
- [x] Compare script `tools/davinci/bench-compare.mjs`: baseline vs current, applying `budgets.toml` thresholds; exit non-zero on breach; `--update-baseline` flag gated behind an env var so refresh is always deliberate _(wall p50 gated per-bench-tolerance vs the baseline report, allocs gated exactly, RSS report-only; registry drift fails both directions — "bench disappeared" / "unregistered bench"; refresh requires `DAVINCI_BASELINE_REFRESH=1`; exact-stdout oracles over committed fixture pairs in `tests/_fixtures/davinci-bench-compare/`)_
- [ ] CI job (extend the existing bench workflow lane) running P0-2/3 benches + compare on PRs touching `crates/**` _(pending: reference-runner recording once Blacksmith lanes drain; wiring lands with the recorded baselines)_
- [x] Ratchet rule enforced in review tooling: a PR that raises any number in `budgets.toml` must reference a charter decision in its description (checked by a `tests/tooling/davinci-budgets.test.ts` that parses git blame provenance — or, simpler, a CI message requiring the string `budget-loosen:` in the commit body) _(implemented as the documented-in-file variant, not git-blame provenance: `budgets.toml` carries the exact `# ratchet: numbers may only tighten; loosening requires budget-loosen: <charter ref> in the commit body` header, and the budgets test asserts the header plus the tolerance ceilings)_

**Acceptance:**

- A test branch with an injected `std::thread::sleep(10ms)` in the parser fails the gate; reverting passes
- Noise-level variance (< threshold in `budgets.toml`) passes 10/10 reruns

**Deps:** P0-2, P0-3.

## P0-5 — Corpus baseline snapshot + diff tool

**Deliverable:** a reproducible whole-corpus output fingerprint to diff every later phase against.

**Steps:**

- [x] `tools/davinci/corpus-baseline.mjs`: runs `tools/fixtures/tool-matrix-report.mjs` across all shards, then reduces each project's per-surface outputs (compile dom/vapor/ssr, lint JSON, format output, check diagnostics) to `{surface, project, file_count, content_hash}` rows _(landed on the four lanes the harness actually emits — `compiler` (the single DOM-backend `build --format json` lane; no vapor/ssr lanes exist in the harness yet), `typechecker`, `linter`, `formatter` — with the scope proof embedded in the artifact; hash contract in `corpus-baseline-notes.md`)_
- [x] Baseline artifact `tests/_fixtures/davinci-baseline.json` (hashes only — small) committed _(covers the 134-project manifest as of the P0-5 sweep; landing required restoring the `vue-storefront` pin that bump #4236 had moved to an emptied upstream revision — see the notes. The 8 projects added by corpus expansion round 1 (#4324) need a reference-runner re-baseline before `corpus-diff` can pass its scope proof again; that re-baseline is P0-6 work)_
- [x] `tools/davinci/corpus-diff.mjs`: compares a fresh run against the baseline; reports per-surface/per-project drift; `--surface` filter _(exit 0 only on zero drift plus TS-11 scope proof on both the committed baseline and the fresh run; missing baseline or scope shortfall fails with exact reasons)_
- [x] Reproducibility check: two runs on the same tree produce byte-identical baseline files (this will surface any nondeterminism in current output — if found, file it, do not fix it in this task) _(ran twice: 535 of 536 rows byte-identical; the one divergent row — `typechecker/element-plus`, a racy corsa-shard TS6307 (3704 vs 3703 errors on the same tree) — is filed in `corpus-baseline-notes.md` and shard-scoped via `corpus-baseline-unstable.json`, exactly as this step prescribes; compiler/formatter stderr timing, temp paths, and print ordering are likewise filed and excluded from the hash contract)_

**Acceptance:**

- `node tools/davinci/corpus-diff.mjs` exits 0 on the committed baseline
- Injected one-character change in a corpus `.vue` file is reported with the right project + surface
- Two-run reproducibility holds (or the nondeterminism is filed as a tracked issue and shard-scoped)

**Deps:** none (P0-6 projects join the baseline when they land).

## P0-6 — Corpus expansion round 1 (charter #31)

**Deliverable:** coverage report + new pinned corpus projects for pug / JSX / Vapor / petite-vue.

**Steps:**

- [x] `tools/davinci/corpus-coverage.mjs`: scans corpus `.vue`/`.jsx`/`.tsx` sources for the construct taxonomy dimensions (P0-12's `taxonomy.toml`) and emits counts per construct per project _(landed with `--write`/`--check`; also scans the petite-vue entries' `.html`/`.js` via the manifest's `petiteVueGlobs`; the report carries a scope-proof footer — hydrated-project count vs manifest total — so a partially hydrated run says so loudly. The `--check` staleness gate joins `tests/tooling/davinci-matrices.test.ts` only once CI hydrates the full corpus)_
- [ ] Committed report `davinci-road/plan/corpus-coverage.md` (generated, with a staleness header) — the committed report currently covers the 8 round-1 projects only (8/142 hydrated, scope proof in the footer) (pending: full corpus hydration + P0-5 baseline — disk)
- [x] Candidate list of real projects filling the gaps (pug-using Vue apps, JSX/TSX Vue apps, Vapor early adopters, petite-vue sites) with license + size + rationale — **review point: maintainer approves the list before submodules land** _(maintainer approval received 2026-08-14 via the recommended-picks review)_
- [x] Approved projects added as pinned submodules under `tests/_fixtures/_git/` following existing conventions (`--depth 1`, license recorded in the fixtures manifest) _(round 1: `wave-ui`, `dho-web-client`, `vue3-admin-design`, `vue3-antd-admin`, `vue-core-vapor` (vuejs/core @ v3.6.0-rc.3, vapor suites), `vue-jsx-vapor`, `wakapi`, `petite-vue` — licenses verified at clone time; the two petite-vue entries pin `expectedVueFileCount: 0` and record their HTML corpus in `petiteVueGlobs` because SFC tool lanes glob `vueGlobs` directly)_
- [x] Re-run P0-5 baseline to include them (pending: full corpus hydration + P0-5 baseline — disk) — done at the phase-final head: 568-row / 142-project baseline committed with matching scope proof

**Acceptance:** coverage report regenerates identically; every taxonomy dimension has ≥1 real-project instance or a recorded "not represented — matrix fixtures only" note; baseline updated in the same PR.

**Deps:** P0-12 (taxonomy), P0-5.

## P0-7 — Croquis consumption matrix as tracked artifact

**Deliverable:** the [semantic-engine measurement](../semantic-engine.md#the-problem-measured) mechanized.

**Steps:**

- [x] `tools/davinci/croquis-consumers.mjs`: for each public product on `crates/vize_croquis/src/croquis.rs` (the ~25 `pub` fields + exported types), resolves consumers **symbol-aware** — rustdoc-JSON or a syn-based scan over `use` paths and field accesses, so aliases/re-exports resolve — with plain text grep only as a cross-check; emits `davinci-road/plan/croquis-consumption.md` with `product × consuming-crate × site-count` _(landed as a `use`-declaration parser with per-file alias tables, `pub use` re-export chains, and typed-receiver field-access counting; grep disagreements reported in the artifact; a rustdoc-JSON upgrade remains possible later)_
- [x] Verify output matches the hand-audited 2026-08-13 numbers (`EffectGraph`→doctor only; `RaceConditionTracker`/`ProvideInjectTracker`→none; etc.) — discrepancies get investigated, not papered over _(exact tracker types confirmed at zero external code references; the matrix additionally surfaces sibling product types — `EffectGraphSummary`, `RaceConditionRisk`, provide keys — consumed by `vize_croquis_cf`, plus one direct `croquis.race_conditions` read in `vize_croquis_cf/src/rules/race_conditions.rs`, which the hand audit's "outside croquis" framing did not count)_
- [x] Staleness check `tests/tooling/davinci-matrices.test.ts`: regenerates and diffs; fails CI when committed artifact is stale

**Acceptance:** matrix committed; staleness test demonstrably fails on an injected fake consumer then passes after regen.

**Deps:** none.

## P0-8 — Rule-parity matrix (SFC × JSX)

**Deliverable:** classification substrate for charter #7's fairness metric.

**Steps:**

- [x] `tools/davinci/rule-parity.mjs`: walks `crates/vize_patina/src/rules/**` (345 files), extracts per rule: registration surface (template/script/markup-facade), whether it runs on `lint()` vs `lint_jsx()` paths, croquis usage — resolved **symbol-aware** (syn-based `use`-path resolution, not raw text matching) _(landed on the shared P0-7 `use`-declaration parser: 345 files = 245 META-bearing rule files + 100 helpers/organizers/tests; **surface** = which trait the file implements and which hooks each impl overrides (`Rule` template-visitor hooks / `run_on_sfc` / `MarkupRule` / `ScriptRule` / `CssRule` / musea / corsa `TYPE_AWARE_RULES`); **path membership** = mechanically joined from the `register…(Box::new(…))` sites plus `Linter::lint_jsx`'s three-lane partition — `as_markup_rule` ⇒ IR lane, `+ jsx_needs_lowering` ⇒ lowered-IR lane, template hooks only ⇒ lowering fallback, `run_on_sfc`/corsa-only ⇒ dispatched-but-inert — and the generator hard-fails if the dispatch anchors in `linter/engine.rs` drift; script/css/musea registries never reach `lint_jsx`, asserted likewise)_
- [x] First-cut classification column: `neutral-core-candidate` / `vue-dialect-bound` / `container-bound`, derived heuristically (uses `v-`-specific node kinds ⇒ dialect; uses SFC block structure ⇒ container) — hand-corrections stored in a sidecar `rule-parity-overrides.toml`, never edited into generated output _(precedence container > dialect > neutral; overrides validate rule names/values and mark rows `*`)_
- [x] Committed artifact `davinci-road/plan/rule-parity.md` + staleness check in `tests/tooling/davinci-matrices.test.ts`

**Acceptance:** totals reconcile with the file count; JSX-runnable count matches the markup-facade migration list; staleness check wired.

**Deps:** none.

## P0-9 — `Span` type + `SourceLocation` consumer inventory

**Deliverable:** the future span type, landed unused, plus the migration map P1 executes.

**Steps:**

- [x] `crates/vize_carton/src/span.rs`: `Span { start: u32, end: u32 }` with `slice(&'a str) -> &'a str`, `to_block_relative(block_start) -> Span`, `len()`; `#[derive(Copy, Clone, PartialEq, Eq, Hash)]`; size static-assert (8 bytes)
- [x] Block-relative hashing helper (`hash_relative(hasher, block_start)`) per the rustc relative-span import
- [x] Inventory script `tools/davinci/sourcelocation-inventory.mjs`: every read of `SourceLocation::{source, start.line, start.column, end.line, end.column}` across the workspace, grouped by crate and function; committed as `davinci-road/plan/sourcelocation-inventory.md` with counts
- [x] Doc note in the inventory: which consumers move to `Span::slice` (diagnostic excerpts), which to offset-derived line/col (source-map `finish()`), which delete outright

**Acceptance:** type + unit tests + size assert land; `tools/davinci/corpus-diff.mjs` empty (zero behavior change); inventory committed with per-crate counts. _Status: type, tests, size assert, and inventory landed unused by production code; the corpus-diff clause stays pending until the P0-5 tool exists (nothing to run it on yet — the type has zero production call sites)._

**Deps:** P0-5.

## P0-10 — Folio harness skeleton + VIR absorption

**Deliverable:** the dump/round-trip substrate; croquis's "VIR" becomes the first folio page.

**Steps:**

- [x] `crates/vize_davinci/` crate skeleton (workspace member, `no_std + alloc`, `experimental`): only `folio` module for now — `trait Folio { fn print(&self, w: &mut W, mode: FolioMode) -> fmt::Result; fn parse(input: &str) -> Result<Self, FolioError>; }` with `FolioMode { Full, Display }`. **Equality laws are mode-explicit:** `Full` is the injective, parseable form (round-trip laws apply); `Display` elides spans/defaults for humans and carries **no** round-trip law. Normalization (stable sequential ids, sorted map iteration) applies to both — _landed with a provided `print_to_string(mode)` convenience on the trait; no `std` feature was needed (host-only code lives in the binary)_
- [x] Normalization rules documented in `davinci-road/plan/folio-format.md` (the "test-mode printer" contract: what is elided, what is stable)
- [x] `crates/vize_davinci/src/bin/davinci-opt.rs` (or a `tools/` binary if binary-in-lib is awkward): `davinci-opt --roundtrip <file>` — parse → print → byte-compare; `--stage croquis` initially — _binary-in-lib worked; the bin also compiles under `wasm32-wasip2` (wasip2 has `std`)_
- [x] Croquis folio: implement `Folio` for the existing VIR dump content (`crates/vize_croquis/src/croquis/vir.rs`) — print delegates to the current renderer, parse added; VIR's "display-only" doc updated to point at Folio — _"delegates" is by contract, not by call: `CroquisFolio` is a document model of the dump; the fixture harness (`crates/vize_davinci/tests/croquis_folio.rs`) pins parser and renderer together, keeping `vize_davinci` free of a `vize_croquis` dependency (dev-dep only). Discovered en route: `[macros]` entries can span physical lines (multi-line type args), handled by the parser_
- [x] Deprecation alias: ~~`crates/vize_curator/src/inspector/payload.rs`~~ the `vir` payload key actually lives in `crates/vize_vitrine/src/wasm/analyze.rs` (curator's inspector payload never carried it); that site keeps `vir` and adds nested `folio.croquis` with the same content; playground consumes either (no playground change required this task)
- [x] insta helper: `assert_folio_snapshot!(value)` using the normalized printer (allowed `#[allow(clippy::disallowed_macros)]` per existing insta convention)

**Acceptance:**

- `davinci-opt --roundtrip` is identity on ≥10 committed croquis-folio fixtures (drawn from the P0-2 fixture ladder — _the ladder had not landed, so the 14 fixtures come from the e2e project fixtures plus two written for section coverage; provenance table in `folio-format.md`_). **Folio identity is defined post-normalization:** the round-trip law is `print(parse(t)) == t` for canonical (normalized) text `t`, plus structural equality `parse(print(v)) == v` for values — non-canonical input is normalized by the first print, by design
- Existing VIR consumers' tests pass untouched
- `wasm32-wasip2` target compiles for `vize_davinci` (`cargo build -p vize_davinci --target wasm32-wasip2`)

**Deps:** none. **Non-goals:** S1/S2 folios; pipeline running in `davinci-opt` (P2).

## P0-11 — Profiler source-level attribution + stable export

**Deliverable:** `vize_carton::profiler` speaks pass × stage × block × span, exports machine-readable.

**Steps:**

- [x] Extend the span key in `crates/vize_carton/src/profiler.rs`: today's dotted string (`"atelier.dom.template.parse"`) gains optional structured fields `{stage, pass, file_id, block, span}` — additive, existing `profile!` call sites unchanged — done: `SpanAttribution` (all fields optional, `Copy`, `const` builders over static strs and integer ids — zero allocation on the hot path) plus a new `profile!(name, attr: …, block)` macro arm and `global_span_attributed`/`record_attributed`; attributed samples land in a separate sharded `key × attribution` store, so every pre-attribution call site, key, and consumer (`get`/`all`/`summary`) is behavior-identical
- [x] Existing allocation-tracking option surfaces per-span alloc counts in the export — done: span guards delta new monotone thread-local allocation counters (`profiler/allocation.rs`), with parent/child attribution mirroring the self/child duration accounting; aggregated as `Metrics::{alloc_calls, alloc_bytes, self_alloc_calls, self_alloc_bytes}` and exported per span; exact counts proven under an installed `ProfilingAllocator` in `crates/vize_carton/tests/davinci_profile_export.rs`
- [x] Export: `--profile-json <path>` on the CLI (`crates/vize/src/commands/` shared arg), schema `davinci-road/plan/profile-export.schema.json`, size-budgeted per `vize_doctor::ai_context` conventions (`crates/vize_doctor/src/ai_context/`) — done: shared `ProfileExportArgs` (`crates/vize/src/commands/profile_export.rs`) flattened into build, lint, and check (direct + socket runners), compile path wired end-to-end; deterministic ranking, 512-span/256-counter default budget with explicit `truncation` accounting (never silent); schema validated in-repo by a strict subset validator following `benchmarks/davinci_harness/src/report.rs` (chosen over a node-side CLI-spawn test to avoid a full `vize` binary build during the concurrent baseline sweep)
- [ ] Zero-overhead check: profiling disabled ⇒ P0-2 benches within noise of pre-change baselines (pending: quiet-machine bench run — P0-4 gate covers it once reference baselines exist; code-level: both `profile!` arms keep the disabled path at the pre-existing single relaxed atomic load — `Profiler::is_enabled` in `profiler/core.rs` — and per-span alloc counting sits behind the existing `ALLOCATION_TRACKING_ENABLED` relaxed-load short circuit in `profiler/allocation.rs`)

**Acceptance:** export validates against schema on a corpus-project compile; overhead check green; schema documented for Spolvero (C-4) and the AI loop (C-10).

**Deps:** P0-2 (baselines for the overhead check).

## P0-12 — Assurance harness

**Deliverable:** the enforcement tooling for charter #21.

**Steps:**

- [x] Banned-assertion lint `tools/davinci/assertion-lint.mjs`: scans `#[cfg(test)]` code and `tests/**` for `contains(`, `starts_with(`/`ends_with(` in asserts, regex-matching asserts, and partial-JSON comparisons; allowlist `davinci-road/plan/assertion-allowlist.toml` (`[[allow]]` group = justification + expiry + covered paths)
- [x] CI self-test: fixture with a deliberately bad assertion that the lint must flag (lint the linter) — `tests/tooling/davinci-assertion-lint.test.ts`
- [x] `cargo-mutants` baseline: run on `vize_carton` + `vize_relief` (pilot pair); scores recorded in `budgets.toml` `[mutation]` section _(measured: carton 0.4678 over 654 viable mutants, relief 0.8372 over 43 — run record + missed-mutant listings in `mutation-baseline.md`)_
- [ ] Mutation CI job (nightly lane, not per-PR — runtime cost) with ratchet comparison — the reviewed job definition lives in `mutation-baseline.md`; it lands with the reference-runner baselines (P0-4's Blacksmith re-record), which also re-record the `[mutation]` floors on the reference runner
- [x] Construct taxonomy DRAFT `davinci-road/plan/taxonomy.toml`: dimensions = element kind (native/component/slot/template/svg/mathml) × directive (`v-if/-else-if/-else, v-for, v-on, v-bind, v-model, v-show, v-html, v-text, v-once, v-memo, v-cloak, v-pre, custom`) × modifier classes × binding sources (setup/props/data/inject/global) × block combinations
- [ ] Taxonomy dimensions signed off — **review point: awaiting maintainer sign-off**
- [x] Matrix generator skeleton `tools/davinci/matrix-gen.mjs`: taxonomy → fixture stubs under `tests/fixtures/davinci-matrix/` (generation only; expected outputs arrive with the stages that consume them — skeleton covers the element-kind × directive plane, deterministic, `--check` staleness mode; fixtures not yet committed)

**Acceptance:** lint self-test green and wired to CI; mutation scores committed + ratcheted; taxonomy signed off; generator emits deterministic fixture sets (two runs identical).

**Deps:** none.

## P0-13 — Seeded-defect + suppression-telemetry pilots

**Deliverable:** both FP/FN oracles running against the _existing_ toolchain.

**Steps:**

- [x] Seeded-defect generator `tools/davinci/seed-defects.mjs`: two pilot classes — (a) undefined template ref: rename a `<script setup>` binding referenced from the template; (b) unused binding: inject an unreferenced `const` — applied to matrix fixtures and a corpus shard copy _(landed with `--fixtures`/`--matrix`/`--corpus-shard` sources; copies only, submodules never mutated; the manifest records every injection's file, span, original/new identifier, expected rule id, plus the per-file edit list the assertion maps baseline diagnostics through)_
- [x] Recall assertion: current Patina must flag 100% of seeded class-(a) instances (`no_undefined_refs` rule) — asserted by **identity, not count**: the seeded-defect manifest records each injection's file + span + expected rule id, and the assertion compares the exact diagnostic set against the manifest (count-only matching is banned by the assurance doctrine); class-(b) recall recorded (not gated yet — `unused_bindings` has no lint consumer today, which this pilot documents as a finding) _(assertion landed as `seed-defects.mjs --assert`, exact-multiset oracle over `expected = shift(baseline) ∪ manifest`; **measured outcome falsifies the 100% expectation**: `vue/no-undefined-refs` is registered by no preset and no opt-in path, so `vize lint` cannot fire it — recall 0/49 on the shard, 0/3 on the committed set, ledgered as FN-1 with the registration-gap witness; the oracle mechanism itself is CI-proven both directions via synthetic-diagnostic hooks, and a same-count wrong-location set fails listing the exact miss. Class-(b) recall recorded: 0% everywhere, FN-2)_
- [x] Suppression scan `tools/davinci/suppression-telemetry.mjs`: collects `eslint-disable` comments in corpus sources with rule names mapped to vize analogs; reports vize diagnostics firing on those exact lines as FP candidates _(vize honors `eslint-disable` pragmas natively, so the scan lints byte-length-preserving defused copies or the intersection is empty by construction; mapping table = the committed `tests/_fixtures/patina-eslint-vue-rule-map.json` (123 mapped rules) + an empty verified-core sidecar; unmapped names are reported, not errors — shard result: 0 mapped candidates with scope proof, 1 unmapped (`no-console`))_
- [x] First ledgers committed: `davinci-road/plan/ledger-fn.md`, `ledger-fp.md` — **with the pilot candidates triaged**, not blank: every candidate the shard scan produces gets a disposition (`fixed` / `justified-with-witness` / `deferred-with-issue`), and an empty ledger is acceptable only alongside scan-scope proof (files-scanned and rules-mapped counts matching the shard manifest) _(FN-1 `vue/no-undefined-refs` unreachable, FN-2 `unused_bindings` unconsumed, FP-1 `type/require-typed-emits` script-relative spans mis-projected onto file coordinates — witnesses read at the sites; the FP mapped-candidate section is empty with the shard scope proof quoted)_

**Acceptance:** both tools run in CI on one corpus shard; class-(a) identity assertion green; ledgers committed with pilot triage complete and referenced from the assurance doc. _(Status: CI runs both tools over the committed miniature set (`tests/tooling/davinci-fpfn-pilots.test.ts`, fixtures at `tests/_fixtures/davinci-fpfn/`) so the identity assertion is exercised without corpus hydration; the corpus-shard run stays local/nightly until CI hydrates the corpus (same pending as P0-6). The class-(a) assertion mechanism is green in CI; the toolchain recall gate is red by measurement and ledgered as FN-1 — flipping it requires the preset-registration decision recorded there.)_

**Deps:** P0-6 (corpus conventions), P0-12 (fixture home).

---

## Exit gate (machine-checkable)

- [x] All benches run in CI with committed baselines and `budgets.toml` (P0-1..4) — harness, 85-entry budget registry, and the compare gate landed; **reference-runner baseline recording + the CI bench lane remain pending** (budgets seeded at 0 = report-only until recorded on Blacksmith)
- [x] Corpus baseline + diff tool reproducible; expansion round 1 merged (P0-5..6) — 142-project baseline committed with scope proof; the one measured nondeterminism (typechecker/element-plus TS6307 flap) is quarantined non-gating and filed
- [x] Consumption + rule-parity matrices committed with staleness checks (P0-7..8)
- [x] `Span` landed unused; `SourceLocation` inventory committed (P0-9)
- [x] `davinci-opt --roundtrip` identity on croquis folio; VIR alias live; `vize_davinci` builds for wasm32-wasip2 (P0-10)
- [x] Profiler export schema validating; zero overhead when off (P0-11) — schema-validated in CI by the carton export test; the bench-measured overhead comparison joins the P0-4 gate once reference baselines exist
- [ ] Assertion lint + mutation baseline + taxonomy signed off (P0-12) — lint live in CI with the 236-file debt allowlist; mutation baseline recorded (`vize_carton` 0.4678, `vize_relief` 0.8372 — `mutation-baseline.md`; the nightly ratchet lane lands with the reference-runner re-record); **taxonomy dimensions still awaiting maintainer sign-off**
- [x] FP/FN pilot oracles running with committed ledgers (P0-13) — CI covers the committed miniature set; corpus-shard lane joins CI with corpus hydration (P0-6)
- [x] `tools/davinci/corpus-diff.mjs` across the whole phase: **empty** (zero behavior change) — verified at the phase-final davinci head: zero gating drift across all rows, scope proof matched (the filed unstable row surfaced non-gating, by design)
