# P2-9 Installment 3 — `v-slot` (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The absorbed-vs-pass-body split (the headline measurement)

The v-slot measurement lands **between** its predecessors and splits
**three ways**, because most of the legacy slot machinery never lived
in the transform lane at all — the old step file
(`crates/vize_atelier_core/src/steps/v_slot.rs` + `params.rs` +
`validate.rs`, 460 lines) is mostly _readers_ for a 1,666-line
`codegen/slots/` tree:

- **Absorbed by lowering.** The outlet half
  (`transform_slot_outlet`'s name + fallback) was already P2-8's
  `ui.slot` (implicit name normalized at lowering), with the outlet
  diagnostics; **this installment absorbs the spelling itself**: the
  P2-8 `defer.v-slot` becomes the attached **`ui.slot-content`** op —
  name position, modifiers, and params exactly as authored, an
  unauthored name kept `None` (never pre-normalized, so the pass can
  record who invented what). Two outlet-name parity fixes landed with
  it, matched to the shipped resolution and pinned: a value-less
  `name` reads as `default`, and a `:name` with no expression is not a
  name candidate (`drop.slot-name-hole`, recorded).
- **DOM realization (P2-11):** everything `codegen/slots/` does with
  the readers — `RenderSlot`/`CreateSlots` registration, dynamic slot
  objects, conditional carriers, the JSX `v-slots` spread — plus
  forwarded slot props on outlets (still `defer.slot-props`, waiting
  for `ui.bind`).
- **The pass body** (`vize_ricalco::pass::vslot`, one shaped
  page-order recursion): the **canonical grouping** `collect_slots` +
  the component-`v-slot` read establish — published as
  `SideTable<SlotFacts>` per `ui.component` (own spellings, template
  groups in child order with duplicate static names dropped silently,
  the implicit default last, carriers as op ids); `get_slot_name`'s
  canonicalization (modifier folding, the synthesized default); the
  four `validate.rs` diagnostics with relief's exact wording
  (misplaced / mixed / duplicate / extraneous-children, pinned against
  `ErrorCode` in the differential suite); and the slot-props **scope
  consumption** — the vfor pattern at slot boundaries.

### The Synthesized producer (the series' first)

Every name the normalization invents is a recorded fact: the canonical
name of a bare `v-slot` (rule `normalize.v-slot.default-name`) and the
implicit default group (rule `normalize.v-slot.implicit-default`)
carry `ScopeOrigin::Synthesized{rule}` inside
`SlotName::Static{text, origin}`; an authored `v-slot:default` spells
the same text under `Authored`. Pinned in both directions
(`a_bare_v_slot_synthesizes_the_default_name_never_authored`,
`an_authored_default_name_is_authored_never_synthesized`), and proved
cross-lane by the comparator: the S2 origin class must equal the
legacy lane's invented class on every group — **2,993 invented groups
over the corpus, zero divergence**. The pass synthesizes no _binding_
names (slot-prop names stay authored or pattern-pending under #4365),
so the scope module's mint-a-fresh-tag obligation for synthesized
bindings has still never fired — recorded, not smoothed.

Hygiene across slot boundaries, consumed and pinned: the lowering now
keys each spelling's scope facts to **its own binding op** (P2-8's
provisional carrier-op home consumed — two spellings on one
`<template #a="x" #b="y">` are two introduction sites, previously
unrepresentable in the element-keyed table; pinned by
`two_spellings_on_one_carrier_are_two_introduction_sites`). The pass
validates entry-present ⟺ params-authored (both directions asserted as
law), tag freshness across every slot site, and bindings byte-equal
through the one scanner; the capture pin
(`a_slot_prop_never_captures_an_outer_authored_binding`) holds a
v-for `item` against a nested slot-prop `item` as two `(name, tag)`
identities.

### Classification (the review point)

**`MandatoryLowering`, barrier**, `Preserved::ALL`:

- _Mandatory_: the grouping is what slot compilation reads and the
  four diagnostics fire at every tier (the legacy validation ran
  unconditionally) — skipping loses both meaning and errors.
- _Lowering, not Diagnostic_: the pass establishes the invariant later
  stages assume (canonical grouping, synthesized names as recorded
  facts) — the kind's literal "the kind that canonicalizes". It also
  diagnoses, so `MandatoryDiagnostic` is _true but partial_ of it;
  installment 2's preserving-mandatory tension applies here in the
  milder diagnosing form — the pass **preserves the tree** (a binding
  op cannot leave the surface without shifting every page-order id
  after it; grouping is a fact, not a mutation), and the const pins
  keep the choice loud.
- _Barrier_: law 1 forces it, and slot gathering is literally the kind
  taxonomy's own barrier example (`Fusability`'s docs) — a component's
  groups read across its whole child list.
