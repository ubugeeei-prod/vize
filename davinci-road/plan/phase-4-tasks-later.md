# Phase 4 — Task contracts, P4-6a through P4-9b

> [!NOTE]
> Continuation of [phase-4-tasks.md](./phase-4-tasks.md) under the 350-line source budget. Same authority, same format; the TODO index in [phase-4.md](./phase-4.md) links each task to whichever file holds its contract.

## P4-6a — Precision tiers and the error-witness law as types

**Start gate:** startable now — P3-independent.

**Lane:** D

**Deliverable:** the assurance doctrine's verdict rules as types in `vize_davinci::diagnostic`: `Tier { Exact, Sound, Complete, Heuristic }` with a declared domain, a `RuleContract` const constructor that rejects `Heuristic` with error severity at compile time, and `Severity::Error` diagnostics constructible only through a proven constructor that takes a non-empty `Witness` (a chain of `WitnessLink { group: AnalysisId, span, key }` naming fact groups). `Witness::LegacyExempt` stays, counted by an inventory that only shrinks.

**Steps:**

- [x] `crates/vize_davinci/src/diagnostic/tier.rs` and `diagnostic/witness.rs`; the existing `Diagnostic::new` stops accepting `Severity::Error`
- [x] Two `compile_fail` doctests: a heuristic rule declaring error severity, and an error diagnostic built without a witness (the provisional "canary rule that tries error-on-unknown fails to compile")
- [x] `davinci-road/plan/witness-exemptions.tsv` (producer, code, exempt count) with `tests/tooling/davinci-witness-exemptions.test.ts`: counts may only fall against the base revision, proven by an injected increase

**Acceptance:** `cargo test -p vize_davinci` and `cargo test -p vize_davinci --doc` green with both `compile_fail` canaries; TS-24 builds; the exemption checker green and demonstrably failing on an injected increase; TS-1, TS-13.

**Deps:** P4-1a.

**Non-goals:** verifying witnesses (P4-6b); Patina adoption (P4-6c); rendering (P4-14a).

**Landed 2026-09-22:** the witness law as types, both `compile_fail` canaries with passing twins, and a source-derived exemption inventory (5 `vize_s1_to_s2` rows) that only shrinks — see the [P4-6a record](./phase-4-records/p4-6a.md).

## P4-6b — Witness verifier

**Start gate:** startable now — P3-independent.

**Lane:** D

**Deliverable:** TS-36: `vize_davinci::witness::verify(&Diagnostic, &FactManager) -> Result<(), WitnessError>` re-checks every link against the fact base, run over every error diagnostic in debug and CI builds through an observer (release ZST).

**Steps:**

- [ ] `crates/vize_davinci/src/witness.rs` + `witness/`; forged-witness fixtures (wrong group, wrong span, missing key) each committed with its exact `WitnessError`
- [ ] Register the TS-36 command in [test-suites.md](./test-suites.md): `cargo test -p vize_davinci --test witness_verify` plus the TS-9 lint fixtures run in debug

**Acceptance:** every forged fixture rejected with the exact error, every valid witness verifies; zero unverifiable witnesses across TS-9; TS-1, TS-13.

**Deps:** P4-6a, P4-1a.

**Non-goals:** producing witnesses for rules (P4-6c, P4-8a…P4-8c); the "why" rendering (P4-14c).

## P4-6c — Patina on the unified channel

**Start gate:** startable now — P3-independent.

**Lane:** D

**Deliverable:** every one of the 248 rules has a `RuleContract` (tier + domain) in one table, `crates/vize_patina/src/rule_contracts.rs` — a table rather than per-file edits, so lane F keeps sole ownership of `rules/` — and Patina diagnostics convert to `vize_davinci::Diagnostic` with file-absolute spans framed by `vize_s0::{SourceRoot, SourceBlock}`, fixing FP-1's root cause (`type/require-typed-emits` / `type/require-typed-props` report script-block offsets against whole-file lines).

**Steps:**

- [ ] Table keys == registered rule names, asserted by a test; `tools/davinci/rule-parity.mjs` gains a `tier` column and fails on a rule without one
- [ ] Error-default rules (99 today) without witnesses enter `witness-exemptions.tsv`; that list is the P4-8 waves' drain queue
- [ ] Offsets canonicalized in `crates/vize_patina/src/output/shared.rs` through the S0 frame

**Acceptance:** [ledger-fp.md](./ledger-fp.md) FP-1 flipped to `fixed` with the layoutit-grid pairs re-measured at 0 of 17; `rust-script tools/commands/davinci/rule-parity.rs --check` green with the tier column (TS-12); TS-9 snapshots unchanged except the FP-1 spans, each changed line listed in the PR; exemption inventory ≤ 99 rule entries; TS-13.

