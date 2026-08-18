# Phase 2 — Task contracts

> [!NOTE]
> The 22 per-task contracts for [Phase 2 — Disegno and the Pass Manager](./phase-2.md) — Deliverable / Steps / Acceptance / Deps / Non-goals for P2-1 through P2-20. They live beside the phase file rather than inside it because the contracts alone exceed the repository's 350-line source-length budget (`tools/moon/cmd/source_file_lengths --max-lines 350`, which plan files are not exempt from). The phase-level record — what the re-cut changed, the phase-1 carry-ins, the TODO index and the exit gate — stays in [phase-2.md](./phase-2.md), and that index is where a task's box gets checked; the **Steps** sub-checkboxes are here.

## P2-1 — `vize_davinci` core types

**Landed 2026-08-19** — full record: [phase-2-records/p2-1.md](./phase-2-records/p2-1.md).

**Deliverable:** the id / side-table / diagnostic substrate every later stage keys on, in the existing P0-10 crate, which today declares exactly one module (`folio`) and depends only on `vize_carton`.

**Steps:**

- [x] `crates/vize_davinci/src/id.rs`: `NodeId(u32)` newtype — `Copy`, `Eq`, `Hash`, with a niche so `Option<NodeId>` stays 4 bytes (`NonZeroU32` or a reserved sentinel — pick one and record why in the PR) _(`NonZeroU32`; the reasoning is in the type's own docs)_
- [x] `crates/vize_davinci/src/side_table.rs`: `SideTable<T>` keyed by `NodeId`, with the residency decision recorded explicitly and the densification trigger documented rather than densified _(sparse `vize_carton::FxHashMap<NodeId, T>` only; the dense arena form and its three-condition trigger are written down, not built)_
- [x] `crates/vize_davinci/src/diagnostic.rs`: the unified `Diagnostic` — `vize_carton::Span`, stage-of-origin, structured parts, and a witness slot (empty until P4-6). Message text is **owned**, the deliberate P1-10 exception, so a diagnostic survives `Allocator::reset`
- [x] Node-size `const` asserts on all three _(with a **deviation recorded**: only the pointer-containing figures carry the `#[cfg(target_pointer_width = "64")]` guard — see the record)_
- [x] `'static` assertion on `Diagnostic` (the P1-11 arena/cache contract, enforced the same way `SfcCompileResult` and the batch cache types are)
- [x] `#![no_std]` + `extern crate alloc;` held; rustdoc per public type

**Acceptance:** `cargo test -p vize_davinci` green (TS-1) — 20 new unit tests, 31 total; the size asserts and the `'static` assertion compile (a violation is a compile error, not a test failure); `cargo build -p vize_davinci --target wasm32-wasip2` green; TS-11 empty, **proved mechanically** rather than argued — `cargo tree -i vize_davinci --workspace` lists no reverse dependencies, so nothing on any compile path can observe these types; TS-13 green on the new tests, with no allowlist entry added (the allowlist only shrinks). **Deps:** none (phase-1 exit). **Non-goals:** replacing `vize_relief::CompilerError` at its call sites — the single-diagnostics-channel convergence that structurally ends dual assembly is P4's; densifying the side table; the S2 ops themselves (P2-5a).

## P2-2 — Pass manager

**Landed 2026-08-19** — full record: [phase-2-records/p2-2.md](./phase-2-records/p2-2.md).

**Deliverable:** `crates/vize_davinci/src/pass/` — pipelines as const data with SIL-style classification and build-time fusion grouping. This is greenfield: `PassObserver`, `PassManager` and a pass-`Pipeline` type have **zero** occurrences in `crates/` today.

**Steps:**

- [x] pass module root: pass description as **const data** — no registry of trait objects, and dispatch resolved per pipeline, never per node (performance guardrail 1) _(`src/pass.rs`; the crate has no `mod.rs` anywhere, per the workspace convention, so the root is `pass.rs` with `#[path]` submodules)_
- [x] Classification enum `PassKind { MandatoryDiagnostic, MandatoryLowering, Optional }` (SIL import). Mandatory passes are unfusable barriers, run at every optimization level, and are the only passes that perform the raw→canonical transition _(the barrier law is asserted in the `const fn` constructor, so a violating `const DESC` does not compile — **verified by compiling one**)_
- [x] `Raw<S>` / `Canonical<S>` wrapper types so the transition is type-level: an optional pass cannot produce `Canonical<S>` because it cannot name the constructor _(private field + a `produce::<P>` that forces a per-monomorphization `const` assertion; the optional case is a compile error, **verified by compiling one**)_
- [x] `Fusability { Fusable, Barrier }`; fusion computes the **preserved-set intersection at build time** (`const fn`), so a grouping regression is a compile error rather than a runtime surprise _(every planning query is a `const fn`; the fixture plan is pinned in `const` items)_
- [x] pipeline syntax `s2(a,b),s2-to-s3(c)` parsed into the same const shape for `davinci-opt --pipeline`; grammar in the module docs with every error message spelled out _(`src/pass/pipeline.rs`; ten documented rejections, each asserted on its full rendered message)_

**Acceptance:** `cargo test -p vize_davinci` green (TS-1) — 61 tests, 25 new here — covering: pipeline-string round-trip byte-exact for canonical strings over an eight-string corpus, and every malformed input asserted on the **exact** documented message and variant (no substring assertions, assurance §4, TS-13 green with no allowlist entry added); the fixture pipeline's fused group boundaries pinned in `const` items; the mandatory-never-fuses canary proven over five arrangements rather than one. No bench was added, so TS-10 has nothing to record. **Deps:** P2-1. **Non-goals:** observer hooks (P2-3); running real passes (P2-9); optimization tiers scaling budgets rather than pass sets (P3-10); a JS/WASM plugin pass tier (phase 6).

## P2-3 — `PassObserver`

**Landed 2026-08-19** — full record: [phase-2-records/p2-3.md](./phase-2-records/p2-3.md).

**Deliverable:** the seven-hook observer plus the four in-tree observers, with fusion groups reported explicitly so timing never lies.

**Steps:**

- [x] Seven hooks: `before_pipeline` / `after_pipeline`, `before_pass` / `after_pass`, `before_analysis` / `after_analysis`, `on_fail` _(all seven with empty default bodies; `on_fail` suppresses `after_pipeline`, since a run that failed did not finish)_
- [x] Timing observer emitting the **P0-11** profile-export schema through `vize_carton::profiler`, reusing `SpanAttribution` rather than a second attribution model _(one attributed span per **walk**, keyed on the group's lead pass)_
- [x] Folio-printing observer, budget-counting observer (the walk counter P2-12b reads), remark sink (a no-op sink until P3-13) _(the folio observer binds to the **existing** P0-10 `Folio` trait rather than waiting for P2-4's derive — recorded as a deviation, since the contract expected it to consume P2-4)_
- [x] **Fused groups are reported as one walk with their member passes named** _(`PassEvent` carries its `FusionGroup`, `is_group_entry`/`is_group_exit` and `group_members`; the law is pinned by `tests/pass_observer_law.rs` and again at the timing level by `tests/pass_observer_timing.rs`)_
- [x] Attachment is checked once per pipeline, never per node (guardrail 1); the no-observer path must compile to the un-observed pipeline _(dispatch is **static**, so there is no attachment check at all — strictly stronger than once per pipeline; no `dyn PassObserver` is offered, on purpose)_

**Acceptance:** zero cost when nothing is attached, pinned by the bench pair `davinci_pipeline_unobserved` / `davinci_pipeline_no_observer` — both measured at **`allocs = 0`**, alloc-identical, both registered in `budgets.toml [bench]` and reported `alloc-gated … ok` by `tools/davinci/bench-compare.mjs` (TS-10). The fusion-group reporting law is pinned by ordinary integration tests under `crates/vize_davinci/tests/`, so they run in the default `cargo test --workspace` lane (the P1-5/P1-7 counter-law shape). Timing output reaches the P0-11 export with the right key and attribution, asserted directly; schema conformance stays pinned where the strict validator lives (TS-15, `vize_carton`) rather than copied — see the record. TS-1 (61 → 71 tests), TS-13 green with no allowlist entry added. **Deps:** P2-2. **Non-goals:** remark _content_ (P3-13); the Spolvero transport (P2-19) and UI (P2-18); provenance materialization policy (P2-8 records the pairs, the ring-buffer-vs-full decision is P2-12b's fusion measurement).

## P2-4 — Folio derive + `davinci-opt` pipelines

**Deliverable:** `#[derive(Folio)]` and a `davinci-opt` that runs pipelines, so every later stage gets its dump and its pass tests for free.

**Steps:**

- [ ] New proc-macro crate (`crates/vize_davinci_derive/`, workspace member, `experimental`, `publish = false`). It is a **host build dependency and stays `std`** — record that edge for P2-14's audit
- [ ] The derive generates the existing trait's exact shape (`crates/vize_davinci/src/folio.rs:81`): `print(&self, w, mode: FolioMode)` and `parse(input) -> Result<Self, FolioError>` with 1-based line numbers, stable field order from the type shape, and the normalization rules already written in [`folio-format.md`](./folio-format.md) (sorted map iteration, stable sequential ids, fixed section order, empty sections omitted, LF)
- [ ] **Derive the mechanical trio only** (print / parse / field order) — the ODS lesson. Anything carrying a semantic decision (what `Display` elides, what a section means) is hand-written and reviewed
- [ ] Extend `crates/vize_davinci/src/bin/davinci-opt.rs`: add `--pipeline "<syntax>"` and generalize `--stage <s>` beyond `croquis`. Today `--roundtrip <file>` is a **required** flag (`davinci-opt.rs:53`); it becomes one of two alternatives, and the existing usage string, exit codes (0 identity / 1 mismatch / 2 usage) and `--stage croquis` behavior stay byte-identical
- [ ] insta pass-test harness: folio in → pipeline → **full normalized folio** snapshot out, with targeted structural asserts only as supplements (assurance §4); `assert_folio_snapshot!` (`folio.rs:114`) is the printer

**Acceptance:** TS-16 per derived type — `print(parse(t)) == t` byte-exact in `Full` mode and `parse(print(v)) == v` structurally, with `Display` explicitly carrying no round-trip law. TS-17 established: at least one pass test per landed pass. The 14 committed P0-10 croquis fixtures still round-trip byte-identically through the extended binary (regression pin). TS-1, TS-13. **Deps:** P2-2, P2-3. **Non-goals:** deriving `Display` prose or semantic equality; a folio _diff_ format (P2-13 prints per-pass folios; diffing is C-3); the independent Lean folio checker (C-23).

## P2-5a — `vize_disegno` S2 op and type family

**Deliverable:** the S2 crate and its ops — the pivot stage and the primary consumer surface. `vize_disegno` does not exist in the tree today.

**Steps:**

- [ ] `crates/vize_disegno/` created and added to `[workspace] members` in the root `Cargo.toml`; `publish = false`, `metadata.vize.stability = "experimental"`, `#![no_std]` plus `extern crate alloc;` from birth
- [ ] Op enums: element / component / text / interpolation / `ui.if { regions }` / `ui.for { binding, region }` / `ui.slot` / `ui.model { contract }` / `vue.directive`. **Regions are owned by their op** — this is what makes fusion tractable, because the enter/exit sibling mutation in `crates/vize_atelier_core/src/transform/structural.rs` (which merges `v-else` branches on the parent's child list) is precisely the re-visit source a region-owning `ui.if` never needs
- [ ] `ui.model` carries the **binding contract only** (what is read, what is written, the value-type flow), with element kind and dialect modifiers as attributes. Realization is never expanded in S2; IME/composition handling is runtime-owned by declaration (architecture, charter #23 tiering)
- [ ] Whatever is genuinely Vue-specific stays a `vue.*` dialect op instead of shaping the core — the fairness litmus test P2-16 then exercises
- [ ] **Drop-free by construction**: every type arena-resident through `vize_carton::{Box, Vec}`, whose `needs_drop` const assertion is the enforcement (P1-10 measured it catching two real violations); no `impl Drop` anywhere in the crate
- [ ] Node-size `const` asserts per op type, guarded by `#[cfg(target_pointer_width = "64")]` (P2-14 makes wasm32 required)
- [ ] Exhaustive-match canary: a test that matches every variant with no `_` arm, so adding a variant breaks it. No `_` arms anywhere downstream
- [ ] S2 folio page from birth via the P2-4 derive

**Acceptance:** TS-16 on the S2 folio (`Full` byte-exact, `Display` no law); TS-1; the guarded size asserts compile; the exhaustive-match canary is _demonstrably_ broken by an injected variant and green after handling it (the P0-7 staleness-check pattern — prove the canary, do not assume it); `grep -rn "impl Drop" crates/vize_disegno/src` → 0; `cargo build -p vize_disegno --target wasm32-wasip2` green; TS-11 empty (nothing consumes S2 yet); TS-13. **Deps:** P2-1, P2-4. **Non-goals:** the expression reference (P2-5b); lowering into it (P2-8); the verifier (P2-6); speculative `vue.*` ops — a dialect op lands with the transform that needs it (P2-9); S3 (`vize_impeto`, phase 3).

## P2-5b — `ExprRef` contract, including the retained-`None` classes

**Deliverable:** `ExprRef<'a>` with a written, tested, folio-serializable policy for **every** expression class the parser actually produces.

**Split reason** (recorded per the plan README): the provisional block assumed `ExprRef { Js, Foreign }` was total. P1-5 measured that the retained AST exists only for text that parses as one complete TS expression covering the whole content, and P1-9 measured 11.73% of corpus rewrites landing outside it. The `None` classes are a real design question the provisional text did not know about, and they are large enough to deserve their own review.

**Steps:**

- [ ] `ExprRef<'a>` with the two architecture variants — `Js(&'a oxc_ast::ast::Expression<'a>)` and `Foreign(&'a ForeignExpr<'a>)`; `Foreign` is **type only** until phase 6 (charter #28), carrying dialect id + source slice + span + side tables
- [ ] **Decide the `None` classes and record the decision.** The measured shapes are: v-for values (`item of items` — the splitter synthesizes sub-expressions that never existed as template text, and JS `in` associates left while Vue's v-for grammar splits at the first viable `in`/`of`, so they genuinely disagree on `a in b in c`), v-on multi-statement bodies, nesting-guard-refused text (`vize_carton::expression_guard::expression_is_safe_to_parse`, refused _before_ parsing and before counting), text oxc rejects, and compound expressions rebuilt from source slices. Candidate resolutions to weigh: (a) a third variant carrying slice + span + a classified reason, with **pessimal documented semantics from day one** (the LLVM `undef`/`poison` regret, imported as a rule in `prior-art.md`); (b) widen the retained contract in `crates/vize_armature/src/parser/expression.rs` so the classes shrink; (c) both. Record which, and the measurement that picked it
- [ ] **Owned folio payload**, because arena references cannot persist across a compile (P1-11's contract, enforced by `'static` assertions and the debug arena-generation stamp in `crates/vize_carton/src/allocator/generation.rs`): `Js` serializes as source slice + span and re-parses into the arena on load; `Foreign` as dialect id + source + span; the escape variant as reason + slice + span
- [ ] Arena-reset replay test: print a folio → drop the `pool::acquire()` guard (arena reset) → parse → structural equality. This is P1-11's resident-cache reset scenario applied to folios
- [ ] The capability contract (enumerate referenced bindings, classify const-ness, map spans, emit for a target) is resolved **per file, never dyn-dispatched per node** (guardrail 1)

**Acceptance:** TS-16 including `Js`, `Foreign` and escape-variant `Full`-mode fixtures **and** the arena-reset replay test; size asserts; TS-1; TS-11 empty. The class sizes backing the decision are machine-measured, not asserted: rerun the P1-7/P1-9 counters and record the per-class numbers in the PR —

```sh
VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
  cargo test -p vize_atelier_sfc --features davinci-differential \
  --test davinci_differential -- --nocapture
```

**Review point:** the maintainer judges the escape variant's semantics against the prior-art rule — an escape variant without pessimal documented semantics is the failure this milestone exists to prevent. **Deps:** P2-5a. **Non-goals:** implementing a MoonBit dialect (phase 6, charter #28); resolving P1-8's scanner waiver ([#4365](https://github.com/ubugeeei-prod/vize/issues/4365)) — this task names where the single implementation would live and encodes neither resolution; deleting `transform_expression/reparse.rs` (P2-9 measures whether the class shrinks; deletion needs the wider contract to land first).

## P2-6 — S2 verifier v1

**Deliverable:** the between-pass verifier, debug/CI only, with an invalid-folio fixture set.

**Steps:**

- [ ] **Local checks only** (GHC Lint discipline): region nesting, id resolution (every `NodeId` a side table references resolves), expr-ref liveness, canonical-form invariants per `PassKind`
- [ ] Expr-ref liveness reuses the mechanism already in tree rather than inventing one: the debug arena-generation stamp (`Allocator::stamp` / `assert_stamp_current`, `crates/vize_carton/src/allocator/generation.rs`) panics on a value read against a reset arena
- [ ] Each invariant documented in [`folio-format.md`](./folio-format.md) — the format doc is where "canonical" is written down
- [ ] Runs between passes in debug/CI **through the P2-3 observer**, never in the release hot path (guardrail 5: verification never ships)
- [ ] Invalid-folio fixture set: hand-built invalid artifacts, each committed with its exact expected diagnostic (code + span + full message, canonical `en` locale)

**Acceptance:** TS-18 established — the verifier rejects **every** committed invalid artifact with the exact diagnostic, no partial matching (TS-13 enforces that mechanically); a release build makes zero verifier calls, asserted by the `cfg` shape plus the P2-3 zero-cost bench (TS-10); TS-1. **Deps:** P2-5a, P2-3. **Non-goals:** whole-program or fixpoint checks — those are barrier analyses and the S3 equivalent is P3-1's phase validator; the independent Lean folio checker (C-23); verifying S1 (P2-7's `render == source` is its own verifier).

## P2-7 — S1 Vue surface tree

**Deliverable:** the lossless Vue-template surface tree with typed holes.

**Steps:**

- [ ] Lossless tree with trivia; `Unexpected` / `Missing` typed structural error nodes (SwiftSyntax import), so every consumer sees one uniformly-shaped tree with holes and S1→S2 has a **single documented hole policy** instead of per-consumer error special-casing
- [ ] `render(tree) == source` debug verifier asserted on **every** construction — the cheapest high-yield verifier in the prior-art survey
- [ ] Emitted by `vize_armature`, or as a thin layer over relief until relief splits; the relief split itself is not this task's (record which shape landed and why)
- [ ] Strings stay `&'a str` per P1-10 — trivia is a source slice or an arena copy, never an owned string; the tree is `Drop`-free and arena-resident, with the container const assertion as enforcement
- [ ] Node-size `const` asserts, `#[cfg(target_pointer_width = "64")]`-guarded

**Acceptance:** TS-19 established — `render(parse(src)) == src` **bytes** as a property over the corpus and over the malformed-fixture set, including the `Unexpected` / `Missing` paths; TS-11 empty (S1 is additive here, nothing consumes it yet); guarded size asserts; TS-1, TS-13. **Deps:** P2-1. **Non-goals:** pug as an S1 dialect (charter #12, phase 4); the OXC-backed lossless **script/JSX** wrapper — "S1" reads wider than this task and the script side is phase 4's, with the formatter and autofix consumers; retiring the `vize_glyph` byte scanner (phase 4, charter #41); `vize_musea`'s hand parser (phase 4).

## P2-8 — S1→S2 Vue lowering

**Deliverable:** a total lowering function, with provenance and a fuzz lane.

**Steps:**

- [ ] **Total function, no rollback** (MLIR import): every input yields S2 or a diagnostic, never a panic and never a partial-then-abandoned state
- [ ] Hygiene scope-tags on synthesized identifiers (slot props, v-for scopes) so a later pass can never confuse a synthesized name with an author's
- [ ] `MacroExpansionInfo`-style provenance pairs recorded at each lowering decision (before/after, by pass name); provenance **survives failure** — partial S2 fragments are kept on error, Lean-InfoTree style, so the LSP and Spolvero stay live on broken SFCs
- [ ] v-for consumes P2-5b's decision for its alias/source sub-expressions rather than re-deriving it; the `a in b in c` disagreement recorded at P1-6 is the reason the retained AST of the v-for value must not be consumed naively
- [ ] Fuzz targets for S1→S2 and the folio parsers added under `tests/fuzz/fuzz_targets/` (joining the five that exist: `css_parse`, `js_ts_expression`, `sfc_parse`, `template_compile`, `template_lexer`) and the new crate paths added to `.github/workflows/fuzz.yml`'s PR path filter

**Acceptance:** TS-20 established — no panic on arbitrary bytes, diagnostics rather than crashes, with fixed crashes carrying deterministic reproducers (TS-8's convention). Every corpus template lowers or produces a diagnostic, asserted by a corpus-runnable entry in the P1-6/P1-7 differential-lane shape (`#[cfg(any(test, feature = "davinci-differential"))]`, env-var corpus widening, exact-pinned counts in the plain suite so a cfg regression fails loudly). TS-19 unaffected; TS-11 empty; TS-1, TS-13. **Deps:** P2-5b, P2-7. **Non-goals:** JSX lowering (P2-16); pug (phase 4); replacing the atelier transform lane (P2-9); emitting anything (P2-11).

## P2-9 — Core transforms as S2 passes

**Deliverable:** the core transform lane re-expressed as classified S2 passes, with the old lane still live. **Explicitly marked small series** (the plan README's permitted exception): one reviewable PR per transform, all under this ID.

**Steps** — the checklist is the actual directory, `crates/vize_atelier_core/src/transforms/`, reached through the Rust module `vize_atelier_core::steps` (`#[path]` attributes in `src/steps.rs`):

- [ ] `v_if.rs` → `ui.if` regions — the first and highest-value port, because regions replace the sibling mutation in `src/transform/structural.rs`
- [ ] `v_for.rs` → `ui.for` region
- [ ] `v_slot.rs` (+ `v_slot/params.rs`, `v_slot/validate.rs`) → slot normalization
- [ ] `transform_text.rs` → text / interpolation merging
- [ ] `hoist_static.rs` (+ `hoist_static/props.rs`, `static_type.rs`) → an S2 **analysis** pass (a fact, not a mutation)
- [ ] `transform_element.rs`, `v_bind.rs`, `v_on.rs`, `v_model.rs`, `v_memo.rs`, `v_once.rs` → the normalized-binding ops; `v_model.rs` lowers to `ui.model`'s **contract**, not its realization
- [ ] `legacy.rs` / `legacy_filters.rs` → `vue.*` dialect ops behind the existing `_legacy` feature (zero cost when off)
- [ ] `transform_expression.rs` and its 13-file subtree stay on the old lane in this task — it is P1-7/P1-9's working set and P2-5b owns its future
- [ ] Every ported pass is classified (`MandatoryDiagnostic` / `MandatoryLowering` / `Optional`) and marked fusable or barrier — **review point**, since a misclassified mandatory pass silently leaves the fusion budget
- [ ] The old lane stays live behind the in-phase flag (charter #26) and is deleted at the exit gate
- [ ] **Differential lane, the P1-6/P1-7/P1-9 shape**: `#[cfg(any(test, feature = "davinci-differential"))]` dual-run comparator inside the migrated path, compared at the DOM-output level; process-global counters; a plain-suite coverage witness pinning exact counts so a cfg regression that disarms the lane fails; a corpus-runnable entry with its exact command recorded. Divergence panics — investigate, never average
- [ ] Measure and record the effect on the P1-9 residual classes: does region-structured lowering shrink `transform_expression/reparse.rs`'s 11.73%? A number from the existing `retained::differential` counters, not a prediction

**Acceptance:** per-pass full normalized folio snapshots (TS-17); DOM output through the **old** codegen unchanged — `node tools/davinci/corpus-diff.mjs --surface compiler` empty with scope proof (TS-11); differential lane green over the corpus, zero divergence (TS-25); the touched benches' `allocs` re-recorded in `budgets.toml` as tightened numbers with their measurement (TS-10, ratchet); TS-1, TS-13. **Deps:** P2-6, P2-8, P2-12a. **Non-goals:** the DOM backend itself (P2-11); the SSR and Vapor lanes (phase 3 — they stay on the old lane, which is the strangler design, not an oversight); deleting the old transform lane (exit gate); porting `transform_expression/` (P2-5b decides its contract first).

## P2-10 — Style `v-bind()` ops

**Deliverable:** SFC style-block bindings visible as S2 ops (charter #13), so lint, the reactivity lattice and the type projection stop having a descriptor-level blind spot.

**Steps:**

- [ ] Surface the existing css-vars coordination as S2 binding ops. It is spread across five sites, all in `crates/vize_atelier_sfc/src/`: `css/transform.rs` (`extract_and_transform_v_bind_with_scope`), `css.rs:31` (`transform_css_v_bind`), `style.rs:644` (`extract_css_vars`), `parse/parse_sfc.rs:197` → `descriptor.css_vars` (`types.rs:42`), and emission in `compile_script/inline/compiler/setup_emit.rs`
- [ ] The op carries the **CSS-block span**, not only the expression, so a diagnostic points into the style block; spans are block-relative via `Span::to_block_relative` (`crates/vize_carton/src/span.rs:82`)
- [ ] The bound expression rides as an `ExprRef` under P2-5b's contract — CSS `v-bind()` contents are exactly the kind of text that may have no retained AST, so the class decision applies here first

**Acceptance:** the facts are visible in the S2 folio — a committed `v-bind()`-bearing SFC fixture whose folio pins them (TS-16, TS-17); **compile output unchanged**, `corpus-diff --surface compiler` empty (TS-11); TS-1. **Deps:** P2-9. **Non-goals:** lint rules or lattice consumers reading them (phase 4); a CSS S1 dialect (the `lightningcss` boundary is P2-14's audit, not this task); changing the emitted css-var naming (`scoped_v_bind_name` / `prod_scoped_v_bind_name` output is byte-frozen by TS-11).

## P2-11 — DOM backend on S2

**Deliverable:** `vize_atelier_dom` lowering S2 → codegen structure directly — the first strangler target, on the surface that holds the hard byte-parity bar.

**Steps:**

- [ ] `vize_atelier_dom` lowers S2 directly; the relief codegen-node universe (`NodeType` 13–20 codegen + 21–26 SSR codegen, of 27 variants total — `crates/vize_relief/src/relief/core.rs:10-42`) stops being **written** by the new path. It is still _read_ by SSR and Vapor until phase 3, so nothing is deleted here
- [ ] In-phase flag `VIZE_DAVINCI_DOM=legacy` (charter #26), production-selectable while the phase is live, **named in the exit gate with its deletion**. P1-13's lesson governs: an undeleted old path is an unfinished deletion with an owner, not a permanent fallback
- [ ] **Differential lane, the P1-9 shape**: dual-run old vs new DOM emission, compared byte-for-byte including helper usage, panicking on any difference; corpus command recorded in the task
- [ ] **Waiver budget: zero.** DOM emitted output is the hard byte-parity bar (charter #23) and this is the most output-visible surface in the phase; any corpus diff is a bug in this task, exactly as P1-9 ran it
- [ ] Patch-flag equivalence fixtures (the flags the new path computes must equal the old path's, per node, exactly)

**Acceptance:** `node tools/davinci/corpus-diff.mjs --surface compiler --shards 2 --timeout-ms 600000` empty across the 142-project manifest with scope proof, run from clean fixtures (TS-11); differential lane zero divergence with its comparison count recorded (TS-25); patch-flag equivalence fixtures exact (TS-1/TS-2); DOM bench `allocs` re-recorded in `budgets.toml` (TS-10); TS-13. **Deps:** P2-9. **Non-goals:** SSR and Vapor backends (phase 3); source maps from a structured S4 emitter (P3-9); deleting the relief codegen-node universe; the vapor run-then-discard double transform (P3-6).

## P2-12a — Phase-start baselines and pinned targets

**Landed 2026-08-19** — full record: [phase-2-records/p2-12a.md](./phase-2-records/p2-12a.md). One acceptance clause is carried rather than met (`corpus-coverage --check`); the record states why and where it goes.

**Deliverable:** the numbers phase 2 will be judged against, recorded **before** the work that could bias them.

**Split reason:** P1-13's gate could not tick "compile bench improvement ≥ target pinned at phase start" because neither a target nor a phase-start baseline ever existed, and it recorded that as a miss rather than inventing a number after the fact. Repeating that would make phase 2's exit unmeasurable too. Compounding it, the provisional P2-12 said "compare against the P0-3 walk baseline" — **P0-3 recorded expression re-parse counts, not walk counts**, and `budgets.toml [traversal]` is an empty reserved section. Pinning therefore becomes its own dependency-free phase-start task that must merge before P2-9.

**Steps:**

- [x] Record the **pre-S2 walk count** per ladder fixture per backend with a temporary counter hook on today's still-live pipeline — the exact P0-3 pattern (`vize_atelier_core::expr_parse_probe`, 18 sites, baseline committed to a plan doc). Ladder: `benchmarks/davinci_harness/fixtures/{small,medium,large,stress-deep,stress-wide,stress-interp}.vue` _(`crates/vize_atelier_core/src/walk_probe.rs`, 19 sites)_
- [x] Commit `davinci-road/plan/walk-baseline.md` with the counts, the exact reproducing command, and the two-run determinism proof (the P0-2/P0-5 convention)
- [x] Fill `budgets.toml [traversal]` (today: `# Populated by P2-12`) with the per-fixture ceilings. State the machine-independence reasoning explicitly, the way the `allocs` field docs were rewritten at P1-13: **walk counts, like alloc counts, are deterministic and machine-independent**, so `[traversal]` gates exactly from day one and does not wait for the Blacksmith reference runner _(18 entries, `<backend>_<fixture> = { walks, visits }`)_
- [x] Extend `tests/tooling/davinci-budgets.test.ts` to reconcile `[traversal]` against the probe ids **in both directions** — today it validates only the `[bench]` registry — so a fixture without a ceiling, or a ceiling without a fixture, fails _(landed as its own suite, `tests/tooling/davinci-traversal-budgets.test.ts`, so neither file passes the 350-line source budget; all three gates proven by injected failures)_
- [x] Pin the **phase-2 improvement target** in `budgets.toml`, in the quantities that are machine-independent (fused-compile `allocs` on the ladder and walk counts), with wall time explicitly report-only until the Blacksmith recording. Record the **phase-start rev** in the same table, so the phase-end re-bench has a defined "before" _(`[target.phase-2]`, phase-start rev `232870a8`)_
- [~] Corpus expansion audit for the surfaces phase 2 touches (charter #31, C-14): `node tools/davinci/corpus-coverage.mjs --check` with its scope proof; any S2 construct with no real-project instance is recorded as "not represented — matrix fixtures only" _(**audit done** against the committed 142/142-hydrated report — `mathml` is the one S2 element kind with zero real-project instances; **`--check` itself is not evaluable** by CI or a normal working tree, which is a plan bug the record states and carries to the exit gate's C-14 line)_

**Acceptance:** `walk-baseline.md` committed and reproducible (two runs identical); `[traversal]` non-empty and reconciled by the extended budgets suites (TS-3); the target table present with non-zero values and the phase-start rev recorded; corpus-coverage `--check` green with scope proof (TS-12) — **the one clause not met, see the record**; TS-11 empty (the probe counts, it does not change behavior — the P0-3 precedent). **Review point:** the maintainer sets the target _numbers_; the artifact's existence and non-zero-ness is what CI checks, and the assurance doctrine forbids choosing them later to fit the result. **Deps:** none. **Must merge before P2-9.** **Non-goals:** the observer-based walk counter (P2-12b); recording the Blacksmith wall baselines and the CI bench lane (P0-4's open pending, not phase 2's to close); `[resource]` corpus-batch RSS, which still has nowhere to live until P5-11 and where P1-11's 766.5 MB → 171.1 MB figure is still stranded.

## P2-12b — Fused build path + walk-count instrumentation

**Deliverable:** `vize build` parsing straight to S2, with the traversal budget measured and gated.

**Steps:**

- [ ] Parse → S2 direct; **S1 is a capability**, materialized on demand only for consumers that need losslessness (formatter, lint autofix)
- [ ] Walk-count instrumentation through the P2-3 budget-counting observer, with fused groups reported as one walk (P2-3's law makes this honest)
- [ ] Gate against `budgets.toml [traversal]` / `walk-baseline.md`
- [ ] Answer the open question **"Fusion depth for the build path"**, which explicitly asks for a phase-2 prototype: measure walk count and whether fusing semantic-fact population into lowering costs diagnostic quality. Synthesized attributes fuse cleanly; anything needing lookahead (sibling `v-else`, slot collection) stays region-local. Record the answer in `open-questions.md`, converting the entry to a decided stub per that doc's own convention
- [ ] Decide provenance policy for the fused walk (off or ring-buffered in the CLI, fully materialized in resident/DevTool mode) with the measurement that chose it

**Acceptance:** TS-22 established — traversal count ≤ the `[traversal]` ceilings in `budgets.toml` on the fixture ladder, measured in CI and gated **exactly** (the alloc-gate reasoning, no tolerance); the walk law pinned by an ordinary integration test so it runs in the default `cargo test --workspace` lane; the fusion-depth open-questions entry updated with its measurement; fused-path benches' `allocs` recorded (TS-10); TS-11 empty for the fused path's output; TS-1, TS-13. **Deps:** P2-12a, P2-11, P2-3. **Non-goals:** optimization tiers scaling budgets (P3-10); the salsa resident tier and the snapshot tree (phase 5); making S1 unconditional; SSR/Vapor fusion (phase 3).

## P2-13 — Folio-after-change, `vize repro`, timing JSON

**Deliverable:** the ICE policy made real (charter #30) plus the per-pass dump controls.

**Steps:**

- [ ] `--folio-after-change` (hash-gated: print a pass's folio only when its hash changed) and `--folio-dir <path>`, on `davinci-opt` and the CLI compile path
- [ ] Panic handler writes `repro.folio` — last-good stage dump + pipeline string + config — and the build reports **that file** as failed while other files continue, never silently degrading to possibly-wrong output
- [ ] `vize repro <file>` replays it. This is a **new** command: there is no `repro` module in `crates/vize/src/commands/` and no `Repro` variant in `crates/vize/src/cli.rs:19`'s enum, so the task adds both plus the module declaration in `crates/vize/src/commands.rs`
- [ ] Timing JSON per the **P0-11** profile-export schema ([`profile-export.schema.json`](./profile-export.schema.json)) — the provisional text's "P0-4 schema" was a miscitation; P0-4 is `budgets.toml`

**Acceptance:** TS-23 established — an injected panic produces a `repro.folio` and `vize repro` replays to the **same** failure, asserted by exact equality on the failure, not a substring; the file-scoped property asserted as an exact file set (a batch with one panicking file still emits every other file); the timing JSON validates against the schema (TS-15); TS-1, TS-13. **Deps:** P2-4, P2-3. **Non-goals:** `folio-reduce` (P3-14); the DevTool pass-timeline UI (C-3); crash telemetry or upload; auto-fallback on internal errors, which charter #26 forbids outright.

## P2-14 — `no_std` boundary audit + wasm32-wasip2 lanes

**Deliverable:** the audit the open question calls for, then the CI lanes it licenses. **The audit comes first** — the workspace makes no `no_std` claim until it says so.

**Steps:**

- [ ] Audit which dependencies genuinely support `no_std + alloc`: the oxc crates (which `vize_carton` and therefore everything downstream depend on), the map crate P2-1 picks, `lightningcss`, `compact_str` (which `vize_carton::String` aliases), `phf` (the interner's well-known table); and which are `std`-bound by nature — rayon (threads), salsa (resident-tier only), the CLI's filesystem and process layers
- [ ] Document the approved boundary in a committed plan doc, including the P2-4 proc-macro crate as an approved `std` host-build edge
- [ ] Separate the **core-compile lane** (`vize_davinci`, `vize_disegno` only) from the full-CLI lane, which stays `std`
- [ ] Add the CI jobs to `.github/workflows/check.yml`: `cargo build -p vize_davinci -p vize_disegno --target wasm32-wasip2` and a `--no-default-features` build. **No `wasm32-wasip2` lane exists in any workflow today** — this task creates it, and it is required for the new crates only
- [ ] Note that `vize_davinci` has no `[features]` section today, so `--no-default-features` is currently vacuous; the audit states what feature seam (if any) the crates should grow rather than leaving the flag decorative
- [ ] Per P1-12's docs-truth precedent, the `no_std` claim must not appear in `docs/content/**` before the audit makes it true

**Acceptance:** TS-24 established and **required** for `vize_davinci` and `vize_disegno`; audit doc committed; both lanes green; the guarded size asserts from P2-1/P2-5a/P2-7 prove their purpose by compiling on a 32-bit target. **Review point:** the maintainer approves the boundary — which dependency edges are accepted as `std`, and which crates the claim covers. **Deps:** P2-5a. **Non-goals:** converting existing crates to `no_std` (the audit says which _could_, it does not do it); the WASI component model as the out-of-process contract transport (charter #15, phase 6); wasm blob size budgets (charter #19).

## P2-15 — Metamorphic suite v1

**Deliverable:** the mutator suite with per-mutator equivalence justifications — because these mutations are _not_ universally semantics-preserving in Vue.

**Steps:**

- [ ] Mutators: attribute reorder, pass-through `<template>` wrap, text-node split/merge, whitespace-insignificant edits
- [ ] **Each mutator ships an equivalence justification and exclusion predicates**: no reordering across duplicate keys or across `class`/`style` merge-order-sensitive attributes; wraps only where root and slot semantics are unchanged; whitespace only within Vue's condense rules
- [ ] A mutator with no safe applicability at a site **skips** that site rather than mutating it, and the skip is **counted** — a suite that silently degenerates to zero mutations must fail, the scope-proof discipline TS-11 established
- [ ] Oracle: S2 folios identical modulo declared normalization (the `folio-format.md` rules), compared as full normalized artifacts
- [ ] Commit the matrix fixture plane. `tools/davinci/matrix-gen.mjs` defaults to `tests/fixtures/davinci-matrix/`, which **is not in the tree** — P0-12 landed the deterministic generator with a `--check` staleness mode but no committed fixtures. Commit the element-kind × directive plane and wire `--check` into `tests/tooling/davinci-matrices.test.ts`
- [ ] Runs over the matrix fixtures **and** a corpus shard in CI

**Acceptance:** TS-21 established — the suite runs in CI over both sources with a scope proof (mutations applied and skips counted; a zero-mutation run fails); TS-12 green for the newly committed fixtures with the staleness check demonstrably failing on an injected edit; TS-13. **Review point:** the per-mutator equivalence justifications — an unjustified mutator is an oracle asserting a wrong expected value, which assurance §4 calls worse than no assertion. **Deps:** P2-5b, P2-8. **Non-goals:** S3 folio equivalence (phase 3); mutators needing semantic facts to decide applicability (phase 4); `folio-reduce` (P3-14); mutating the corpus submodules in place — copies only, the P0-13 convention.

## P2-16 — JSX lowering re-targets S2

**Deliverable:** `vize_atelier_jsx` lowering to Disegno instead of relief, which is the neutral core's first real fairness test.

**Steps:**

- [ ] `lower_source` at `crates/vize_atelier_jsx/src/lib.rs:206` — signature `lower_source<'a>(bump: &'a Allocator, allocator: &oxc_allocator::Allocator, source, lang)` — produces S2 rather than a relief `RootNode`; the crate-private `lower_source_with_compat` (`lib.rs:229`) follows
- [ ] Record whether the JSX hot path's deliberate bypass of `MarkupDocument::from_jsx` can now go. That bypass exists because Relief is Vue-shaped — it is the symptom the neutral core is supposed to remove, so its survival or removal is the honest fairness measurement
- [ ] Differential lane in the house shape for the JSX path

**Acceptance:** the babel-compat oracle green on the new path — `cargo test -p vize_atelier_jsx` (`babel_compat_oracle`), TS-6, with the nine committed snapshots unchanged; the JSX corpus projects' rows in TS-11 empty; differential lane zero divergence (TS-25); TS-1, TS-13. **Deps:** P2-11. **Non-goals:** rule-corpus fairness convergence (phase 4, TS-39) — this task _measures_ the gap, it does not close it; Svelte/Solid input dialects; deleting the relief JSX lowering, which Patina still consumes until it re-bases in phase 4.

## P2-17 — IR contract review milestone

**Deliverable:** a signed-off checklist — the last cheap-fix window before caches, Spolvero and external consumers depend on the S2 format.

**Steps** — the checklist, against the prior-art rules imported from LLVM's three expensive regrets:

- [ ] **No redundant encodings**: every S2 field is semantic **xor** derived-and-cached, never both (the pointee-type regret: ~7 years to remove)
- [ ] **No constructor-time folding**: folding happens in exactly one designated pass per stage (the top infinite-loop source)
- [ ] **The escape variant has pessimal documented semantics** from day one — P2-5b's decision is reviewed here against the `undef`/`poison` regret
- [ ] **Spans survive lowering**: every S2 op traces to an authored SFC span
- [ ] **`schema_version` on every agent-visible artifact** (devtool.md's data layer requires it: folio format, profile export, remark and fact-table schemas) so Spolvero negotiates and refuses mismatches loudly
- [ ] **Provenance survives failure**: partial S2 kept on error (P2-8's commitment, verified here)

**Acceptance:** the signed-off checklist committed. The mechanical half is machine-checked and must land as tests, not prose: a corpus-wide assertion that every S2 op's span resolves into its authored SFC, and a folio-level assertion that `schema_version` is present and negotiated. **Review point** for the judgement half — this milestone exists precisely because these are the cheap fixes that become expensive once formats have consumers. **Deps:** P2-11, P2-12b, P2-13. **Non-goals:** S3 contracts (P3-5's op reference does the same job one stage later); freezing the format for external consumers, which is phase 6's contracts GA; a stability guarantee — charter #23 keeps internal formats free to break until then.

## P2-18 — Spolvero feed v1

**Deliverable:** the observer's folio output as a consumable feed, rendered in the existing inspector.

**Steps:**

- [ ] P2-3's folio-printing observer writes a folio directory with a payload schema carrying `schema_version` (devtool.md's data-layer requirement)
- [ ] `vize_curator`'s inspector renders S1/S2 pages — `crates/vize_curator/src/inspector/payload.rs` (`InspectorPayload`, `build_payload`, `serialize_payload`) — next to the existing croquis alias. The alias itself lives in `crates/vize_vitrine/src/wasm/analyze.rs:312-315`, which carries both the deprecated `vir` key and nested `folio.croquis` (P0-10 corrected the location; the inspector payload never carried it)
- [ ] **Registry gap to close in this task:** [`test-suites.md`](./test-suites.md) has no suite covering the Spolvero feed payload. Add one there in the same PR — the registry is the source of TS-ids and a gate naming an unregistered suite is a plan bug, so this task must not invent an id here

**Acceptance:** the feed payload validates against its committed schema, gated by the newly registered suite; the croquis alias keeps working byte-identically (`folio.croquis` and `vir` both present). **Review point:** that the playground actually shows the stage ladder for a compiled SFC — a rendering claim no CI job evaluates today, which is why it is marked rather than dressed up as a gate. TS-3, TS-13. **Deps:** P2-4, P2-3. **Non-goals:** the `vize devtool` local server (C-7); the transport decision (P2-19); provenance navigation and remarks rendering (C-5, needs P3-13); the Fresco TUI (C-8).

## P2-19 — DevTool protocol spike

**Deliverable:** the open question closed with a working prototype, and the spike disposed of deliberately.

**Steps:**

- [ ] Prototype the three candidates against the P2-18 feed: JSON-lines stream, served files, or a content-mapper-style JSON-RPC (`vize content-mapper` is the existing precedent for the last)
- [ ] Evaluate against the three consumers devtool.md names — browser playground (`vize_vitrine` wasm), local server, and `--format agent` output under `vize_doctor::ai_context` budgeting — plus the `schema_version` negotiation requirement
- [ ] Record the decision in [`devtool.md`](../devtool.md) and convert the `open-questions.md` "DevTool protocol" entry into a decided stub pointing at it, per that document's own convention

**Acceptance:** decision recorded in `devtool.md`; the open-questions entry is a stub; the spike code is either kept (with tests and a home) or deleted, and the PR says which and why. **Review point** — a transport choice is a judgement, and "spike code left lying around" is the failure mode this acceptance names. **Deps:** P2-18. **Non-goals:** implementing the chosen transport at production quality (C-7); the JS plugin API shape (charter #29, phase 4/5 spike); authentication or remote access.

## P2-20 — Phase exit

**Deliverable:** the exit gate in [phase-2.md](./phase-2.md), evaluated and recorded there, in phase 0's and phase 1's manner: **a line is ticked only when it is satisfied, an unticked line names its blocker, and no line's wording is softened to make it tickable.**

**Steps:**

- [ ] Evaluate every line of the exit gate in [phase-2.md](./phase-2.md) and record the evidence inline
- [ ] Delete the in-phase old paths (P2-9's transform lane flag, P2-11's `VIZE_DAVINCI_DOM=legacy`) or record each as an unfinished deletion with an owner and an issue — charter #26's fix-forward switch happens here
- [ ] Restate the retirement condition for the `davinci-differential` lanes (phase 1's and phase 2's), which are written to live "for one release"
- [ ] Re-bench the phase-start rev and this tree, compare against the P2-12a target, and record the result — including a miss, if it is one
- [ ] Corpus waiver ledger reviewed and empty (C-16)

**Acceptance:** the exit gate in [phase-2.md](./phase-2.md), with every line either ticked with its evidence or carrying a named blocker. **Deps:** all of P2-1..P2-19. **Non-goals:** re-cutting phase 3 — that is phase 3's own re-cut at this exit, per the plan README; closing P0-4's Blacksmith pending; unblocking P1-8.
