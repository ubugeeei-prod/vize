# P2-9 Installment 6 — `hoist_static` as the first Optional analysis pass (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The absorbed-vs-pass-body split (the headline measurement)

The series' pattern breaks here: unlike installments 4–5's dead step
files, `hoist_static.rs` is **live in the shipped lane** — `lane.rs:295`
runs it after traversal (`atelier.transform.hoist_static`), and
`codegen/children.rs:232` reads `is_static_node` besides. The living
code splits three ways, and the split _is_ the design:

- **The analysis** — `static_type.rs`'s whole-subtree lattice
  (`StaticType`: NotStatic / HasDynamicText / FullyStatic) and
  `props.rs`'s per-prop hoistability (`is_hoistable_static_prop`,
  `has_static_props`, the two nested-children predicates) — is the pass
  body: `vize_s1_to_s2::pass::hoist`, published per owner as
  [`StaticFacts`] (level + `props_hoistable` + `nested_static` +
  `native_descendants` + `foreign`), dense over `ui.element` /
  `ui.component`, keyed by page-order id, one provenance record per
  fact (`pass.hoist-static.fact`).
- **The decisions** — which node whole-hoists
  (`TemplateChildNode::Hoisted`), which props-hoist
  (`hoisted_props_index`) — are position- and option-dependent
  (`is_root`, the inherited `hoist_static_vnodes` flag,
  `hoist_static`/`inline`/`scope_id`), so they stay with the stage that
  holds position and options: **P2-11 realization**. The differential
  lane replays the shipped position rules over the facts to prove the
  facts _determine_ the decisions.
- **The realization** — `VNodeCall` construction, scoped-CSS attribute
  injection, `_hoisted_N` emission, `should_use_block`,
  `count_dynamic_children` — moves whole to P2-11, untouched here.

### Classification (the review point): the series' first `Optional`

**`Optional`, `Fusable`, `Preserved::ALL`** — the milestone the series
has carried as an open question since installment 1, and the
measurement supports it rather than merely permitting it:

- _Optional_ because the shipped lane proves it about itself:
  `TransformOptions::default()` ships `hoist_static: false` (only the
  DOM layer opts in), the step emits **no diagnostic**, and un-hoisted
  output is correct output — skipping loses optimization facts and
  nothing else, `PassKind::Optional`'s literal definition.
- _Fusable_ because the lattice is a synthesized attribute — one
  post-order visit, each owner's fact from its own surface plus its
  children's already-computed facts, no sibling lookahead, no fixpoint
  — which is `Fusability::Fusable`'s own definition ("single-visit,
  local, synthesized-attribute style").
- **What the fusion machinery actually did with it, pinned**: the P2-2
  grouping rule put the pass in the pipeline's **first non-barrier
  group** — `group_count()` 5 → 6, `group(5) == {start: 5, len: 1,
is_barrier: false}`, the group's preserved set `ALL` — all
  const-pinned in `pass.rs` plus the runtime twin
  (`the_fusion_plan_is_five_lone_barriers_plus_one_fusable_singleton`).
  `is_fully_serialized()` stays true in its literal sense (six groups
  for six passes — fusion still buys nothing), because the singleton's
  only neighbour is a barrier: the const fusion pins now guard a live
  invariant — the next fusable pass to land adjacent joins this group
  and drops the walk count below the pass count for the first time.
  The walk pins moved `walks=5` → `walks=6` in the six suites that
  hold them.

### The const rule: the pessimal law's first real consumer