- **The fusion answer**: three lone barriers, const-pinned
  (`group_count() == 3`, `is_fully_serialized()`); the walk pins moved
  `walks=2` → `walks=3` in all three suites that hold them — the
  serialization's measured cost, re-pinned deliberately.

The pass drives its own **shaped recursion** over the shared
`PageWalk` (flat visitation cannot hand a component its children's
carrier ids or its bindings' own ids): the mint arithmetic keeps its
one home in `pass/walk.rs`, the recursion's structure is asserted
against the minted accounting on every run, and `walk.rs`'s docs name
the two shapes.

### The differential lane extension

The slot projection compares, per template, **slot-active units** and
**outlet names**. The unit rule was the design problem: the two lanes
_classify components differently_ (the comparator's legacy parse runs
default options — uppercase-first only — while S2 uses
`is_native_tag`), so units are defined lane-neutrally: a node whose
**authored tag** is not a native tag and that carries a `v-slot` or a
direct `<template v-slot>` child — computed identically on both trees,
so neither classifier enters the projection. The legacy projection
reads through the shipped helpers themselves (`get_slot_name`,
`get_slot_props_string`, `is_dynamic_slot`), making it the legacy
lane's own reading rather than a re-implementation. Compared exactly,
per group: canonical name (static text with modifier folding, or
trimmed dynamic expression text), the **invented-vs-authored class**
(the Synthesized producer's cross-lane witness), params text
(blank-normalized), group order, and dedupe behaviour; outlets by
name. Implicit-only components (plain children, no `v-slot` anywhere)
are deliberately outside the unit rule — their grouping is
definitional `[default]`, pinned suite-side, with the DOM bytes
arriving at P2-11.

Counted classes, measured then decided:

- `units_conditional` — a `<template v-if/v-for v-slot>` carrier under
  the unit: modeled by **neither** projection (legacy groups it only
  in dynamic-slot codegen; S2 dropped the wrapper's `v-slot` at
  lowering under `drop.template-attribute`) and both lanes therefore
  agree blindly; counted so the blind spot has a number — **111
  units** on the corpus, the recorded wrapper gap's new measurement.
- `units_filler_default` — only the legacy raw predicate (comments,
  kept single spaces) would synthesize an implicit default; the shared
  predicate (whitespace-only text and comments are filler) drives both
  projections and the class is counted — **5 units** on the corpus
  (comment-only trailing content), owned by the missing S2 comment
  story.
- `units_forwarded` — the JSX `v-slots` spread beside real slot
  features: **0** on the corpus.

- **Battery**: 30 → 44 templates (the v-slot half: named + params
  groups, the two synthesized-name directions, modifier folding,
  dynamic names, duplicate/mixed/extraneous, outlet names incl. the
  value-less `name`, the comment-filler and conditional-carrier
  classes, whitespace-only agreement). Exact-pinned in the plain
  witness and the corpus entry — **predicted before the first run and
  confirmed unchanged by it**: 44 compared; if half 21 ops / 39
  branches (the conditional-carrier template adds one chain; keys
  unchanged 13/2/1/1); for half unchanged; slot half **12 units, 16
  groups, 7 params, 4 invented, 1 dynamic, 1 conditional, 1 filler, 6
  outlets, 1 dynamic outlet**.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the standing command; run twice 2026-08-21, byte-identical):
  12,215 files, 12,021 templates, **12,017 compared, zero
  divergence** — the if and for halves exactly installment 2's
  numbers, plus **6,090 slot units, 10,498 groups (2,147 params
  compared, 2,993 invented, 18 dynamic), 2,897 outlets (185
  dynamic)**; counted classes 111 conditional / 0 forwarded / 5
  filler. The 4 skips are the same named hard-parse-error files;
  `skipped_s2_errors` = 0.

### The residual class: measured, unmoved

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). The contract flagged slot params as a
possible first mover (they feed `rewrite_expression` in the old lane);
measured: **unmoved**, for the same reason as installment 2 — the S2
lane feeds no shipped `rewrite_expression` site; `SlotFacts` joins
`ForFacts` as the prepared feed the P2-5b widening will read.

### TS-17

