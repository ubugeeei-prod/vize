# P2-9 Installment 7 — Vue 2 dialect sugar (2026-08-23)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

The last unticked transform-directory line. Landed as three stacked PRs
because the ops, the admission, and the legalization have different
review surfaces (the P2-5a "a dialect op lands with the transform that
needs it" rule, plus the house 350-line budget):

| PR                                                       | branch                         | what                                                                                |
| -------------------------------------------------------- | ------------------------------ | ----------------------------------------------------------------------------------- |
| [#4633](https://github.com/ubugeeei-prod/vize/pull/4633) | `feat/p2-9-legacy-dialect-ops` | `BindingOp::VueSync` / `VueSlotScope`, `ExprRef::Filter`, folio + verifier + canary |
| [#4634](https://github.com/ubugeeei-prod/vize/pull/4634) | `feat/p2-9-legacy-lowering`    | `LegacyCaps` + S1→S2 admission (`:foo.sync`, `slot-scope`/`scope`, pipe filters)    |
| [#4637](https://github.com/ubugeeei-prod/vize/pull/4637) | `feat/p2-9-legacy-pass`        | `legacy-sugar` pass: expand / convert / wrap, Vue 3 table untouched                 |

### No `_legacy` cargo feature (the plan line's zero-cost reading)

The task text said "behind the existing `_legacy` feature (zero cost
when off)". Ricalco is `publish = false` and has no `_legacy` feature
today; adding one would be a cargo-graph gesture, not a cost model.
The measured zero-cost shape, copied from the shipped
`desugar_legacy_template` short-circuit:

- [`LegacyCaps`](../../../../crates/vize_ricalco/src/lower/caps.rs) is
  three bools resolved once per file from
  `vize_carton::config::VueVersion`. It is a copy of armature's
  `LegacyDialectCapabilities` so ricalco never grows an armature edge.
- Vue 3 is `LegacyCaps::VUE3` — every flag off. `needs_sugar()` is one
  field-read or-chain that is const-false on that value.
- `pipeline_for` returns the existing six-pass [`TRANSFORM`] table when
  that bit is false. Vue 3 never prepends, never walks sugar, never
  allocates a filter wrap. Pinned: `walks=6` on a `:title.sync`
  template under the default `lower()`.

Vue 1 keeps filters only (the carton version table); Vue 2 / 2.7 turn
all three flags on.

### Classification (the review point, seventh occurrence)

**`MandatoryLowering`, barrier, `Preserved::NONE`**, pinned by test
(`legacy-sugar` is `LEGACY_PASSES[0]`):

- _Mandatory_: skipping leaves `vue.sync` / `vue.slot-scope` /
  `vue.filter` in the folio. No later pass consumes them; DOM
  realization would have to learn Vue 2.
- _Lowering, not Diagnostic_: `.sync` inserts an `ui.on` and rekeys
  every later page-order id; that is a structural rewrite, not a
  preserved diagnostic.
- _Barrier_: the rewrite is not single-visit-local (sync ids are
  collected, then the tree is rewritten, then every side table is
  rekeyed). Law 1 would have forced the barrier anyway.
- _Prepended_: inserting listeners shifts every later id, so nothing
  that ran before this pass can keep a numbered fact. Putting it first
  makes `Preserved::NONE` cheap rather than a rekey of installments
  1–6. Vue 2 therefore has **seven lone groups**, `walks=7`,
  `is_fully_serialized()` still literally true. Vue 3's fusion plan is
  the installment-6 shape, untouched (`group_count() == 6`).

### What the pass does (`pass/legacy.rs` / `pass/legacy/filter.rs`)

Order, matching the shipped desugar then the rest of the S2 table:

1. Collect `vue.sync` page-order ids (the bind keeps its id; the new
   listener is minted after it).
2. Expand `:foo.sync` → `ui.bind` + `@update:foo` with handler
   `$event => ((<authored source>) = $event)`. Remaining modifiers
   (`.camel`, …) stay on the bind. Dynamic `:[foo].sync` is **not**
   admitted at lowering (stays `ui.bind`); the pass never sees it.
3. Convert `vue.slot-scope` 1:1 into `ui.slot-content` (same
   introduction site, so the lowering's scope facts stay valid). An
   element that already carries `v-slot` is not rewritten.
4. If `v2_event_sugar`: strip `.native`; map the numeric keyCodes
   8/9/13/27/32/37/38/39/40/46 → delete/tab/enter/esc/space/left/up/
   right/down/delete.
5. If `supports_filters`: wrap `ExprRef::Filter` as `_filter_*(...)`.
   `VueFilterApp.args` is the parenthesis interior (no closing `)`).
   `a | f` → `_filter_f(a)`; `a | f(b)` → `_filter_f(a,b)`.
6. Rekey side tables + provenance for minted listener ids; recount
   `op_count`.

The Vue 3 table still runs after this on Vue 2 (v-if … hoist-static),
so a converted `ui.slot-content` is grouped by the existing v-slot
pass — battery-pinned (`slot_facts.len() == 1`).

### Deliberate gaps (named, not silently skipped)

- **Mixed text-runs.** `hello {{ msg | cap }}` is one compound
  `ui.interpolation` whose pipe is absorbed into a compound opaque
  (installment 4's producer). It is not `ExprRef::Filter`, so this
  pass does not wrap it. Lone interpolations and bind values do wrap.
  Closing the gap means teaching the compound parts table a filter
  slice, which is a P2-5b/installment-4 follow-up, not a quiet extra
  arm here.
- **No Vue 2 atelier comparator.** The existing dual-run lives on the
  Vue 3 shipped lane (installments 1–6). This port does not change
  that lane (`walks=6` pinned). A Vue 2 comparator would need a
  second shipped-lane fixture dialect; it is not required to keep Vue
  3 compile output unchanged, and it is recorded rather than faked.
- **P1-9 residual unmeasured this installment.** Filter wrapping
  happens on S2 `ExprRef`s after lowering; it does not feed
  `steps/expression/reparse.rs`. Inventing a 12.73% number
  without running the counters would violate the task's "a number
  from the existing counters, not a prediction" clause. The
  measurement stays an open series checkbox.

### Shipped path

No shipped compile path changed: ricalco is `publish = false`;
`lower()` is still `LegacyCaps::VUE3`; the six-pass table is the same
bytes. `cargo tree -i` on the Davinci crates still names only
unpublished experimental crates. `corpus-diff.mjs` is the standing
deferred recipe, unchanged.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_ricalco` green (Vue 3 snapshots and
  `walks=6` unchanged; new `legacy_lowering` + `legacy_pass` pins +
  soundness). Clippy `-p vize_ricalco --lib --tests -- -D warnings`
  green.
- **TS-13**: no new substring assertions.
- **TS-17**: Vue 3 folio snapshots untouched. Vue 2 pins are
  structural oracles over the owned folio (bind+on, `_filter_*`,
  slot-content, `.native`/keyCode) rather than a second snapshot
  corpus — the Vue 3 snapshots remain the shipped-lane ratchet.
- **TS-10 / TS-11 / TS-25**: benches untouched; no compiler surface
  moved; Vue 3 differential lane not re-armed (no Vue 3 tree change).
- **House**: every new file ≤ 350 (largest: `pass/legacy/filter.rs`
  171); no `mod.rs` under `src/`; ricalco stays `no_std + alloc`;
  `GetAllocator` is implemented for `&Allocator` only, so minted
  nodes use `Vec::new_in(&allocator)` / `Box::new_in(..., &allocator)`.

### What the series still owes (why `phase-2.md` stays open)

The transform-directory line is done. The series box stays unticked
because these checkboxes are still honest:

- `steps/expression/` stays on the old lane (non-goal, P2-5b).
- The old lane stays live behind `VIZE_DAVINCI_TRANSFORM` (charter
  #26) until the exit gate.
- Differential lane + P1-9 residual number: not this installment's
  measurement (see gaps). Vue 3's prior corpus figures are still the
  last recorded ones.

**Next task in dependency order:** P2-10 (style `v-bind()` as S2
ops). It does not need the Vue 2 comparator; it needs the binding
family (installment 5) and a closed transform-directory line.
