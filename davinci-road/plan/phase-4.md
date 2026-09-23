# Phase 4 — Consumer Convergence

> [!NOTE]
> **Early re-cut 2026-09-21, while phase 3 is still live.** The plan README's rule is that a phase is re-cut at its predecessor's exit and that a provisional task cannot be picked up until it carries the full contract. Phase 4 is re-cut **before** the phase-3 exit because almost all of it consumes only S2 (phase 2, exited 2026-09-12), and the maintainer's completion target (**2026-09-23**, ahead of the Vue Fes Japan 2026 presentation on **2026-10-24**) leaves no room to idle the consumer half of the program behind S3 work it does not read. The [plan README](./README.md#phase-files) records the early re-cut rule this phase follows: every task states its **start gate** — either _startable now_ (P3-independent) or _gated on_ a named phase-3 task — and a gated task is not picked up until that task lands. The exit gate below is unchanged and still requires the phase-3 exit (P4-17 depends on P3-16). The target date orders work; it does not relax a gate, a ratchet or a waiver ledger (roadmap: phases are ordered, not scheduled).

**The per-task contracts live in [phase-4-tasks.md](./phase-4-tasks.md) (P4-1a…P4-5c), [phase-4-tasks-later.md](./phase-4-tasks-later.md) (P4-6a…P4-9b) and [phase-4-tasks-last.md](./phase-4-tasks-last.md) (P4-10a…P4-17)** — Start gate / Lane / Deliverable / Steps / Acceptance / Deps / Non-goals for all 39 tasks. This file keeps the phase-level record: what the re-cut changed, what phase 3 carried in so far, the start gates, the parallel lanes, the critical path, the TODO index and the exit gate. [`davinci-phase4-contract-status.test.ts`](../../tests/tooling/davinci-phase4-contract-status.test.ts) enforces the structure: every index entry links a full contract, every dependency names a real task, the graph is acyclic, a _startable now_ task has no open phase-3 dependency, lanes own disjoint paths, and the exit-gate lines below survive verbatim.

## What the re-cut changed

Each item is a scope or design change forced by what the tree measures today (2026-09-21, `origin/main` at `b6d258df`), not a reformat.

