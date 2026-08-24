# P2-9 Installment 5 — the element/binding family (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The absorbed-vs-pass-body split (the headline measurement)

The largest installment measures the family per file, and the
installment-4 pattern repeats at scale: **five of the six old step
files are dead code in the shipped lane** (zero call sites outside the
then-current `steps.rs`/`lib.rs` re-exports — the historical
`transform_element.rs` name, now `steps/element.rs`, whole;
`v_bind.rs` whole, `v_on.rs` whole except a shadowed local name,
`v_once.rs` whole against codegen's own `has_v_once` twin, `v_memo.rs`
all but the two codegen reads). The living code is
`lane/element.rs` (the v-model expansion and its validations) and
`codegen/*` (everything else). The port followed the living code:

- **Absorbed by lowering** (`lower/bindop.rs` + the `lower/binding.rs`
  dispatch): the `ui.bind`/`ui.on` op construction — name position
  (static / dynamic / the spread and object forms), modifiers verbatim,
  value — with the two **parser** normalizations mirrored byte-for-byte
  (`parser/attribute.rs:267-340`): the Vue 3.4 same-name shorthand
  (`:foo-bar` reads its camelized argument, `normalize.bind.same-name`)
  and the `.` dot shorthand's synthesized leading `prop` modifier. An
  authored-blank value materializes no expression — the shipped
  parser's behaviour, caught by the corpus lane's first surface run
  (directus `@contextmenu.stop=""`) and mirrored exactly. Measured and
  matched: the legacy lane emits **no** grammar diagnostic for a
  valueless `v-bind`/`v-on` (relief's `VBindNoExpression`/
  `VOnNoExpression` have zero live call sites), so the lowering emits
  none either. The **outlet props surface** landed with the ops:
  `SlotOp` gained attributes + bindings (the installment-3 gap — static
  slot props, `:x` binds and `@x` handlers on `<slot>`, `name`
  consumed by the shipped selection rule), retiring
  `defer.slot-props`; and the lowering now **captures template-wrapper
  `v-if` keys** into `Lowered::wrappers` before the unwrap drops the
  rest (`lower.branch-wrapper-key`) — the installment-1 wrapper-key
  gap's home.
- **DOM realization (P2-11):** everything the live lane does with the
  ops at codegen — patch flags, `class`/`style` normalization, event
  name casing (`create_on_name`), `withModifiers` guards, the v-model
  product props and helper selection, duplicate-attribute dedupe
  (#958), and the whole of `v-once`/`v-memo`: their only living reads
  are codegen's (`has_v_once`, `get_memo_exp` at block/inline/v_for
  codegen), so no op lands for them here — the measured "facts or
  dialect ops" answer is **neither yet**: their ops land with the stage
  that reads them, and the deferral messages now name that owner (the
  stale "(P2-9)" promise is gone from every remaining `defer.*`).
- **The pass bodies:**
  - `pass/vif.rs` + `vif/keys.rs` — the dynamic arm the installments
    have counted since series 1: a carrier's `:key` binding extracts
    beside the static attribute (first spelling in authored span order,
    the legacy `extract_key_prop` scan), slot-outlet roots extract from
    the new outlet surface, wrapper keys fold in from the capture
    channel, and the collision check is kind-blind text equality
    (`extract_key_value_str` under the default dialect; a bare `key` or
    an expression-less dynamic spelling never collides). A dynamic
    key's binding op **stays** on the surface — a pass removing a
    binding op would shift every page-order id — and the fact records
    `bind_index` so realization and the projection exclude it.
  - `pass/vslot.rs` — the outlet's binding ids mint in the shaped
    recursion, and a `v-slot` spelled on an outlet (now a
    `ui.slot-content` binding there) fires the legacy `VSlotMisplaced`
    — the installment-3 "misplaced-on-outlet waits on `ui.bind`" line
    closed.
  - **`pass/vmodel.rs` — the one new pass**, and the measured answer to
    the family's "one pass or several": `v-bind`/`v-on` carry **zero**
    transform-time behaviour in the live lane, so their port is
    entirely lowering; what remains of the family is the live lane's
    two model validations, with relief's exact wording —
    `VModelOnScope` (the value against the alias **texts** of
    enclosing `v-for` scopes and the simple-identifier params of
    enclosing `v-slot` scopes, `enter_v_for_scope`/
    `enter_v_slot_scope_if_needed` mirrored, the carrier's own model
    outside its own scope exactly as `traverse_node` orders it) and
    `VModelArgOnElement`. The legacy lane _removes_ invalid models; a
    binding op cannot leave the S2 surface, so the pass publishes the
    removal's preserving twin — the sparse [`ModelFacts`] fault table.
    Pattern params contribute no scope name (#4365, the one-scanner
    rule): the recorded weaker behaviour, pinned loud, with the
    differential class (`models_pattern_scope`) counted instead of
    compared.