`pass/hoist/consts.rs` is where P2-5b's `ExprRef` const-classification
capability is consumed for real: an [`ExprRef::Opaque`] bind value is
**never** constant — the answer is `OpaqueExpr::is_constant` itself
(pessimal law 3), and for the first time it changes a published fact
(battery-pinned: `:n="a b"` blocks its element's hoistability). For
retained JS the rule is **deliberately weaker** than the shipped
classifier (`is_constant_simple_expression` with `bindings: None`):
no identifier at all (the shipped rule admits `vize_croquis`'s global
allowlist — a std crate this `no_std` crate must not grow an edge to),
no `this` (a measured shipped quirk: its visitor checks identifier
references only, so `:x="this.y"` is shipped-constant), no TS-only
spelling (the shipped mjs re-parse refuses those, and excluding them is
what keeps the rule **one-sided**: S2-constant ⇒ shipped-constant, so
every divergence is an S2 under-hoist), plus the four context
substrings mirrored byte-for-byte. Each narrowing is pinned by test;
the differential class (`consts_templates`, corpus **96**) is counted,
never compared — the #4365 record-the-weaker-rule precedent.

### The differential lane (decisions, not realization bytes)

The comparator gained a fourth kind of projection: a **hoist-armed
second legacy run** (same parse options, `hoist_static: true`) whose
_actual mutations_ are the old side's truth, walked in lockstep with
the run-1 tree and the S2 folio. Two structural safeguards keep the
comparison honest:

- **Replay control is legacy ground truth**: the walk descends exactly
  where the shipped driver descended by re-asking the shipped
  predicate (`get_static_type`, now `pub` alongside the widened
  `codegen::is_constant_simple_expression`) on the run-1 tree — a fact
  divergence can never desynchronize the traversal, it surfaces as one
  aligned verdict mismatch. The S2 facts enter only through
  `predict()` — the shipped position rules over the published facts —
  and the per-element vnodes-flag contribution is asserted across
  lanes besides.
- **A shape pre-check** (owner kinds + nesting, wrapper templates
  unwrapped identically on both sides) turns any S1-tree nesting
  deviation into a counted class instead of a walk panic
  (`tree_templates` — measured **0** on the whole corpus: outside the
  already-counted in-table class, the two S1 front ends agree on
  structural nesting everywhere).

Counted classes, measured then decided: template-level `vpre` (0),
`table` (103 — the surface class's twin), `models` (77 — legacy
_removes_ what S2 only faults, plus the pattern-scope seam),
`classifier` (222 — lowercase non-native tags: legacy element, S2
component, both the lattice and the vnodes flag flip), `consts` (96);
element-level `comments_elements` (513 — S2 carries no comment ops;
the element and its ancestors skip, descendants still compare, sound
because a comment-bearing element is legacy-`NotStatic` and therefore
demonstrably descended); subtree-level `builtins_subtrees` (344 — a
deferred builtin poisons the inherited vnodes flag below it);
`wrapper_hoists` (0 corpus-wide, battery-pinned at 1 — a `<template
v-for>` wrapper with static attrs props-hoists in the legacy lane and
S2 keeps no wrapper position). Taint propagates up through element
parents only — `If`/`For`/component/outlet boundaries stop it, which
is what keeps corpus comparisons alive around tainted islands.

- **Battery**: 75 → 90 templates (the decision half: root props / root
  dynamic-text props, the quiet-nesting quirk — a fully static subtree
  under a directiveless dynamic parent never hoists, mirrored — the
  directive-parent and component-parent vnodes arms, branch and
  for-carrier hoists, the `<svg>` foreign props arm, the `ref`
  blocker, a constant bind, and one template per counted class).
  Honest label (the series convention): the hoist tallies were taken
  from the first run after a per-class plausibility audit — **97
  verdicts, 17 whole, 6 props, 1 wrapper hoist, 4 comment elements,
  3 builtin subtrees, 1/2/3 consts/classifier/models templates** —
  and confirmed unchanged by every later run; every earlier half's
  counts were re-pinned for the 15 new templates in both witnesses.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the standing command; run twice 2026-08-21, byte-identical):
  12,215 files, 12,021 templates, **12,017 compared, zero
  divergence** — every earlier half exactly installment 5's numbers,
  plus **97,546 hoist verdicts compared: 7,886 whole-vnode hoists and
  5,934 props hoists agreed**, the counted classes as above.
  One divergence was caught and closed during bring-up, and it is the
  installment's measured lesson: `<TransitionGroup>` inside `<svg>`
  (directus `arrows.vue`) — the legacy `ns != Html` props-hoist arm
  reads the parser's _inherited_ namespace, and `ui.component`
  deliberately carries none — so [`StaticFacts`] gained its `foreign`
  bit (own namespace for elements, inherited context for components,
  the lowering's integration-point rule mirrored), pinned by the
  component-context and `foreignObject`-return unit tests.

### The cacheHandlers boundary (measured, deferred whole)

Installment 5 deferred the v-on caching question here; measured, none
of it is this pass's fact. The shipped decision
(`codegen/props/events.rs:270`) is `cache_handlers_in_current_scope()
&& dir.exp.is_some() && !is_setup_const_handler(..)`: (a) the
`cache_handlers` **option** and the `BindingMetadata`-driven
setup-const check are compile-option inputs, not template facts;
(b) the scope condition (`!has_slot_params()`) is ancestor context the
realization walk carries natively — the emitter is already walking the
tree when it needs the answer, and `SlotFacts` holds the params
surface besides; (c) the cache-index allocation is emission ordering.
Nothing is a whole-subtree synthesized attribute, so nothing belongs
beside the lattice: **the cacheable-handler analysis lands whole with
P2-11's realization**, and the boundary is recorded here rather than
half-landed as a speculative fact.

### The residual class: measured, unmoved — the standing reason cited

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). Unmoved, as installment 5 recorded the
structural reason once and for all: the S2 lane feeds no shipped
`rewrite_expression` site, so no installment of this series can move
the number — it moves when P2-5b's widening or P2-11's read puts S2
structure in front of the shipped prefixer. `StaticFacts` joins the
prepared feed.

### TS-17