**Deps:** P4-6a.

**Non-goals:** migrating rule bodies (P4-8a…P4-8c); compiler and Canon producers (P4-14b).

## P4-7a — S2-backed markup facade

**Start gate:** startable now — P3-independent.

**Lane:** E

**Deliverable:** `MarkupDocument` (`crates/vize_patina/src/markup.rs`, 1,636 lines) gains a zero-copy S2 inner variant: SFC templates through the S1→S2 lowering, JSX through P2-16's S2 projection; `ui.if` regions answer `MarkupConditional`, `ui.for` answers `MarkupList`, `ui.bind`/`ui.on`/`ui.model`/`vue.directive` answer `MarkupBinding`. A TS-25 lane proves the facade observes the same document either way.

**Steps:**

- [ ] `crates/vize_patina/src/markup/s2.rs`; Patina gains `vize_s1`, `vize_s2`, `vize_s1_to_s2` dependencies (publishable per the release firewall)
- [ ] `crates/vize_patina/src/markup/differential.rs` behind `davinci-differential`: the full `MarkupRule` hook trace (hook, spans, names, values, modifiers) compared exactly between the Relief/OXC and S2 projections
- [ ] Corpus-runnable entry and a plain-suite witness pinning the comparison count

**Acceptance:** `cargo test -p vize_patina --features davinci-differential --test davinci_markup_differential` zero divergence over the rule fixtures and a corpus shard with scope proof; `cargo bench -p vize_patina --bench davinci_markup` S2 variant `allocs` recorded in `budgets.toml` (TS-10); TS-9 unchanged (no lane switched); TS-13.

**Deps:** none (phase-2 exit).

**Non-goals:** switching the lint lanes (P4-7b); porting rules (P4-8a…P4-8c).

## P4-7b — Facade switch and Relief projection deleted

**Start gate:** startable now — P3-independent.

**Lane:** E

**Deliverable:** `lint_sfc` and `lint_jsx` drive markup rules over the S2 facade only; `MarkupDocumentInner::{Relief, Jsx}` and `MarkupDocument::from_jsx` are deleted; `ir.rs`'s `LintDocumentKind` maps to S1 input dialects.

**Steps:**

- [ ] Switch both lanes in `crates/vize_patina/src/linter/engine*`, then delete the old variants
- [ ] Re-record `patina_jsx_markup_one_root` and the markup bench `allocs` (tightened or held)

**Acceptance:** TS-9 lint snapshots unchanged; TS-39 SFC/JSX agreement for the 40 markup-facade rules; `grep -rn "MarkupDocumentInner::Relief\|fn from_jsx" crates/vize_patina/src` empty; TS-10 ratchet held; TS-11 lint surface empty with scope proof.

**Deps:** P4-7a.

**Non-goals:** the 208 rules not yet on the facade (P4-8a…P4-8c).

## P4-8a — Neutral-core rule wave

**Start gate:** startable now — P3-independent.

**Lane:** F

**Deliverable:** the 92 neutral-core candidates of the [rule-parity matrix](./rule-parity.md) — 43 template (28 already on the facade, 15 to port), 42 script, 7 CSS — run over the neutral core on SFC **and** JSX, read facts through declared demands instead of direct Croquis imports, and carry witnesses where they report errors. **Explicitly marked small series:** one installment per rule-family directory (`a11y/`, `vue/`, `opinionated/`, `script/`, `css/`, `ecosystem/`), so several agents run the wave at once; `rules/html/` and `rules/vue/permitted_contents*` belong to lane I.

**Steps:**

- [ ] Template rules become `MarkupRule`s over the S2 facade; script rules also run on `.jsx`/`.tsx` programs through the script registry
- [ ] Paired fixtures `crates/vize_patina/tests/fixtures/parity/<rule>/{sfc.vue,jsx.tsx}` with exact diagnostic snapshots
- [ ] Each installment regenerates the rule-parity matrix and drains its rules from `witness-exemptions.tsv`

**Acceptance:** per installment — TS-39: each rule's SFC and JSX fixtures produce the same diagnostics; TS-9 SFC snapshots unchanged; TS-35 zero undeclared; the regenerated matrix shows SFC∩JSX up by the installment's rule count and direct Croquis imports down (both ratchets); the exemption inventory shrinks.

**Deps:** P4-7b, P4-6c, P4-1a.

**Non-goals:** dialect-bound rules (P4-8b); new rules.

## P4-8b — Dialect-bound rule wave

**Start gate:** startable now — P3-independent.

**Lane:** F

