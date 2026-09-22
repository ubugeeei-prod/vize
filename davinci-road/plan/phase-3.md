# Phase 3 — Impeto and Backend Convergence (provisional decomposition)

> [!WARNING]
> Provisional; re-cut at phase-2 exit. Suites referenced as TS-n from
> [test-suites.md](./test-suites.md).

## TODO index

- [x] P3-1 `vize_impeto` crate + phase validator
- [x] P3-2 Reactivity lattice fact group v1
- [x] P3-3 S2→S3 lowering + shared partition
- [x] P3-4 Lean reference semantics + differential runner
- [x] P3-5 Impeto op reference doc (before optional passes)
- [ ] P3-6 Vapor backend on S3
- [x] P3-7 VDOM patch flags from lattice facts _(owner-keyed table and hydrated DOM corpus gate; see [record](./phase-3-records/p3-7.md))_
- [ ] P3-8 SSR thin path
- [ ] P3-9 S4 structured emitter + universal source maps _(slice 1 pins TS-31 source-map budgets before emitter migration; see [record](./phase-3-records/p3-9.md))_
- [ ] P3-10 Try-measure-commit extraction _(slice 1 pins optimization budgets before extraction; see [record](./phase-3-records/p3-10.md))_
- [x] P3-11 IVM oracle
- [x] P3-12 Behavioral (sprout) runner incl. IME scripts
- [x] P3-13 Optimization remarks + corpus remarks-diff
- [x] P3-14 `folio-reduce` _(`vize reduce`; see [record](./phase-3-records/p3-14.md))_
- [x] P3-15 Lean theorems (lattice / grouping / IVM linearity)
- [ ] P3-16 Phase exit
- [ ] P3-17 Production SFC reach _(slice 1 measures `compile_sfc` reach per shipping shape and pins `[reach]` floors; see [record](./phase-3-records/p3-17.md))_

---