`crates/vize_s1_to_s2/tests/hoist_pass_snapshot.rs`, two committed
fixtures → lower → pipeline → full normalized folio snapshots:
`tests/fixtures/hoist/levels.vue` (the lattice's rungs on one page:
fully-static subtree, dynamic text, the `<svg>` directive block with a
hoistable surface, the `ref` blocker, the weaker-const refusal) and
`positions.vue` (facts under branches, loops, components, carriers and
outlets — nine owners, one fact each). The snapshots are byte-identical
to the pre-pass folio — **the fact-not-mutation proof in the oracle
itself** — with the facts, `walks=6 passes=6`, and the empty
diagnostics channel as supplements. The five earlier snapshot suites
re-pinned `walks=6`; their folio snapshots did not move.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_s1_to_s2` — 135 tests green (16
  `hoist_pass` + 2 snapshots new; the vfor/vif provenance pins gained
  the pass's fact records — re-pinned deliberately);
  `cargo test -p vize_atelier_core` fully green (the plain witness
  re-pinned); davinci / disegno / sinopia suites green.
- **Metamorphic (TS-21)**: matrix plane census unchanged (the 90
  committed stubs, 321 mutations, pinned — the plane reads its own
  fixture dir, not the battery); the full corpus run twice with
  identical counts, **179,992 mutations, zero divergences** — the
  analysis pass rides inside every mutated run and moved nothing.
- **TS-13**: `assertion-lint: OK`, allowlist untouched.
- **TS-11, deferred precisely**: no shipped compile path changed — the
  one `src` edit in a published crate is a visibility widening
  (`codegen.rs`: `pub use helpers::is_constant_simple_expression`, so
  the comparator drives the shipped classifier instead of a drift-prone
  copy) plus the pass/comparator additions in `publish = false` /
  dev-dep test space; no `Cargo.toml` of a published crate changed, so
  the dependency graph the publish gate checks is bit-identical and
  the mechanical argument stands; the clean-fixture `corpus-diff`
  sweep recipe stands as recorded in installment 1.
- **House rules**: every new file ≤ 350 after the deliberate splits
  (`pass/hoist.rs` 183 + `hoist/lattice.rs` 297 + `hoist/consts.rs`
  170; the comparator split four ways — `hoist.rs` 238 +
  `hoist_walk.rs` 301 + `hoist_owner.rs` 295 + `hoist_old.rs` 240,
  with `mod.rs` at 345); no `mod.rs` under `src/`; ricalco stays
  `no_std + alloc` (the one new dependency, `oxc_ast_visit`, joins the
  already-present `oxc_ast`; wasm32-wasip2 builds green for disegno +
  ricalco via the P2-4 sysroot overlay); clippy house invocation clean
  and the new/touched test targets additionally clippy-clean under
  `--tests` (two first-cut findings fixed: the house string types in
  test space, one needless lifetime); `cargo fmt --check` clean.
- **Benches**: none touched, none added — the pass adds no code to any
  shipped path (the second legacy run is comparator test space); the
  fuzz workspace is untouched (no grammar or lowering change).
- **TS-12**: `croquis-consumption.md` regenerated (the new test files
  move the naive consumer counts); `--check` green.
- **Size asserts, probe-corrected**: `StaticFacts` first landed at 4
  bytes; the corpus-forced `foreign` bit moved it to **5**, the assert
  moved with the measured change (the ratchet recording a real design
  correction, not a guess).

### Gaps and owners (what later installments inherit)

- **Comments in S2**: third measured face after entities and in-table
  construction, now with a number on the decision surface
  (`comments_elements` 513). Dev-mode DOM realization must render
  comment vnodes, so P2-11 is the forcing point for either comment ops
  or a comment side-channel; this pass's facts are comment-blind by
  scope, recorded.
- **The classifier seam** (`classifier_templates` 222): the legacy
  parser's uppercase-only component rule vs the S2 `!is_native_tag`
  rule is now measured on the decision surface; reconciling the two
  classifiers (or recording one as canonical) is a pre-P2-11 decision,
  since realization picks `createElementVNode` vs `resolveComponent`
  by exactly this bit.
- **The weaker const rule**: `consts_templates` 96 is the price of not
  duplicating the allowlist into `no_std` space; if P2-11 wants those
  hoists, the honest path is a carton-level (or disegno-level) shared
  allowlist home, not a copy.
- **`inline` root arm**: dead under the comparator's default options
  (`inline: false`); its predicate (`native_descendants`) is computed,
  published and unit-pinned, and the arm itself becomes comparable
  when a comparator lane with inline options exists (P2-11's
  script-integration surface).
- **Next installment**: `legacy.rs` / `legacy_filters.rs` → `vue.*`
  dialect ops behind the `_legacy` feature — the last unticked
  transform line; after it the series closes into the flag-removal
  exit-gate work.