**Deliverable:** the 133 Vue-dialect-bound rules — 101 template (11 on the facade, 90 to port), 29 script, 3 CSS — over `ui.*` and `vue.*` ops; a rule runs on JSX wherever the semantics exist there (JSX `v-model` lowers to the same `ui.model` contract), with a per-rule opt-out only where semantics genuinely diverge. Same small-series shape as P4-8a.

**Steps:**

- [ ] Opt-outs recorded in `rule-parity-overrides.toml` with a reason; the generator rejects a reason-less override
- [ ] Paired fixtures and exemption draining as in P4-8a

**Acceptance:** as P4-8a, plus TS-12 rejecting a reason-less override (proven by injection).

**Deps:** P4-8a.

**Non-goals:** container-bound rules (P4-8c).

## P4-8c — Container-bound wave and legacy visitor retirement

**Start gate:** startable now — P3-independent.

**Lane:** F

**Deliverable:** the 23 container-bound rules (17 template, 6 Musea) migrated, then the legacy `Rule` template-visitor hooks (`run_on_template`, `enter_element`, `exit_element`, `check_directive`, `check_for`, `check_if`, `check_interpolation`) and the JSX `fallback` lane (`legacy_keep_mask`) deleted — charter #7's litmus closed and fact adoption raised from 23 rules to the neutral-core majority (charter #35).

**Steps:**

- [ ] Migrate, then delete the hooks and the fallback lane
- [ ] Final TS-39 run across all waves

**Acceptance:** the matrix shows JSX `fallback` 0, `no-jsx-hooks` only on container-bound rules, every neutral-core rule in SFC∩JSX, direct Croquis imports 0; the exemption inventory has no rule entries; TS-9; TS-11 lint surface empty with scope proof.

**Deps:** P4-8b, P4-11a, P4-6b.

**Non-goals:** Svelte/Solid dialects (charter #1); the JS plugin tier (P4-16).

## P4-9a — Template CFG complexity facts and metric spec

**Start gate:** startable now — P3-independent.

**Lane:** G

**Deliverable:** the metric definition recommended in [open questions](../open-questions.md#complexity-metric-definition), written as `davinci-road/plan/complexity-metrics.md`, and the `ComplexityFacts` group computed by a fusable analysis pass over S2 regions, replacing today's string counting in `vize_croquis_cf/src/rules/complexity.rs`.

**Steps:**

- [ ] `crates/vize_s1_to_s2/src/pass/cfg.rs`: `Optional`/`Fusable`, `Preserved::ALL`; per component **own cyclomatic** = 1 + decisions (each `ui.if` branch beyond the first, plus one for an `ui.if` without `v-else`; each `ui.for`; each `&&`, `||`, `??` and `?:` in a retained expression AST; an `Opaque` expression adds 0 and is counted as unknown) and **own cognitive** (+1 per structure, + nesting depth for nested `ui.if`/`ui.for`/scoped-slot regions, +1 per run of like logical operators)
- [ ] TS-34 naive evaluator over the same definition
- [ ] Record the corpus distribution (p50/p90/p95/p99 of both metrics) with its command; pin the default thresholds at the recorded p95 (warn) in the spec

**Acceptance:** TS-34 agreement over the matrix plane and a corpus shard; TS-17 pass snapshot; TS-22 walk counts unchanged (the pass fuses); pass bench `allocs` recorded (TS-10); the distribution table reproduces from its recorded command.

**Deps:** P4-1a.

**Non-goals:** cross-file attribution and the rule (P4-9b); script-side complexity.

## P4-9b — Cross-file complexity rule and Doctor finding

**Start gate:** startable now — P3-independent.

**Lane:** G

**Deliverable:** **rendered complexity** = own + Σ own(child) over the distinct child components reachable in the render tree (P4-3b identities, strongly connected components collapsed so recursion counts once); the Patina rule `vue/max-template-complexity` (tier `exact`, warning, threshold from P4-9a) on **own**; a Doctor hotspot finding on **rendered**; `ComplexityInput`'s template-control-flow dimension fed from facts, with the string-scanning `logical_operator_count` deleted.

**Steps:**

- [ ] Rule in lane G's own file `crates/vize_patina/src/rules/facts/max_template_complexity.rs`, registered by one line; Doctor and `crates/vize_curator/src/complexity.rs` render facts
- [ ] Fixtures: a recursive component, a child shared by two parents, an aliased import

**Acceptance:** TS-9 rule fixtures exact; Doctor snapshot churn documented in the PR (analysis surface, charter #23); `grep -rn "logical_operator_count" crates` empty; TS-35; TS-12.

**Deps:** P4-9a, P4-3b, P4-6a.

**Non-goals:** error severity for complexity; project-wide bands beyond the existing report.