**P3-1 `vize_impeto`.** Flat id-based ops (generalizing
`vize_atelier_vapor/src/ir.rs`'s 16 variants), **explicit state edges** for
DOM/effect ordering, named phases `built → partitioned → scheduled` with a
between-pass validator (edges resolve, regions nest, effects well-scoped —
TS-27), `no_std + alloc`, folio from birth (TS-16). _Accept:_ TS-16/24/27;
size asserts. **Landed 2026-09-12:** see
[P3-1 record](./phase-3-records/p3-1.md).

**P3-2 Reactivity lattice v1.** Fact group classifying bindings/expressions
(static → props-stable → reactive → unstable) using the React-Compiler effect
vocabulary (`Freeze`/`Capture`/`MutateGlobal`…) as per-binding summaries over
retained oxc ASTs; escape analysis drives demotion; `provide/inject`-derived
bindings cap at `reactive` (Effekt lexical/dynamic rule). **Lattice states
and verdicts are orthogonal axes:** the four lattice states are the fact's
_value_; `proven/refuted/unknown` is the epistemic _status_ of that value —
a binding can be provenly `reactive` or unknowably classified, and rules fire
only on proven values per the assurance doctrine. _Accept:_ declarative rule spec + naive evaluator committed
(TS-34 pattern); lattice folio page. _Landed 2026-09-12:_ see
[P3-2 record](./phase-3-records/p3-2.md).

**P3-3 S2→S3 lowering.** Total, no-rollback; static/dynamic partition
computed once here and exported as facts (SSR reads them without S3).
**Exported partition facts describe canonical S3 only**: optional passes
(P3-10 extraction) either provably preserve the partition (verifier-checked)
or trigger fact revalidation before anything downstream reads them — stale
exports are a verifier failure, not a footgun. ANF-ish
discipline: pure expressions vs effectful ops separated. _Accept:_ TS-17
pass snapshots; TS-20 totality fuzz extended to S2→S3.
_First slice 2026-09-12:_ see
[P3-3 record](./phase-3-records/p3-3.md) for the `vize_s2_to_s3` crate,
total lowering skeleton, and exported partition fact contract. _Closed
2026-09-12:_ TS-17 S3 Folio snapshots and TS-20 S2→S3 fuzz coverage were added
to the same record.

**P3-4 Lean reference + differential.** `formal/impeto/` Lean package
(CI-lenient lane per charter #39): executable small-step semantics for S3 ops
under both Vapor and VDOM interpretations; runner compares compiled-output
behavior traces vs reference on S3 fixtures (TS-28). _Accept:_ runner in CI
on the fixture ladder. The 2026-09-15 compiled-runtime slice evaluates emitted
DOM/Vapor render functions under a deterministic JS host trace runner in TS-28;
P3-4 still exits only once the mounted behavior runner uses the same reference
contract.
_First slice 2026-09-12:_ see
[P3-4 record](./phase-3-records/p3-4.md) for the pinned Lean package, the
initial S3 Folio parser, executable reference traces, and CI-lenient TS-28
workflow. _Third slice 2026-09-15:_ backend-specific `.vdom.trace` and
`.vapor.trace` artifacts are now checked by the Lean runner and mirrored by the
Rust S2→S3 fixture bridge. _Fifth slice 2026-09-15:_ compiled DOM/Vapor render
functions are now executed by the TS-28 cargo gate through the shared JS runtime
trace runner. _Stateful slice 2026-09-16:_ the dynamic-button fixture now feeds
Rust-owned S3 values into independent Lean semantics and compares the same full
JSON observations against both mounted Vue runtimes, including patches, clicks
and unmount. _Control/slot slice 2026-09-18:_ both Rust-lowered template fixtures
now share full stateful observations, including boolean condition changes and
absent-slot fallback `v-text` updates. _Model slice 2026-09-22:_ native
`v-model` (IME, `.lazy`/`.trim`/`.number`, checkbox/radio/select) shares
twelve full reference observations with both mounted runtimes. _Closed
2026-09-22:_ supplied slots (static text or displayed slot props, including in
branches and keyed loops) now share eight full observations too. The mounted
runner and the Lean reference use one contract across the fixture ladder, the
TS-29 matrix and the model/slot references (see the record).

**P3-5 Op reference doc.** `davinci-road/plan/impeto-ops.md`: every op's
meaning under both interpretations, written **before any optional pass
lands** (MIR anti-lesson); Lean file is the normative companion; Folio is the
concrete syntax. _Accept:_ review point — signed off; doc cross-linked from
rustdoc. _Landed 2026-09-12:_ see
[P3-5 record](./phase-3-records/p3-5.md).

**P3-6 Vapor on S3.** `vize_atelier_vapor` lowers S2→S3→generate with full
semantic context; deletes the run-then-discard double transform
(`compile.rs`) and the duplicated directive transforms
(`transforms/{v_if,v_for,v_on,v_bind,v_model,v_show,transform_slot,transform_text}.rs`);
calls upstream `@vue/runtime-vapor` APIs only (charter #38). In-phase flag
for fallback. _Accept:_ TS-33 behavioral parity; TS-30 traces; vapor bench
improvement (the P0-3 double-transform number is the floor to beat).
_First slice 2026-09-12:_ see
[P3-6 record](./phase-3-records/p3-6.md) for the production-path S3 bridge,
verified artifact guard, and profiler counters. _Native generation slice
2026-09-20:_ a private checked S3 payload now reaches the shared emitter for
ordinary native HTML with direct-reference prop/text/click bindings, bypassing
legacy transform/lowering. Unsupported surfaces explicitly retain the legacy
lane. Graph-payload mutations, zero legacy-walk probes, and mounted identity/event
traces enforce that boundary; full parity and benchmark promotion remain open.
_Text/event expansion 2026-09-21:_ validated compound-text parts now survive into
S3 generation with coalesced DOM addresses and stable following siblings. Static
events share modifier/delegation semantics with legacy lowering; mounted traces
cover key/DOM guard composition and combined listener options. The remaining
P3-6 semantic, corpus and benchmark gates are unchanged.
_Control-flow slice 2026-09-21:_ S3 `If`/`For` project natively: branch chains
and element-carried keyed/unkeyed loops with identifier aliases, nested inside
admitted elements or at the root. Template wrappers, destructuring and nested
bodies count as `legacy.control_flow`. Mounted identity traces fixed a shared
fast-removal defect in both lanes; fixture parity holds at 116/117.
_Expression slice 2026-09-22:_ compound props, text, handlers, conditions,
sources and keys consume S2's retained ASTs (moved, not reparsed); `v-show`,
`v-html`, `v-text`, static-class merging and unprefixed binding metadata (the
SFC path) are native, with zero legacy walks and reparses.
_Component slice 2026-09-22:_ ordinary components with props, listeners and
default slots, `<slot>` outlets with fallbacks, and root fragments/text are
native; mounted traces register child components per backend and lane. Named
and scoped slots, dynamic and built-in components stay `legacy.component`.
_Single parse 2026-09-22:_ admitted sources skip the legacy parser; markup it
diagnoses (CDATA, self-closed or implicitly closed elements, empty modifiers)
stays legacy, checked over fixture prefixes and deletions. With dense admission
tables the native route is 13% faster than the retained lane on expressions
and 3% on components, 7–9% slower on text, events and control flow.

**P3-7 VDOM patch flags from facts.** `patch_flag.rs` inference replaced by
lattice-fact consumption; flags become explicit S3 decisions (or S2→S4
annotations if S3 detour measures badly — decide by TS-22). _Accept:_ corpus
DOM byte-parity (TS-11 empty); patch-flag equivalence fixtures.
_First slice 2026-09-13:_ see
[P3-7 record](./phase-3-records/p3-7.md) for the S2→S4 annotation boundary and
the initial `PatchFacts` split.
_Second slice 2026-09-13:_ `PatchFactsTable` is now owner-keyed by
`ui.element` / `ui.component` `NodeId` and read by the VNode writers before
printing patch flags or dynamic-props arguments. Subsequent slices integrate
binding metadata through the reactivity lattice and gate the DOM corpus.
_Sixth slice 2026-09-16:_ the S2 DOM corpus runner now emits
`patch_fact_entries` evidence and the Real Project Matrix artifact validator
rejects a clean byte-parity run that never materialized owner-keyed patch facts.
_Closed 2026-09-16:_ [Real Project Matrix run 35055720056](https://github.com/ubugeeei-prod/vize/actions/runs/35055720056)
is terminal green on `0e6985a4cb846e393daa1fad705b069e09bf85fa` with 42,279
comparisons, 390,264 patch-fact entries, zero S2 refusals, and zero divergences.
The [record](./phase-3-records/p3-7.md) preserves the corpus scope and exclusions.

**P3-8 SSR thin path.** S2→S4 string-plan lowering reading partition facts;
`vize_atelier_ssr` codegen re-targets. _Accept:_ SSR corpus byte-parity
(TS-11 empty for ssr).
_First slice 2026-09-13:_ see
[P3-8 record](./phase-3-records/p3-8.md) for the production-path S4 bridge,
partition-fact string-plan witness, and byte-parity guard.
_Third slice 2026-09-21:_ the production selector emits plain-element SSR
(elements, static and bound attributes, text, interpolation, fallthrough
roots) from the S4 plan, holds byte parity on the snapshot suite and a
two-emitter differential battery, and counts `accepted` / `legacy.<reason>`.
_Fourth slice 2026-09-21:_ `v-if` chains, `v-for` loops, `v-show`, `v-html`,
`v-text`, and native `v-model` also emit from the plan (region boundary
segments, legacy fragment and fallthrough rules, `v-for` scope strips).
_Fifth slice 2026-09-21:_ the feature-gated SSR corpus gate compares both
emitters on every SFC template (smoke sweep of the checkout in CI: 717
compared, 274 plan-emitted, 0 divergences).
_Sixth slice 2026-09-22:_ components without slot content, `<Teleport>`,
`<Suspense>`, transparent built-ins, and `<slot>` outlets emit from the plan
(checkout smoke: 494 of 717 plan-emitted, 0 divergences).
_Seventh slice 2026-09-22:_ component slot content emits the static slots
object with push-form slot functions and their VNode fallback (checkout
smoke: 523 of 717 plan-emitted; vendor sample: 1,034 of 1,279; 0
divergences).
_Eighth slice 2026-09-22:_ `<component :is>`, component `v-model`
arguments and modifiers, and whitespace/comment gaps between `v-if` branches
emit from the plan (checkout smoke: 643 of 717; vendor sample: 1,207 of
1,279; 0 divergences).
_Ninth slice 2026-09-22:_ `createSlots` shapes (conditional, looped, and
dynamically named slot templates) emit from the plan, and every traversal
ladder fixture now skips the legacy SSR codegen walk (checkout smoke: 650 of
717; vendor sample: 1,242 of 1,279; 0 divergences).
_Tenth slice 2026-09-22:_ plain `<template>` and the non-raw-text legacy
content tags (`iframe`, `noscript`, ...) emit from the plan (checkout smoke:
663 of 717, 0 divergences). Element directives stay refused pending a
decision: the legacy lane renders them through an unbound `_directives`
(see the record).
_Production sweep 2026-09-22:_ the corpus gate also drives the SFC adapter
entry point on both lanes; production reach is 112 of 722 checkout and 171 of
1,279 vendor templates (0 divergences), bounded by `legacy.croquis` on every
`<script setup>` SFC until Croquis-informed rewrites land in the shared
transform door.
_JSX SSR 2026-09-22:_ JSX/TSX SSR enters the same plan lane from its S2
projection (`compile_s2_to_ssr`), byte-identical to the walker on every JSX
SSR snapshot and a 28-case differential; JSX custom directives and
`<component>` still use the walker. The Vue 3.5 alignment of the legacy
directive output (`fix(ssr)!`), Croquis-informed rewrites, a canonical Real
Project Matrix run, and the legacy walker deletion remain before P3-8 closes.

**P3-9 S4 emitter + source maps.** Structured span-carrying emission document
replaces `CodegenContext.code` string appends across dom/vapor/ssr; one
`SourceMapBuilder`; **SSR and Vapor emit source maps**; delete
`crates/vize_atelier_sfc/src/source_map.rs` text-matching recovery.
_Accept:_ TS-31 coverage budget — **the numeric threshold is pinned in
`budgets.toml` before this task merges**, and the text-matching recovery may
only be deleted once the new path's measured coverage ≥ the old heuristic's
measured coverage; TS-11 empty (maps are additive artifacts).
_First slice 2026-09-13:_ see
[P3-9 record](./phase-3-records/p3-9.md) for the TS-31 budget pin.
_Second slice 2026-09-13:_ the same record now covers SSR/Vapor opt-in map
entry points and binding bridge propagation. _Third slice 2026-09-21:_ TS-31
is measured by an independent-decoder harness over a committed fixture battery
([report](./ts31-sourcemap-coverage.json)); rows below budget are listed in the
record's tracked-shortfall ledger. _Fourth slice 2026-09-21:_ the DOM codegen
emits through span-carrying context methods and meets every TS-31 DOM budget.
_Fifth slice 2026-09-21:_ SSR carries spans through template-literal parts
and direct writes via the shared `SpannedText` primitive and meets every TS-31
SSR budget. _Sixth slice 2026-09-22:_ DOM and SSR write into one
target-neutral `EmitDocument` whose generated↔authored range links serialize
as v3 segments and convert to P4-5a `ProjectionMapping` rows; maps and bytes
unchanged. _Seventh slice 2026-09-22:_ both Vapor lanes (legacy lowering and
native S3) emit token-level maps through the same document and meet every
TS-31 Vapor budget, so no backend row remains below budget. The structured
SFC path and legacy recovery
removal remain open.

**P3-10 Try-measure-commit.** Placement alternatives (hoist/cache/inline/
group) kept explicit on S3 nodes; extraction pass performs candidates,
locally simplifies with fact approximations in scope, measures (emitted
size, reactive-edge count, update-path length), commits under an explicit
multi-metric rule — **no metric may regress beyond a per-metric ε and at
least one must improve** (correctness-adjacent metrics like reactive-edge
count are constraints, size is the objective; the ε values and ordering are
pinned in `budgets.toml` at task start from corpus distributions, ties reject
in favor of the simpler shape) — under a decrementing per-component budget;
`-O` tiers = budget constants in `budgets.toml`. _Accept:_ TS-17 snapshots of decisions; TS-32 remarks record
applied/missed; no output regression (TS-11/TS-33).
_First slice 2026-09-13:_ see
[P3-10 record](./phase-3-records/p3-10.md) for the task-start budget pin,
zero-epsilon metric policy, and `-O0` through `-O3` candidate budgets.
_Placement slice 2026-09-21:_ hoist/cache/group alternatives are an explicit,
`S3V010`-verified overlay on S3 ops with a companion Folio page; the graph and
exported partition stay canonical. _Extraction slice 2026-09-21:_ the
try-measure-commit pass commits under the pinned rule and per-component budget,
with `-O` tiers synced to `budgets.toml`, TS-17 decision snapshots, and one
applied/missed `s3.extract-placements` remark per candidate, gated corpus-wide
by a TS-32 baseline. Backend consumption of the committed placements
(TS-11/TS-33) remains open.

**P3-11 IVM oracle.** Incremental-update ≡ from-scratch render on the Lean
reference for keyed/unkeyed `v-for`, conditional toggles, mixed non-linear
expressions (TS-29). _Accept:_ suite green over matrix fixtures.

_Array-loop slice 2026-09-20:_ the [P3-11 record](./phase-3-records/p3-11.md)
adds independent scoped array execution and retained-identity reconciliation,
with full mounted VDOM/Vapor observations. _Closed 2026-09-21:_ a generated
32-case matrix covers keyed and positional arrays, objects and ranges; per-item
and guard toggles; and non-linear expressions evaluated by an independent Lean
JavaScript subset. It runs through the proved update machine and both mounted
runtimes. The only divergence is an exactly pinned upstream `runtime-vapor`
unkeyed-object key-alias defect.

**P3-12 Behavioral runner.** Sprout-style: mount compiled VDOM + Vapor
against scripted prop/interaction traces in a headless DOM; **IME composition
scripts pin `ui.model` realizations** (charter #40): compositionstart →
intermediate input → compositionend, `.lazy`/`.number`/`.trim`, checkbox
arrays, select-multiple (TS-30). _Accept:_ trace equality across backends and
vs reference.

_Progress 2026-09-16:_ [P3-12 record](./phase-3-records/p3-12.md)
covers the mounted Vue runtime gate, reactive button and slot/branch scripts,
IME composition, model modifiers, checkbox arrays, select-multiple, and member
bindings. It records the `v-text`, source-order, and model-expression defects
exposed by those scripts. _Closed 2026-09-22:_ every acceptance surface
(IME scripts, modifiers, checkbox arrays, select-multiple, keyed interactions
including live model state across reorder, slots) now has trace equality across
both backends and against the Lean reference; see the record's gate table.

**P3-13 Remarks.** `{pass, kind: applied|missed, span, args}` structured
remarks through the observer; corpus remarks-diff job (TS-32); missed-remarks
feed C-13. _Accept:_ remarks render in Spolvero (C-5); diff job wired.
_Substrate slice 2026-09-21:_ see [P3-13 record](./phase-3-records/p3-13.md)
and [remarks-format.md](./remarks-format.md): the observer's remark hook with
a compile-time zero-cost gate (0 allocations detached, measured), the
`[remarks]` page and schema-versioned JSON, `davinci-opt --remarks`, and
`hoist-static` as the first emitter. _TS-32 slice 2026-09-21:_ the corpus
remarks-diff over the 433 in-repo fixtures is wired into `clippy-and-test`
against a committed baseline whose bless refuses unexplained
`applied → missed` transitions. _Feed slice 2026-09-21:_ the Spolvero feed
(inspector, `analyzeSfc`, `davinci-opt`) carries the remarks, and the C-13
backlog is mined from the TS-32 corpus. _Rendering slice 2026-09-22:_ the
playground's Davinci tab renders them (C-5). **Landed** 2026-09-22: both
acceptance items hold; emitters for decisions made outside the pass manager
are follow-ups in the record.

**P3-14 `folio-reduce`.** Interestingness-script driver (llvm-reduce model)
with S1-subtree deletion vocabulary; oracles composable from diagnostics /
remarks / folio content / budget breaches. _Accept:_ reduces a seeded crash
fixture to ≤ 20% size while preserving the oracle. **Landed 2026-09-22:**
`vize reduce` (see [P3-14 record](./phase-3-records/p3-14.md)) reduces the
content-seeded crash repro from 3733 to 117 bytes (3.1%, 1-minimal), and the
reduced repro still replays through `vize repro`.

**P3-15 Lean theorems.** Lattice laws (classification monotonicity, join),
effect-grouping preserves dependency edges, keyed-`v-for` IVM linearity —
proved against the P3-4 semantics as they stabilize. _Accept:_ theorems in
CI-lenient lane; failures block S3-semantics changes, not unrelated PRs.

_Lattice slice 2026-09-21:_ the [P3-15 record](./phase-3-records/p3-15.md)
proves the lattice order, least-upper-bound join, declarative-spec minimality
and classification monotonicity in Lean, audits every theorem's foundations
during `lake build`, and checks the Rust evaluator's 400-row fact page against
the proved classifier. _Grouping slice 2026-09-21:_ acceptance by the TS-27
scheduled-phase edge contract is proved to order every state edge in the P3-4
VDOM and Vapor traces, and to keep scoped edges in scope. Any accepted
regrouping therefore preserves the edge set; 45 Rust validator verdicts match
the Lean contract. _Closed 2026-09-21:_ the reference update machine is proved
to equal recompute-from-scratch, retain keyed identities and allocate exactly
the inserted delta, completing the three theorem families in the CI-lenient
lane (see the record).

**P3-17 Production SFC reach.** Every Davinci backend stage must be reached
by the compiles users actually run: `compile_sfc` with a Croquis summary,
inline render closures, binding metadata, scoped styles and module-mode
hoisting, not only the bare template entry points the stage corpora compare.
_Accept:_ the production-reach gate reports per-shape reach from the backends'
selection counters with `reach-budgets.toml [reach]` floors only rising; the
production-path parity oracle (forced legacy DOM lane, whole-module byte
equality) is empty; the Croquis refusal is lifted from the DOM selector on
that oracle. _First slice 2026-09-22:_ see
[P3-17 record](./phase-3-records/p3-17.md): measured reach is 0/342 DOM
templates on both DOM shapes, 29/342 SSR, 3/342 Vapor (44 and 151 after #6337 / #6308) on the committed
fixtures; the Croquis-informed S2 rewrites and the module-hoisting entry
remain open.

**P3-16 Phase exit.**

_First slice 2026-09-14:_ see
[P3-16 record](./phase-3-records/p3-16.md) for the executable Phase 3
status ledger that keeps TODO entries, slice records, and remaining-work
claims in sync. Phase exit remains open until the gates below are terminal.
_Second slice 2026-09-15:_ the same guard now rejects stale sibling records
that still describe a completed Phase 3 task as unfinished.

- [ ] Vapor: TS-33 behavioral parity green; SSR **and VDOM**: TS-11 byte-empty (P3-7 changes patch-flag derivation, so DOM parity re-gates here)
- [ ] TS-31 source-map coverage ≥ budget on all three backends
- [ ] Production reach: `[reach]` floors held and raised, production-path DOM parity oracle empty, no DOM shape refused for its Croquis summary (P3-17)
- [ ] Vapor compile bench beats the pinned double-transform floor
- [ ] TS-32 remarks-diff clean; old vapor/ssr lanes + flags deleted
- [ ] TS-27/TS-28/TS-29/TS-30 all mandatory-green; TS-20 totality fuzz extended to S2→S3 green