### The op-family additions (the canary, proven again)

`ui.bind` and `ui.on` landed in `vize_disegno` by the P2-5a ritual:
variants injected first, build captured broken —
`folio/owned.rs:282` (`own_binding`, E0004) on the lib, then the test
targets: `op_family` (the canary itself), `folio_laws`, `folio_mirror`
(E0063 on the `SlotOp` shape change) — then fixed and re-run green.
Size asserts: `BindOp` 72 / `OnOp` 72 held on the first guess;
`SlotOp` moved 56 → 104 with its props surface; ricalco's `BranchKey`
first guess of 56 was corrected to 48 by the probe (the ratchet
working) and `WrapperKey` 40 / `WrapperKeys` 24 / `ModelFacts` 1 held.
The folio grew the two binding lines (print + parse through the shared
optional-field walker, exact-message rejections added) and the outlet's
phased frame (`parse/frame.rs`, split under the budget); the verifier
walks the new lines and the outlet body (span order + nesting — the
new ops carry no regions); the TS-16 reference page now exercises both
ops on the outlet surface (`ops=10` → `ops=12`). No new opaque reason,
no scope-origin change.

### Classification (the review point)

**One new pass: `v-model`, `MandatoryDiagnostic`, barrier,
`Preserved::ALL` — the series' first diagnostic kind.** After three
installments of the recorded preserving-mandatory tension, the
diagnostic kind's literal definition finally fits: the pass
canonicalizes nothing and mutates nothing — its whole product is the
two user diagnostics plus the fault record. The fusion question the
contract flagged ("the first non-barrier would be a series first") is
answered by measurement: **no fusable pass lands** — everything the
family ports is diagnostics, which are meaning at every tier, and law 1
(mandatory ⇒ barrier, enforced at `PassDesc::new`) forces the barrier;
independently the scope environment is ancestor context a fused single
visit does not carry. Five lone barriers, const-pinned
(`group_count() == 5`, `is_fully_serialized()`); the walk pins moved
`walks=4` → `walks=5` in the five suites that hold them. The first
`Optional`/fusable pass remains unclaimed — `hoist_static` is still
the natural candidate.

### The differential lane extension (the binding projections)

The surface projection compares, per owner both trees keep (element,
component, template carrier, slot outlet, in document order): static
attributes (name + value), `v-bind` units (name static/dynamic/spread,
modifiers verbatim, value trimmed), `v-on` units (same shape), custom
directives, and the **reconstructed `v-model` contract** — the legacy
lane expands models in place, so its collector rebuilds the authored
contract from the product props (authored props never share a source
span; product groups always do; the modifiers product rides the stub
span), native kept-directives compare directly, and the S2 side
excludes exactly the fault-table models the legacy lane removed.
Wrapper elements the S2 lowering unwraps are skipped as owners with
their leftover props counted. Counted classes, measured then decided:
`builtins_excluded` (the remaining `defer.*` set — corpus **482**),
`wrapper_attrs` (the wrapper-facts gap on the binding surface —
**858**), `models_pattern_scope` (**214** owners), entity-bearing
values (**33** templates, the text class over the surface),
`models_dynamic_arg` / `values_compound` / `keys_dynamic_arg` /
`keys_compound` (**0** everywhere on the corpus), and one genuinely
new S1-scope deviation the surface lane exposed on its first corpus
run: **`table_templates` (103)** — the legacy parser's in-table tree
construction (foster parenting of non-table content, implicit
`tbody`/`tr` insertion — `parser/element/table.rs`; witnessed on
element-plus `table.vue`, `<hColgroup>` fostered out of its
`<table>`) against S1's authored nesting; the class skips the surface
half per template, is counted, and belongs to the future S1
tree-reconciliation story. The chain check's key half now **compares**
what installments 1–2 counted: dynamic keys, wrapper keys, and outlet
keys (`keys_dynamic`/`keys_wrapper` replace the `keys_template_if`/
`keys_slot_root` skip counters; the legacy `:[key]` arg-content quirk
stays counted as `keys_dynamic_arg`, deliberately never imitated).