`crates/vize_ricalco/tests/vslot_pass_snapshot.rs`, two committed
fixtures → lower → pipeline → full normalized folio snapshots:
`tests/fixtures/vslot/groups.vue` (pattern params, modifier-folded
group, implicit default, the bare-`v-slot` synthesized name, a dynamic
group with filler-only siblings) and `invalid.vue` (all four
diagnostics in one artifact, the duplicate dropped silently from the
grouping while the authored default survives). The snapshots show the
**kept** surface — `ui.slot-content` lines with name/mods/params
exactly as lowered — since this pass, like v-for, moves nothing;
supplements pin `walks=3`, the four scope consumptions
(`scope #0..#3`, the pattern position enumerating zero names), and the
exact diagnostic order.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_ricalco` — 77 tests (17 unit + the
  P2-8 suites + vif/vfor suites + 11 `vslot_pass` + 2 vslot
  snapshots) green; `cargo test -p vize_atelier_core` fully green
  (witness suite now 6 tests: the four-message relief pin and the
  end-to-end duplicate-slot check added); disegno (the op-family
  canary grew the `ui.slot-content` arm) / davinci / sinopia suites
  green. The ricalco lowering battery census moved and was re-pinned
  deliberately in both lanes: (89, 33, 107, 1) — the `#named` battery
  fixture now lowers to a `ui.slot-content` op (+1 op) instead of the
  defer Info (−1 diagnostic). The lowering corpus census moved the
  same way and is re-recorded deliberately: 12,215 files,
  with_diagnostics **10,142** (P2-8's 10,416 included 274 templates
  whose only finding was the `defer.v-slot` Info — the deferral this
  installment retired; run twice, identical).
- **TS-13**: `assertion-lint: OK`, allowlist untouched.
- **TS-11, deferred precisely**: no shipped path touched — no
  `Cargo.toml` changed (the new op and pass live in the unpublished
  `vize_s2`/`vize_ricalco`; the comparator additions are
  dev-dep test space), so the dependency graph the publish gate
  checks is bit-identical to the last green run and the mechanical
  argument stands; the clean-fixture `corpus-diff` sweep recipe stands
  as recorded in installment 1.
- **House rules**: every file ≤ 350 after three deliberate splits
  (`pass/vslot.rs` 290 + `consume.rs` 243 + `spell.rs` 171 +
  `group.rs` 213; folio parse gained `binding_line.rs` for the
  attached-binding line grammar; the comparator's slot projection
  split legacy-side into `slots_old.rs`); no `mod.rs` under `src/`;
  ricalco stays `no_std + alloc` (wasm32-wasip2 build green);
  `SlotName` 48 / `SlotBound` 24 / `SlotParams` 56 / `SlotCarrier` 8 /
  `SlotGroup` 112 / `SlotFacts` 24 size asserts (first guesses were
  corrected by the probe — the ratchet working) and `SlotContentOp`
  72 in disegno; clippy house invocation clean, the new/touched test
  targets additionally clippy-clean under `--tests`; `cargo fmt
--check` clean.
- **Benches**: none touched, none added — no shipped path gains code.

### Gaps and owners (what later installments inherit)

- **Conditional/iterated slot carriers, now measured**: 111 corpus
  units hold a `<template v-if/v-for v-slot>` child that neither lane's
  projection models (legacy realizes it in dynamic-slot codegen; S2
  drops the wrapper's spelling at lowering). This is the
  template-wrapper-facts gap from installments 1–2 wearing its third
  hat; one wrapper-facts home closes all three when an installment
  needs it, and P2-11's dynamic-slot realization is the natural
  forcing point.
- **Outlet attribute surface**: forwarded slot props stay
  `defer.slot-props` (they land with `ui.bind`), and a `v-slot` on a
  `<slot>` outlet still defers (`defer.slot-directive`) — so the
  legacy `VSlotMisplaced` on outlets has no S2 twin yet; both move
  with the outlet's binding surface.
- **Comment content**: the 5 filler-default units are comment-only
  slot content — real DOM output in the legacy lane, invisible to S2
  (comments drop at lowering, the S1 v1 scope). The class is counted;
  it belongs to whatever installment gives comments an S2 op.
- **The `v-slots` spread**: zero corpus share beside real slot
  features; stays a `vue.directive` ride-through until P2-11's slot
  object realization decides it.
- **Next installment**: `transform_text` remains the standing
  recommendation — it unlocks the `Compound` opaque producer, and its
  whitespace-condense port is exactly what would let the S2 lane model
  the kept-space filler class this installment could only count.
  `transform_element`/`v_bind` is the alternative: it retires
  `defer.v-bind` (dynamic keys, outlet props) and the two dynamic-key
  counters that have waited since installment 1.
