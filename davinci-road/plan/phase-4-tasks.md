# Phase 4 — Task contracts, P4-1a through P4-5c

> [!NOTE]
> Full contracts for [Phase 4 — Consumer Convergence](./phase-4.md), early re-cut 2026-09-21. Each task carries Start gate / Lane / Deliverable / Steps / Acceptance / Deps / Non-goals. The phase file holds the re-cut record, the lanes, the critical path, the TODO index (where boxes are checked) and the exit gate; [phase-4-tasks-later.md](./phase-4-tasks-later.md) and [phase-4-tasks-last.md](./phase-4-tasks-last.md) continue the contracts under the 350-line source budget. House patterns from phase 2 apply by name: fixtures before behavior (#21), exact oracles only (TS-13), the P1-9 differential-lane shape for every replaced lane (TS-25), every new bench lands with its measured `allocs` (TS-10), and a task that finds its scope wrong updates this file in the same PR.

## P4-1a — Fact API core

**Start gate:** startable now — P3-independent.

**Lane:** A

**Deliverable:** `vize_davinci::fact`, the one typed query surface of charter #5/#8: fact groups as const data, static demand declarations per consumer, a `FactManager` that computes exactly the demanded union, and the debug-build undeclared-access detector with the stratification check. The crate stays `#![no_std]` + `alloc`.

**Steps:**

- [x] `crates/vize_davinci/src/fact.rs` + `fact/`: `trait FactGroup { const ID: AnalysisId; const STRATUM: u8; const DEPENDS: Demand; type Key; type Value; }` — group identity **reuses** `pass::preserved::AnalysisId`, so a pass's `Preserved` mask names fact groups directly (one identity space, capped by `MAX_ANALYSES = 64` with its existing const assertion)
- [x] `Demand(u64)` built in `const` context (`Demand::NONE.with(G::ID)`); each consumer declares `const DEMAND: Demand`
- [x] Stratification as a `const fn` over the registered group descriptors: a group may depend only on strictly lower strata, so a demand cycle is unrepresentable (the Swift anti-lesson); violations are compile errors proved by `compile_fail` doctests
- [x] `FactManager` computes the transitive demand closure in stratum order through registered producers, each group at most once per artifact (a process-global counter pins it), and serves `get::<G>()` as a borrowed table
- [x] Debug detector: under `debug_assertions` a `FactView` carries its consumer's `Demand`; an undeclared access returns `FactError::Undeclared { consumer, group }` and bumps a counter; the release shape is a zero-sized type, const-asserted
- [x] Register the TS-35 command in [test-suites.md](./test-suites.md): `cargo test -p vize_davinci --test fact_demand`

**Acceptance:** `cargo test -p vize_davinci --test fact_demand --test fact_manager` green; `cargo test -p vize_davinci --doc` runs the two stratification `compile_fail` doctests; an undeclared access is asserted equal to the exact `FactError` value; the once-per-artifact counter is pinned; TS-24 (`--target wasm32-wasip2`, with and without default features) green; TS-1, TS-13; TS-11 empty (nothing consumes the API yet).

**Deps:** none (phase-2 exit).

**Non-goals:** migrating a tracker (P4-3a…P4-3f); α/β serialization (P4-2); pass-manager invalidation (P4-1b); salsa or cross-compile caching (phase 5).

**Landed 2026-09-22:** `vize_davinci::fact` with the TS-35 lane, both stratification `compile_fail` doctests and the pinned once-per-artifact counter — see the [P4-1a record](./phase-4-records/p4-1a.md).

## P4-1b — Fact preservation and recompute mode

**Start gate:** startable now — P3-independent.

**Lane:** A

**Deliverable:** facts wired to the P2-2 pass manager: post-hoc `Preserved` sets (LLVM new-PM import), named preservation groups, invalidation of every group a pass does not preserve, and a debug recompute-and-compare mode that catches a pass lying about what it preserves.

**Steps:**

- [x] `crates/vize_davinci/src/fact/preserve.rs`: `FactManager::after_pass(&PassDesc)` drops every group outside `desc.preserved`; named groups (`PRESERVE_STRUCTURE`, `PRESERVE_BINDINGS`) are `const Preserved` values
- [x] `FactVerifyObserver` (P2-3 static dispatch, release ZST): after each pass, recompute every group the pass claims to preserve and compare by exact equality
- [x] A fixture pass that claims `Preserved::ALL` while mutating is rejected with the exact `FactError::StalePreserved { pass, group }`

**Acceptance:** `cargo test -p vize_davinci --test fact_preserve` green including the lying-pass fixture; bench pair `davinci_fact_query_observed` / `davinci_fact_query_unobserved` registered in `budgets.toml [bench]` with measured, identical `allocs` (TS-10 — the P2-3 zero-cost shape); TS-22 walk counts unchanged; TS-1, TS-13.

**Deps:** P4-1a.

**Non-goals:** caching facts across compiles (P5-1); S3 phase ordering (phase 3).

**Landed 2026-09-22:** `after_pass` invalidation, `PRESERVE_STRUCTURE` / `PRESERVE_BINDINGS` over the new `fact::ids` table, the `FactVerifyObserver` recompute-and-compare mode with the lying-pass fixture, and the 19/19-alloc bench pair — see the [P4-1b record](./phase-4-records/p4-1b.md).

## P4-2 — Fact-group alpha beta split

**Start gate:** startable now — P3-independent.

**Lane:** A

**Deliverable:** every fact group may declare an α form (owned, versioned, explicit `export`, the Lean environment-extension import) beside its β form (in-memory index rebuilt on demand). α is what P5-2's per-SFC summary serializes.

**Steps:**

- [x] `trait AlphaExport: FactGroup { const ALPHA_SCHEMA: u16; type Alpha: Folio; fn export(..) -> Self::Alpha; fn import(Self::Alpha) -> FactTable<Self>; }` in `crates/vize_davinci/src/fact/alpha.rs`
- [x] α values are owned and `'static`-asserted (the P1-11 arena/cache contract); each α page prints `schema_version` (the P2-17 rule)
- [x] `davinci-road/plan/fact-alpha-schemas.md` documents every α group's key, value and version; a Rust test reads it via `include_str!` and fails on an undocumented group

**Acceptance:** `cargo test -p vize_davinci --test fact_alpha` — `import(export(t)) == t` exactly for every group registered at merge time, TS-16 `Full`-mode byte round trip per α page, the schema-doc test proven to fail on an injected undocumented group; TS-24; TS-1, TS-13.

**Deps:** P4-1a.

**Non-goals:** persistence and fingerprints (P5-1, P5-2); a cross-file store (P4-3b, P4-3f own the project groups).

**Landed 2026-09-22:** `AlphaExport`, the versioned `AlphaDocument`, `fact-alpha-schemas.md` with its executable check (proven to fail on an undocumented group) and exact α/β round trips — see the [P4-2 record](./phase-4-records/p4-2.md).

## P4-3a — Bindings and undefined-refs fact groups

**Start gate:** startable now — P3-independent.

**Lane:** B

**Deliverable:** `Croquis.bindings`, `binding_spans` and `undefined_refs` become the fact groups `Bindings` and `UndefinedRefs` in `crates/vize_croquis/src/facts/`, with the existing tracker code as their population pass (semantic-engine.md #1) and every non-projection reader on a declared demand. Measured readers today: `Croquis.bindings` 77 sites (canon 45, atelier_sfc 16, maestro 8, atelier_core 2, croquis_cf 2, vitrine 2, atelier_jsx 1, patina 1), `undefined_refs` 8 (canon 7, patina 1).

**Steps:**

- [x] `vize_croquis` gains a `vize_davinci` dependency; groups implement `FactGroup`, keyed as written in the P4-2 schema doc (binding name / `SymbolId`; S2 `NodeId` re-keying waits for S2 to carry script scope)
- [x] Declarative spec + naive evaluator in `crates/vize_croquis/src/facts/spec/` (the Polonius discipline); TS-34 compares it with production over the P2-15 matrix plane and a corpus shard (`VIZE_DAVINCI_FACT_CORPUS`, the two test-scripts submodules, skips counted)
- [ ] Switch readers to `get::<Bindings>()` / `get::<UndefinedRefs>()` **except** `crates/vize_canon/src/virtual_ts/**` and `crates/vize_maestro/src/virtual_code/**` (lane C deletes those readers in P4-5c) **and the legacy compile lane** (`vize_atelier_core/src/lane/`, `vize_atelier_jsx`, the `vize_atelier_sfc` compiler), which reads the analysis it builds in the same pipeline as a fused attribute (semantic-engine.md #2): a demand there costs a table build per compile — measured 0.02–1.02 % of an SFC compile on the P0-2 ladder, a regression charter #22 refuses — and the lane retires with the legacy transform lane (charter #26); the struct fields stay as producer storage until then _(scope amended 2026-09-22, see the record)_. The DOM reader `vize_atelier_dom/src/compile/croquis_facts.rs` demands `Bindings` (`dom/croquis-projection`). The legacy compile lane stays fused.
- [x] Regenerate [croquis-consumption.md](./croquis-consumption.md) (`rust-script tools/commands/davinci/croquis-consumers.rs --write`)

**Acceptance:** TS-34 exact agreement with scope proof (a zero-comparison run fails); TS-35 zero undeclared accesses in `cargo test --workspace` (debug); the regenerated matrix shows the two fields read only from lane-C paths, the legacy compile lane and producer-side writes, each remaining file classified in the record (amended with the step above); TS-9 lint/check fixtures, TS-40 digests and TS-11 compile/check/lint surfaces unchanged; TS-12 green.

**Deps:** P4-1a.

**Non-goals:** projection readers (P4-5c); the reactivity lattice (P4-3d); new diagnostics.

**Slice 2 landed 2026-09-22:** both groups, TS-34 exact agreement on every plane (corpus shard 670 + 773 artifacts, zero divergences) and the six remaining analysis readers on a declared demand. The DOM projection demands `Bindings`. The task stays open for the legacy compile lane — see the [P4-3a record](./phase-4-records/p4-3a.md).

## P4-3b — Component-usage fact groups

**Start gate:** startable now — P3-independent.

**Lane:** B

**Deliverable:** `component_usages` (61 resolved read sites), `used_components` (40), `component_registrations`, `template_info`, `element_ids` and the `render_tree.rs` edge model become `ComponentUsages` and `RenderTree` groups keyed by **(caller file, resolved component identity)** through the module graph (`crates/vize_croquis_cf/src/facts/`), never by local import name — the semantic-engine contract P4-9b and P4-11b build on.

**Steps:**

- [x] Per-file group in `crates/vize_croquis/src/facts/components.rs`; project group in `crates/vize_croquis_cf/src/facts/render_tree.rs` resolving identities with the existing `rules/component_resolution.rs` logic moved behind the group
- [x] Edges are registration-order independent and **stale edges are removed on re-resolution**
- [x] TS-34 spec + naive evaluator; readers outside lane C switched; matrix regenerated

**Acceptance:** TS-34 as in P4-3a; regression fixtures with exact diagnostic sets for aliased imports, re-exports, same-basename components in different directories, both child registration orders, and a stale edge after re-resolution; TS-35; cross-file lint snapshots unchanged; TS-12.

**Deps:** P4-1a.

**Non-goals:** props/emits contract checking as new rules (a later consumer); complexity (P4-9b); HTML composition (P4-11b).

## P4-3c — Unused-bindings fact group and lint consumer

**Start gate:** startable now — P3-independent.

**Lane:** B

**Deliverable:** `unused_bindings` becomes the `UnusedBindings` group and gains its first lint consumer, closing ledger entry FN-2 (0/130 corpus-shard recall, 0/90 matrix, 0/4 miniature): the rule `vue/no-unused-setup-bindings` reads the group with a declared tier and domain (script-setup bindings not read by template, script or style `v-bind()`).

**Steps:**

- [ ] Group + TS-34 spec; the rule lives in lane B's own directory `crates/vize_patina/src/rules/facts/` (fact-driven rules migrated by lane B) and is registered by one line in the rule registry — the only edit outside lane B's paths
- [ ] Revisit the `__davinci_seeded_unused` seed name per the ledger caveat (underscore-prefixed names are exempt by convention)

**Acceptance:** `rust-script tools/commands/davinci/seed-defects.rs --assert` reports class (b) recall 130/130 on the shard, 90/90 on the matrix stubs and 4/4 on the miniature set; `tests/_fixtures/davinci-fpfn/expected/assert-report.json` updated in the same PR; [ledger-fn.md](./ledger-fn.md) FN-2 flipped to `fixed`; TS-38 no new FP candidate on the shard; TS-35.

**Deps:** P4-1a, P4-3a, P4-6a.

**Non-goals:** default-preset enablement of `vue/no-undefined-refs` (FN-1, an FP-audit change); whole-project dead code.

## P4-3d — Reactivity merges into the S3 lattice

**Start gate:** startable now — P3-independent (P3-2 landed).

**Lane:** B

**Deliverable:** Croquis's `ReactivityTracker` (566 lines; readers: croquis_cf 22, maestro 6, canon 3, patina 1, vitrine 1) populates the P3-2 lattice group in `vize_impeto` — **one lattice, not two** — exposed as the `Reactivity` fact group with the `proven/refuted/unknown` verdict axis.

**Steps:**

- [ ] `crates/vize_croquis/src/facts/reactivity.rs` maps `ReactiveKind` sources onto the lattice's value axis; the only `vize_impeto` edit is one new constructor file (P3-6/P3-15 own the rest of that crate)
- [ ] Readers of `Croquis.reactivity` / `ReactiveKind` outside lane C switched to the group
- [ ] TS-34 spec over the lattice's join laws

**Acceptance:** TS-34 agreement over the matrix plane and a corpus shard; P3-2's `[s3-reactivity-folio]` fixtures byte-identical (TS-16/TS-17); croquis_cf reactivity diagnostics unchanged (TS-9); TS-35; TS-12.

**Deps:** P4-1a, P3-2.

**Non-goals:** lattice theorems (P3-15); effect graphs (P4-3e); Vapor planning (P3-6).

## P4-3e — Effect-graph fact group

**Start gate:** gated on P3-6 — Vapor on S3 is the consumer this group is shaped for.

**Lane:** B

**Deliverable:** `build_effect_graph_from_*` (`effect_graph.rs` + `effect_graph/`) becomes the `EffectGraph` group demanded by Vapor's S3 effect grouping, Doctor (`crates/vize/src/commands/doctor/analysis.rs`) and croquis_cf complexity.

**Steps:**

- [ ] Group + TS-34 spec in `crates/vize_croquis/src/facts/effect_graph.rs`
- [ ] Doctor and croquis_cf read it through declared demands; Vapor's demand is added in P3-6's crate only by one declaration line

**Acceptance:** TS-34; TS-35; Doctor snapshots unchanged; TS-33 unchanged; TS-12.

**Deps:** P4-1a, P4-3d, P3-6.

**Non-goals:** new effect rules; S3 grouping itself (P3-6).

## P4-3f — Provide inject and race fact groups

**Start gate:** startable now — P3-independent.

**Lane:** B

**Deliverable:** `Croquis.provide_inject` (croquis_cf 26 sites, vitrine 2) and `race_conditions` become `ProvideInject` and `RaceConditions` groups; cross-file pairing resolves through P4-3b identities; `crates/vize_croquis_cf/src/rules/{provide_inject,race_conditions}.rs` read demands.

**Steps:**

- [ ] Groups + TS-34 specs; readers switched; matrix regenerated

**Acceptance:** TS-34; TS-35; `vize lint` cross-file fixtures and Doctor snapshots unchanged; TS-12.

**Deps:** P4-1a, P4-3b.

**Non-goals:** the soundness verdict (P4-4a); new rules.

## P4-4a — Orphan verdicts for non-effect products

**Start gate:** startable now — P3-independent.

**Lane:** B

**Deliverable:** a recorded verdict — productize with a named consumer and tier, or demand-gate to zero cost (charter #5) — for every product the consumption matrix lists without consumers (`Croquis.hoists`/`HoistTracker`, `Croquis.symbols`, `Croquis.used_directives`, the `reactivity_overlay` family, `SetupContextTracker`) and a corpus-soundness verdict for provide/inject and race rules; the coarse `DrawerOptions`/`SfcCroquisOptions` presets (`full`, `for_lint`, `for_compile`, `for_declaration`) replaced by demand sets.

**Steps:**

- [ ] Evidence per product: TS-34 agreement plus a corpus-shard TS-38 scan for the provide/inject and race rules; verdicts in the task record
- [ ] Demand-gated groups are not computed unless demanded — proved by an alloc-identical bench pair with and without the gated group registered

**Acceptance:** TS-12 — the regenerated matrix's "no external consumers" section lists only demand-gated products, each naming its gate; the bench pair registered in `budgets.toml` with identical `allocs` (TS-10); `grep -rn "SfcCroquisOptions::for_lint\|DrawerOptions::for_compile" crates` empty; TS-9/TS-11 unchanged.

**Deps:** P4-3a, P4-3b, P4-3c, P4-3d, P4-3f.

**Non-goals:** `EffectGraph` (P4-4b); deleting a product that has a consumer.

## P4-4b — Effect-graph verdict

**Start gate:** gated on P3-6 — the verdict depends on whether Vapor consumes the group.

**Lane:** B

**Deliverable:** the `EffectGraph` verdict: productized for Vapor, Doctor and complexity, or demand-gated for compile and kept for Doctor, recorded with TS-34 corpus evidence.

**Steps:**

- [ ] Measure the group's consumers after P3-6; record the verdict and its evidence

**Acceptance:** TS-12 shows `EffectGraph` with ≥1 consumer on every demanding surface or gated; TS-34 corpus evidence recorded.

**Deps:** P4-3e, P3-6.

**Non-goals:** Vapor effect grouping (P3-6).

## P4-5a — One mapping model and one diagnostic post-pass

**Start gate:** startable now — P3-independent.

**Lane:** C

**Deliverable:** one span-link model, `ProjectionMapping` (generated range ↔ authored range, sub-spans, semantic links), absorbing Canon's `virtual_ts::{VizeMapping, VizeSubSpan, VizeSemanticLink}`, Canon's `source_map::{Mapping, SourceMap}` and Maestro's `virtual_code::source_map::SourceMap`; and one diagnostic assembly post-pass over finished `Vec<Diagnostic>` shared by `vize check` and the Maestro session, ending the session-vs-CLI dual path.

**Steps:**

- [x] `crates/vize_canon/src/virtual_ts/mapping.rs` owns `ProjectionMapping`; Maestro and Canon source-map types become thin views, then are deleted
- [x] `assemble_diagnostics(finished, &ProjectionMapping)` is the single assembly point; both the CLI batch path and `crates/vize_maestro/src/ide/diagnostics/corsa*` call it
- [x] A regression fixture drives one SFC through both entry points and asserts identical diagnostics

**Landed 2026-09-22:** `ProjectionMapping` with per-row `ProjectionMeta` (Canon's `source_map` module and Maestro's virtual-code source map deleted, `SfcSourceMap` a thin view, TS-40 byte-identical) and `vize_canon::projection::assemble_diagnostics`, the one post-pass both `vize check` and the Maestro session call per authored file, pinned by a real-Corsa fixture that drives one SFC through both entry points; every rule that ran on one surface only now runs on both — see the [P4-5a record](./phase-4-records/p4-5a.md) for the convergence ledger and the fourth mapping model P4-5c deletes.

**Acceptance:** TS-40 digests byte-identical (`node --test tests/tooling/davinci-ts40-projection.test.ts tests/tooling/lsp-davinci-ts40-projection-diagnostics.test.ts`); `grep -rn "virtual_code::source_map::SourceMap" crates` empty; TS-7 LSP smoke; TS-9 check fixtures unchanged; TS-1, TS-13.

**Deps:** none (phase-2 exit).

**Non-goals:** a new generator (P4-5b); deleting generators (P4-5c); the S4 document (P3-9).

## P4-5b — S2 projection as an S4 target

**Start gate:** gated on P3-9 — the projection emits through P3-9's span-carrying S4 document; a projection-private emitter would be a second emission design (charter #26).

**Lane:** C

**Deliverable:** the virtual-language projection generated from S2 (template) plus the retained oxc program (script) as an S4 target emitting `ProjectionMapping` rows, in `crates/vize_canon/src/projection/`, with the TS-25 differential lane comparing it against the current generators.

**Steps:**

- [ ] Expressions project through P2-5b's `ExprDialect` capability; `ExprRef::Opaque` follows its pessimal laws and is counted per class
- [ ] Differential comparator `crates/vize_canon/tests/davinci_projection_differential.rs` behind `davinci-differential`: `vize check` diagnostic sets and authored-anchor resolution compared exactly; generated text is free
- [ ] Corpus-runnable entry with its command recorded; plain-suite coverage witness pins the comparison count

**Acceptance:** TS-25 zero divergence over the TS-40 matrix and the `vize check` corpus with scope proof; TS-24 unaffected; TS-1, TS-13.

**Deps:** P4-5a, P3-9.

**Non-goals:** MoonBit projection (P6-4); incremental reuse (P5-7); route types (P4-10b).

## P4-5c — Consumers switch and three generators deleted

**Start gate:** startable now — P3-independent (waits behind P4-5b).

**Lane:** C

**Deliverable:** Canon's CLI and editor paths, the content-mapper protocol, Maestro and Patina's type-aware lane consume the P4-5b projection, and all three generators are deleted: `crates/vize_canon/src/virtual_ts/` (generator half), `crates/vize_maestro/src/virtual_code/` and `crates/vize_croquis/src/virtual_ts/`.

**Steps:**

- [ ] Switch each consumer, then delete; the remaining lane-C `Croquis` field reads disappear with the generators
- [ ] Re-baseline TS-40 on the new side with the P4-5b differential run as evidence

**Acceptance:** TS-11 check surface empty with scope proof; content-mapper protocol fixtures byte-compatible (the external tsgo protocol); `grep -rn "VirtualTsGenerator\|generate_virtual_ts_with_croquis" crates` empty; the consumption matrix shows zero direct field reads for every migrated group; TS-7, TS-9, TS-40.

**Deps:** P4-5b, P4-3a, P4-3b.

**Non-goals:** Corsa session reuse (P5-8); block-level reuse (P5-7).