- **Battery**: 58 → 75 templates (the surface half: bind forms and
  both parser shorthands, on forms incl. the bare listener and the
  object form, native/component/argument/modifier models, the three
  invalid-model classes, the classifier-ambiguity class, a custom
  directive, outlet props, the still-deferred built-ins, static attrs,
  the entity-attr class, the dynamic wrapper key). A `duplicate-attrs`
  template was measured out: `DuplicateAttribute` is a **hard** legacy
  parse error (the vue2-elm corpus skip), so the #958 dedupe question
  is outside both lanes' shared domain. Honest label (the installment-4
  convention): the key-half pins were predicted and confirmed
  (keys 14/2/2), the surface tallies were taken from the first run
  after a per-class plausibility audit — **126 owners, 7 attrs, 5+1+1
  binds, 3+1+1 ons, 1 directive, 3 models, 2 invalid, 1 dynamic-arg,
  2 pattern-scoped, 2 keys excluded, 2 built-ins, 1 wrapper attr,
  1 entity template** — and confirmed unchanged by every later run.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the standing command; run twice 2026-08-21, byte-identical):
  12,215 files, 12,021 templates, **12,017 compared, zero
  divergence** — the if, for, slot and text halves exactly
  installment 4's numbers, the keys now **45 static + 81 dynamic + 0
  wrapper compared** (the 81 were installment 1's counted skips), plus
  **120,734 owner surfaces: 119,584 attrs, 54,698 binds (+6 dynamic,
  +4,461 spreads), 12,877 handlers (+3 dynamic, +55 object forms),
  1,260 custom directives, 7,627 `v-model` contracts** compared;
  `models_invalid` **0** corpus-wide (no real template authors an
  invalid model — the two verdict paths are battery-pinned). The 4
  skips are the same named hard-parse-error files;
  `skipped_s2_errors` = 0. One fine asymmetry is deliberate and
  recorded rather than smoothed: `keys_excluded` reads 80 against 81
  compared dynamic keys — one dynamic branch key lives inside a
  `table_templates` template, whose surface half (the exclusion's
  accounting home) skips while its chain half still compares. Two
  corpus catches drove lowering fixes before the clean runs: the
  authored-blank value (directus) and the in-table class
  (element-plus), each recorded above.

### The residual class: measured, unmoved — deliberately reframed

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). The contract named this installment the
strongest candidate yet to move the number (v-bind/v-on expressions
are the reparse residual's largest feeders); measured: **unmoved**,
for the standing structural reason — the S2 lane feeds no shipped
`rewrite_expression` site, so no installment of this series can move
it: the number moves when P2-5b's widening (or P2-11's read) puts S2
structure in front of the shipped prefixer. What this installment
contributes is the largest prepared feed so far: every `v-bind` value
and `v-on` handler now sits in an op with a retained AST where
admitted, beside `ForFacts`/`SlotFacts`/`TextFacts`/`ModelFacts`.

### TS-17