1. **Three virtual-TS generators, not two.** The provisional P4-5 named `vize_canon/src/virtual_ts/` (218 files, 38,675 lines) and `vize_maestro/src/virtual_code/` (13 files, 2,724 lines). A third lives in `vize_croquis/src/virtual_ts/` (10 files, 2,135 lines) and feeds Patina's type-aware lane (`vize_patina/src/linter/native_type_aware/driver.rs` calls `generate_virtual_ts_with_croquis`). There are also three mapping models: Canon's `virtual_ts::VizeMapping`, Canon's `source_map::{Mapping, SourceMap}` and Maestro's `virtual_code::source_map::SourceMap`. **P4-5 splits three ways**: P4-5a unifies the mapping model and the diagnostic post-pass (startable now); P4-5b builds the S2 projection as an S4 target (gated on P3-9's structured emission document — the architecture makes the projection an S4 target, and a projection-private emitter would be the dual lane charter #26 forbids); P4-5c switches the consumers and deletes all three generators.
2. **The orphan table is stale.** The 2026-08-13 audit listed `RaceConditionTracker` and `ProvideInjectTracker` with zero consumers. Today `vize_croquis_cf` reads `Croquis.provide_inject` (10 files / 26 sites) and `Croquis.race_conditions`, and both are surfaced by `vize lint`'s cross-file lane and Doctor (`with_provide_inject(true)` / `with_race_conditions(true)` in `crates/vize/src/commands/lint/cross_file.rs` and `doctor/analysis.rs`). The real zero-consumer products are `Croquis.hoists` (`HoistTracker`), `Croquis.symbols`, `Croquis.used_directives` and the `reactivity_overlay` family ([consumption matrix](./croquis-consumption.md)). **P4-4 splits**: P4-4a rules on the non-effect products with TS-34 evidence; P4-4b owns `EffectGraph`, whose Vapor consumer is P3-6.
3. **The rule corpus is 248 rules in 381 files, not 345 files.** The [rule-parity matrix](./rule-parity.md) measures 92 neutral-core candidates, 133 Vue-dialect-bound, 23 container-bound; SFC∩JSX 150; JSX lanes `fallback` 110 / `ir` 28 / `ir-lowered` 12 / `no-jsx-hooks` 11; croquis adoption 23 rules. 99 rules default to error severity and 149 to warning, so P4-6's "error ⇒ proven + witness" law has a concrete migration size, drained by an exemption inventory that only shrinks. **P4-8 splits into three waves** (P4-8a neutral-core, P4-8b dialect-bound, P4-8c container-bound + legacy visitor retirement), each an explicitly-marked small series whose installments are one rule family directory, so several agents can run a wave in parallel.
4. **Complexity already exists, as string counting.** `vize_croquis_cf/src/rules/complexity.rs` (347 lines + 467 in submodules) scores `cyclomatic = component_count + v-if count + v-for count + logical operators scanned from expression text` and renders through `vize_curator/src/complexity.rs` (201 lines). There is no CFG, `v-else-if` is not counted, and attribution sums a whole project. P4-9 replaces the input with S2-region CFG facts and the open question gets a written recommendation (below and in [open questions](../open-questions.md#complexity-metric-definition)).
5. **i18n is 449 keys × 3 locales plus Rust supplements, and covers half the rules.** `vize_carton/src/i18n/{en,ja,zh}.json` carry 449 keys each (already past the source-length baseline, so new keys go to `i18n_supplemental*.rs`); 127 of 248 rules have a `description` in all three locales, 121 have none, and the 56 compiler `ErrorCode` variants (`vize_relief/src/errors.rs`) have no catalog at all. P4-14 splits into renderer (P4-14a), catalog completeness over every producer (P4-14b) and `--explain` pages (P4-14c), and registers **TS-53** so the gate names a real suite.
6. **Diagnostics have no unified channel yet.** `vize_davinci::Diagnostic` (P2-1) carries a `Witness` slot with `LegacyExempt`; nothing outside the S1→S2 lowering produces it. FP-1 (script-relative offsets rendered against whole-file lines, [ledger](./ledger-fp.md)) is deferred to exactly this work. P4-6 splits into types (P4-6a), witness verifier (P4-6b) and Patina adoption with FP-1 fixed (P4-6c).
7. **Davinci stage crates are publishable now.** The P2-era firewall (published crates could not depend on `vize_davinci`/`vize_s2`) is lifted: [`davinci-stage-release-firewall.test.ts`](../../tests/tooling/davinci-stage-release-firewall.test.ts) asserts all six stage crates are publishable. Patina, Canon, Glyph and Musea may therefore take ordinary runtime dependencies on `vize_davinci`, `vize_s1` and `vize_s2`, subject to the [stage dependency policy](./stage-dependencies.md).
8. **Pug has no parser anywhere.** No crate parses pug; Maestro has a 339-line line-heuristic cursor helper (`vize_maestro/src/ide/pug.rs`) and corpus coverage scans pug line-heuristically. Charter #12's first-class S1 pug dialect is therefore its own task (P4-12c), independent of the Glyph rewrite (P4-12b).
9. **S1 is template-only.** `vize_s1` (1,381 lines) is the Vue-template surface tree; SFC block splitting lives in `vize_croquis/src/sfc/` (3,687 lines) and there is no OXC-backed lossless script wrapper. P4-12b and P4-13 consume the existing splitter plus S1 template trees and name the script wrapper as a non-goal.

## Carried from phase 3 (so far)

| Phase-3 state (2026-09-21)                                                                                        | Phase-4 tasks affected | How                                                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| **P3-2 landed** — S3 reactivity lattice fact group in `vize_impeto`, `proven/refuted/unknown` orthogonal to value | P4-3d, P4-6a           | The Croquis `reactivity` wave merges into this group instead of creating a second lattice; P4-6a reuses its verdict axis for witnesses.         |
| **P3-6 open** — Vapor on S3 is the intended `EffectGraph` consumer                                                | P4-3e, P4-4b           | Both gated on P3-6. Doctor and croquis_cf complexity stay the only consumers until then.                                                        |
| **P3-9 open** — only module-start source-map anchors exist; no span-carrying S4 document                          | P4-5b                  | Gated on P3-9's structured emission slice. P4-5a (mapping model) proceeds now and defines the projection-side row type P3-9's document carries. |
| **P3-16 open** — phase-3 exit                                                                                     | P4-17                  | Phase order: P4 cannot exit before P3.                                                                                                          |
| **P2 DOM compile allocation miss** carried to phase 3                                                             | none                   | Not P4's; P4 benches record their own `allocs` and never loosen a ratchet.                                                                      |

The ledgers carry two deferred items into this phase: **FP-1** (`type/require-typed-emits` offsets, owner P4-6c) and **FN-2** (`unused_bindings` has no lint consumer, owner P4-3c).

## Start gates and parallel lanes

**Startable now (P3-independent), 35 of 39 tasks:** every task except the four below; two of them (P4-5c, P4-10b) wait behind the gated P4-5b. **Gated on phase 3, 4 tasks:** P4-3e and P4-4b on P3-6; P4-5b on P3-9; P4-17 on P3-16.

Each lane owns a disjoint set of paths; tasks in different lanes never edit the same file, so one agent per lane can run concurrently from the first hour. The only cross-lane edits are registration touchpoints — a `mod` line in `lib.rs`, a `Cargo.toml` dependency, a one-line rule or seed-class registration, a regenerated matrix (`rule-parity.md`, `croquis-consumption.md`), a `test-suites.md` row — and those are the expected rebase conflicts. A path listed with `minus lane <L>` is carved out for lane L.

| Lane | Tasks                     | Owns (only this lane edits)                                                                                                                                                                                                                                                     |
| ---- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A    | P4-1a, P4-1b, P4-2        | `crates/vize_davinci/src/fact*`, `crates/vize_davinci/tests/fact_*`, `davinci-road/plan/fact-alpha-schemas.md`                                                                                                                                                                  |
| B    | P4-3a…P4-3f, P4-4a, P4-4b | `crates/vize_croquis/src/facts*`, `crates/vize_croquis_cf/src/facts*`, `crates/vize_croquis_cf/src/rules/provide_inject*`, `crates/vize_croquis_cf/src/rules/race_conditions*`, `crates/vize_patina/src/rules/facts/unused_setup_bindings*`, `crates/vize/src/commands/doctor/` |
| C    | P4-5a, P4-5b, P4-5c       | `crates/vize_canon/src/virtual_ts*`, `crates/vize_canon/src/projection*`, `crates/vize_canon/src/source_map*`, `crates/vize_maestro/src/virtual_code*`, `crates/vize_croquis/src/virtual_ts*`                                                                                   |
| D    | P4-6a, P4-6b, P4-6c       | `crates/vize_davinci/src/diagnostic*`, `crates/vize_davinci/src/witness*`, `crates/vize_patina/src/rule_contracts*`, `crates/vize_patina/src/output*`, `davinci-road/plan/witness-exemptions.tsv`                                                                               |
| E    | P4-7a, P4-7b              | `crates/vize_patina/src/markup*`, `crates/vize_patina/src/linter/engine*`, `crates/vize_patina/src/ir.rs`                                                                                                                                                                       |
| F    | P4-8a, P4-8b, P4-8c       | `crates/vize_patina/src/rules/` minus lane B, minus lane G, minus lane I; `crates/vize_patina/tests/fixtures/parity/`                                                                                                                                                           |
| G    | P4-9a, P4-9b              | `crates/vize_s1_to_s2/src/pass/cfg*`, `crates/vize_croquis_cf/src/rules/complexity*`, `crates/vize_curator/src/complexity*`, `crates/vize_patina/src/rules/facts/max_template_complexity*`                                                                                      |
| H    | P4-10a, P4-10b            | `crates/vize_croquis_cf/src/providers*`, `crates/vize_maestro/src/ide/ecosystem*`, `crates/vize/src/commands/check/nuxt*`                                                                                                                                                       |
| I    | P4-11a, P4-11b            | `crates/vize_patina/src/html_content_model*`, `crates/vize_patina/src/rules/html/`, `crates/vize_patina/src/rules/vue/permitted_contents*`                                                                                                                                      |
| J    | P4-12a, P4-12b            | `crates/vize_glyph/`, `davinci-road/plan/style-spec.md`                                                                                                                                                                                                                         |
| K    | P4-12c                    | `crates/vize_s1/src/pug*`, `crates/vize_s1_to_s2/src/lower/pug*`                                                                                                                                                                                                                |
| L    | P4-13                     | `crates/vize_musea/src/parse*`                                                                                                                                                                                                                                                  |
| M    | P4-14a, P4-14b, P4-14c    | `crates/vize_davinci/src/render*`, `crates/vize_carton/src/i18n*`, `crates/vize/src/commands/explain*`, `tests/tooling/davinci-diagnostic-catalog*`                                                                                                                             |
| N    | P4-15a, P4-15b            | `tools/commands/davinci/seed-defects*`, `tools/commands/davinci/suppression-telemetry*`, `tests/_fixtures/davinci-fpfn/`                                                                                                                                                        |
| O    | P4-16                     | `crates/vize_vitrine/src/napi/plugin*`                                                                                                                                                                                                                                          |
| X    | P4-17                     | `davinci-road/plan/phase-4-records/`                                                                                                                                                                                                                                            |

**Projection consumers belong to lane C.** P4-3 waves switch every `Croquis` field reader **except** `crates/vize_canon/src/virtual_ts/**` and `crates/vize_maestro/src/virtual_code/**`; those readers die with the old generators in P4-5c, so switching them first would be work deleted a week later.

## Critical path

Ordered for the 2026-09-23 completion target. **Bold** tasks are the public-demo set (highest presentation value); the chain is the longest dependency path to the exit gate.

1. P4-1a fact API core → P4-6a diagnostic tiers → P4-6b witness verifier → P4-6c Patina on the unified channel
2. P4-7a S2 markup facade → P4-7b facade switch → **P4-8a** neutral-core wave → **P4-8b** dialect wave → P4-8c container wave
3. P4-15a seeded-defect matrix → P4-15b corpus FP triage → P4-17 exit (with P3-16)

Demo chains that run beside it, all startable now: **P4-11a → P4-11b** cross-component HTML conformance (`<p>` × child-root `<div>`); **P4-14a** rustc/Elm-grade renderer in en/ja/zh → P4-14b → P4-14c `--explain`; P4-3b → **P4-9a → P4-9b** complexity over real template CFGs; **P4-10a** typed Vue Router params; **P4-12c** pug as an S1 dialect; P4-5a → **P4-5b** (at P3-9) → P4-5c one projection.

## TODO index

Each ID links to its contract; the box is checked only in the PR that satisfies the task's acceptance criteria, with a record under `phase-4-records/`.

- [x] [P4-1a](./phase-4-tasks.md#p4-1a--fact-api-core) Fact API core — lane A · startable now
- [x] [P4-1b](./phase-4-tasks.md#p4-1b--fact-preservation-and-recompute-mode) Fact preservation and recompute mode — lane A · startable now
- [x] [P4-2](./phase-4-tasks.md#p4-2--fact-group-alpha-beta-split) Fact-group alpha/beta split — lane A · startable now
- [ ] [P4-3a](./phase-4-tasks.md#p4-3a--bindings-and-undefined-refs-fact-groups) Bindings and undefined-refs fact groups — lane B · startable now
- [x] [P4-3b](./phase-4-tasks.md#p4-3b--component-usage-fact-groups) Component-usage fact groups — lane B · startable now
- [ ] [P4-3c](./phase-4-tasks.md#p4-3c--unused-bindings-fact-group-and-lint-consumer) Unused-bindings fact group and lint consumer — lane B · startable now
- [ ] [P4-3d](./phase-4-tasks.md#p4-3d--reactivity-merges-into-the-s3-lattice) Reactivity merges into the S3 lattice — lane B · startable now
- [ ] [P4-3e](./phase-4-tasks.md#p4-3e--effect-graph-fact-group) Effect-graph fact group — lane B · gated on P3-6
- [ ] [P4-3f](./phase-4-tasks.md#p4-3f--provide-inject-and-race-fact-groups) Provide/inject and race fact groups — lane B · startable now
- [ ] [P4-4a](./phase-4-tasks.md#p4-4a--orphan-verdicts-for-non-effect-products) Orphan verdicts for non-effect products — lane B · startable now
- [ ] [P4-4b](./phase-4-tasks.md#p4-4b--effect-graph-verdict) Effect-graph verdict — lane B · gated on P3-6
- [x] [P4-5a](./phase-4-tasks.md#p4-5a--one-mapping-model-and-one-diagnostic-post-pass) One mapping model and one diagnostic post-pass — lane C · startable now
- [x] [P4-5b](./phase-4-tasks.md#p4-5b--s2-projection-as-an-s4-target) S2 projection as an S4 target — lane C · gated on P3-9
- [ ] [P4-5c](./phase-4-tasks.md#p4-5c--consumers-switch-and-three-generators-deleted) Consumers switch and three generators deleted — lane C · startable now (behind P4-5b)
- [x] [P4-6a](./phase-4-tasks-later.md#p4-6a--precision-tiers-and-the-error-witness-law-as-types) Precision tiers and the error-witness law as types — lane D · startable now
- [x] [P4-6b](./phase-4-tasks-later.md#p4-6b--witness-verifier) Witness verifier — lane D · startable now
- [x] [P4-6c](./phase-4-tasks-later.md#p4-6c--patina-on-the-unified-channel) Patina on the unified channel — lane D · startable now
- [x] [P4-7a](./phase-4-tasks-later.md#p4-7a--s2-backed-markup-facade) S2-backed markup facade — lane E · startable now
- [ ] [P4-7b](./phase-4-tasks-later.md#p4-7b--facade-switch-and-relief-projection-deleted) Facade switch and Relief projection deleted — lane E · startable now
- [ ] [P4-8a](./phase-4-tasks-later.md#p4-8a--neutral-core-rule-wave) Neutral-core rule wave — lane F · startable now
- [ ] [P4-8b](./phase-4-tasks-later.md#p4-8b--dialect-bound-rule-wave) Dialect-bound rule wave — lane F · startable now
- [ ] [P4-8c](./phase-4-tasks-later.md#p4-8c--container-bound-wave-and-legacy-visitor-retirement) Container-bound wave and legacy visitor retirement — lane F · startable now
- [x] [P4-9a](./phase-4-tasks-later.md#p4-9a--template-cfg-complexity-facts-and-metric-spec) Template CFG complexity facts and metric spec — lane G · startable now
- [x] [P4-9b](./phase-4-tasks-later.md#p4-9b--cross-file-complexity-rule-and-doctor-finding) Cross-file complexity rule and Doctor finding — lane G · startable now
- [x] [P4-10a](./phase-4-tasks-last.md#p4-10a--provider-contract-and-vue-router-provider) Provider contract and Vue Router provider — lane H · startable now
- [ ] [P4-10b](./phase-4-tasks-last.md#p4-10b--nuxt-provider-and-projected-route-types) Nuxt provider and projected route types — lane H · startable now (behind P4-5b)
- [x] [P4-11a](./phase-4-tasks-last.md#p4-11a--content-model-tables-and-exact-per-file-checker) Content-model tables and exact per-file checker — lane I · startable now
- [ ] [P4-11b](./phase-4-tasks-last.md#p4-11b--composed-cross-component-conformance) Composed cross-component conformance — lane I · startable now
- [x] [P4-12a](./phase-4-tasks-last.md#p4-12a--style-specification) Style specification — lane J · startable now
- [ ] [P4-12b](./phase-4-tasks-last.md#p4-12b--glyph-on-s1) Glyph on S1 — lane J · startable now
- [ ] [P4-12c](./phase-4-tasks-last.md#p4-12c--pug-as-an-s1-dialect) Pug as an S1 dialect — lane K · startable now
- [x] [P4-13](./phase-4-tasks-last.md#p4-13--musea-onto-s0-and-s1) Musea onto S0 and S1 — lane L · startable now
- [x] [P4-14a](./phase-4-tasks-last.md#p4-14a--structured-diagnostic-renderer) Structured diagnostic renderer — lane M · startable now
- [ ] [P4-14b](./phase-4-tasks-last.md#p4-14b--catalog-completeness-for-every-producer) Catalog completeness for every producer — lane M · startable now
- [ ] [P4-14c](./phase-4-tasks-last.md#p4-14c--explain-pages-and-witness-why) Explain pages and witness why — lane M · startable now
- [ ] [P4-15a](./phase-4-tasks-last.md#p4-15a--seeded-defect-matrix-at-full-scale) Seeded-defect matrix at full scale — lane N · startable now
- [ ] [P4-15b](./phase-4-tasks-last.md#p4-15b--corpus-suppression-triage-to-zero-untriaged) Corpus suppression triage to zero untriaged — lane N · startable now
- [x] [P4-16](./phase-4-tasks-last.md#p4-16--js-plugin-sdk-spike) JS plugin SDK spike — lane O · startable now
- [ ] [P4-17](./phase-4-tasks-last.md#p4-17--phase-exit) Phase exit — lane X · gated on P3-16

"Startable now (behind P4-5b)" means the task has no phase-3 dependency of its own but depends on a gated task.

---

## Exit gate (machine-checkable)

The four provisional gate lines are normative and kept verbatim; the re-cut adds lines, it never rewords one.

- [ ] TS-40 check parity; TS-39 lint agreement; TS-5 + TS-41 glyph gates
- [ ] Consumption matrix: every computed group ≥1 consumer or gated (TS-12)
- [ ] TS-36 witnesses verify; TS-37 100% recall per class; TS-38 zero untriaged candidates
- [ ] canon/maestro projection duplicates + glyph byte scanner + musea hand parser: deleted
- [ ] TS-35 zero undeclared fact accesses, demand graph acyclic by strata; TS-34 spec/impl agreement for every migrated fact group
- [ ] Error-severity witness exemption inventory empty (P4-6); TS-53 renderer snapshots exact in en/ja/zh and every diagnostic code catalogued in all three locales
- [ ] Croquis `virtual_ts` (the third generator) deleted with the other two; pug compiles, lints and formats through S1 (charter #12)
- [ ] Standing gates held: TS-1..9, TS-11 empty for every surface the phase touches, TS-12 matrices current, TS-13 clean, TS-10 ratchets tightened only