`crates/vize_ricalco/tests/vmodel_pass_snapshot.rs`, two committed
fixtures → lower → pipeline → full normalized folio snapshots:
`tests/fixtures/vmodel/bindings.vue` (bind/on forms with both parser
shorthands applied, the dot shorthand's `prop` modifier, the same-name
camelized value, native and component models unexpanded, the outlet's
props surface, the spread/object/dynamic-name forms) and `invalid.vue`
(the three diagnostics in pipeline order — misplaced-on-outlet from
the v-slot pass, then on-scope / arg-on-element in page order — with
every op kept and the fault table as the removal's twin). Supplements
pin `walks=5`, the exact fault kinds, and the diagnostic order. The
four earlier snapshot suites re-pinned `walks=5`; their folio
snapshots did not move (the new ops appear only where authored).

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_ricalco` — 116 tests (19 unit + the
  P2-8 suites + the vfor/vslot/text suites + `vif_pass` 8 +
  `vif_pass_keys` 5 (the series-5 key arms, split under the budget) +
  11 `vmodel_pass` + 2 vmodel snapshots + the metamorphic 7) green; `cargo test -p vize_atelier_core` fully green (witness suite
  8 tests: the v-model relief pins and the end-to-end scoped-model
  check added); `vize_disegno` green (the canary grew the two arms,
  TS-16 reference page extended, rejections exact); davinci / sinopia
  suites green. The ricalco lowering battery census moved and was
  re-pinned deliberately: **(83, 28, 101, 1)** — five battery
  `v-bind`/`v-on` deferral Infos became ops (78 → 83 ops, 33 → 28
  diagnostics; records unchanged on purpose — one `lower.bind`/
  `lower.on` record replaces each `defer.*` record). The lowering
  corpus lane re-run twice, identical: 12,215 files, 12,215 checked,
  with_diagnostics **804** (10,142 → 804: 9,338 corpus files whose
  only findings were the retired `defer.v-bind`/`defer.v-on`/
  `defer.slot-props` Infos — the sharpest single measurement of what
  this installment absorbed).
- **Metamorphic (TS-21)**: the reorder quotient extended to the
  outlet's new props surface (`normalize.rs` sorts slot attributes);
  matrix plane census unchanged (321 mutations, pinned); the corpus
  shard reproduces its exact line; the full corpus run twice with
  identical counts, **179,992 mutations, zero divergences**.
- **TS-13**: `assertion-lint: OK`, allowlist untouched.
- **TS-11, deferred precisely**: no shipped path touched — no
  `Cargo.toml` changed (the ops live in unpublished `vize_disegno`,
  the lowering/passes in unpublished `vize_ricalco`, the comparator in
  dev-dep test space), so the dependency graph the publish gate checks
  is bit-identical to the last green run and the mechanical argument
  stands; the clean-fixture `corpus-diff` sweep recipe stands as
  recorded in installment 1.
- **House rules**: every file ≤ 350 after the deliberate splits
  (`lower/bindop.rs` 160; `lower/structural.rs` 287 +
  `structural/wrapper.rs` 147; `pass/vif.rs` 289 + `vif/keys.rs` 149;
  `pass/vmodel.rs` 343; disegno's `folio/owned/binding.rs` 135 +
  `folio/print/binding.rs` 140 + `folio/parse/frame.rs` 180; the
  comparator split four ways — `surface.rs` 178 types +
  `surface_check.rs` 220 + `surface_old.rs` 250 + `surface_old_help.rs`
  136 + `surface_s2.rs` 141, with `s2_lane.rs` back at 335, and the
  vif suite split into `vif_pass.rs` 237 + `vif_pass_keys.rs` 192); no
  `mod.rs` under `src/`; ricalco stays `no_std + alloc` (wasm32-wasip2
  builds green for disegno + ricalco, via the P2-4 sysroot overlay);
  clippy house invocation clean after two first-cut findings
  (`manual_contains`, a flattenable `if let` loop) plus the
  comparator's conversion nits, the new/touched test targets
  additionally clippy-clean under `--tests`; `cargo fmt --check`
  clean; the fuzz workspace (`s1_lowering`, `folio_parse`)
  `cargo check`-green over the extended folio grammar.
- **Benches**: none touched, none added — no shipped path gains code.
- **TS-12**: `croquis-consumption.md` regenerated (the new test files
  move the naive consumer counts); all `--check` entries green.

### Gaps and owners (what later installments inherit)

- **Closed here, cited**: dynamic `:key` extraction + the dynamic
  collision arm (installments 1–2's `keys_dynamic` skip, corpus 81 →
  compared); slot-outlet branch keys (installment 1's
  `keys_slot_root` → compared under the static/dynamic counters);
  `<template v-if>` wrapper keys (installments 1–2's
  `keys_template_if` drop → captured at lowering, folded by the pass,
  compared); the outlet binding surface incl. forwarded props
  (installment 3's `defer.slot-props`) and the misplaced-on-outlet
  diagnostic (installment 3's missing `VSlotMisplaced` twin);
  `defer.v-bind` / `defer.v-on` retired everywhere.
- **Stays, measured**: the `<template v-for>` wrapper key (both lanes
  keep it outside the for projection by design — the legacy lane
  leaves the template element in its tree, S2 records the drop; the
  keyed-fragment realization at P2-11 is the forcing point for a
  `ui.for` wrapper surface); the conditional slot carriers
  (installment 3's third hat, re-measured at exactly **111**, now also
  visible as `wrapper_attrs` 858 on the binding surface — the legacy
  lane models them only inside dynamic-slot codegen, so a comparison
  needs P2-11's realization on both sides); the `v-slots` spread
  (re-measured **0** beside real slot features, still a
  `vue.directive` ride-through); `v-model` dynamic arguments
  (`models_dynamic_arg` corpus **0**, battery-pinned, still deferred at
  lowering — retiring it means a `ModelOp` shape change no real
  template yet demands); `v-once`/`v-memo` and the `v-html` family
  (dead transform files, living reads all codegen — P2-11 owns their
  ops; corpus share of the legacy props: `builtins_excluded` 482);
  pattern-params scope names (#4365's seam, `models_pattern_scope`
  214); and the new **in-table tree construction** class
  (`table_templates` 103 — the S1 v1 no-reconciliation scope's third
  measured face, after entities and comments).
- **Next installment**: `hoist_static` remains the standing
  recommendation — the first S2 **analysis** pass, the first genuine
  `Optional` candidate (the fusion machinery's first real test), and
  its static-type lattice reads exactly the normalized binding surface
  this installment finished.
