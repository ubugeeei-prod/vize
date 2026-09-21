# Impeto Op Reference

Impeto is S3's shared backend contract. S2 owns the authored semantic tree;
Impeto owns the flat operation stream, explicit regions, state edges, effect
scopes, and the decisions that DOM, VDOM, Vapor, and SSR must not rediscover in
separate emitters.

This page is the human reference for the 16 stable operation mnemonics. The
normative executable companion is the Lean package in `formal/impeto/`, which
parses the same S3 Folio text and records the first VDOM/Vapor trace labels.
Folio remains the concrete interchange syntax; see
[`folio-format-impeto.md`](./folio-format-impeto.md).

## Program Model

An S3 Folio program has four order-bearing sections:

| section   | role                                                                     |
| --------- | ------------------------------------------------------------------------ |
| `regions` | Root and child regions. A child records its parent region and owning op. |
| `ops`     | Flat operation records: id, mnemonic, region, optional effect, and span. |
| `edges`   | Explicit ordering and dependency edges between op ids.                   |
| `effects` | Effect scopes owned by dynamic ops.                                      |

The phase name describes which invariants consumers may assume:

| phase         | meaning                                                                                                                         |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `built`       | S2 order has been lowered into S3 ids. Partition facts can exist beside the program, but optional scheduling has not committed. |
| `partitioned` | Static/dynamic partition decisions have been validated for downstream readers.                                                  |
| `scheduled`   | Effect/data ordering has been committed; backends consume the schedule rather than reordering on their own.                     |

`effect=-` means the op is structural or statically decided in the current
program. An effect id means the op participates in update ordering and owns or
belongs to an effect scope. `dom-order` keeps authored sibling/binding order,
`effect-order` keeps observable dynamic order, and `data-dependency` is reserved
for fact/schedule edges that are not DOM order.

## Operation Semantics

| mnemonic                   | source meaning                                                       | VDOM trace            | Vapor trace            | notes                                                                                          |
| -------------------------- | -------------------------------------------------------------------- | --------------------- | ---------------------- | ---------------------------------------------------------------------------------------------- |
| `impeto.set-prop`          | Assign one named property, attribute, model channel, or sync target. | `patch-prop`          | `assign-prop`          | Usually dynamic; `v-model` and `vue.sync` lower here until a later pass specializes them.      |
| `impeto.set-dynamic-props` | Assign an object-shaped property bag or other dynamic prop group.    | `patch-dynamic-props` | `assign-dynamic-props` | Covers argument-less `v-bind` and CSS-bind carrier facts in the first lowering slice.          |
| `impeto.set-text`          | Set literal text, interpolation text, or `v-text` output.            | `set-text`            | `text-effect`          | Literal S2 text can stay effectless; interpolation and `v-text` are dynamic.                   |
| `impeto.set-event`         | Attach an event listener and its modifiers/options.                  | `patch-event`         | `listen`               | The op records listener placement; later passes may split handler shape decisions into facts.  |
| `impeto.set-html`          | Assign trusted raw HTML from `v-html`.                               | `set-html`            | `html-effect`          | Always a dynamic sink unless a future verifier proves a static authored value.                 |
| `impeto.set-template-ref`  | Publish a template ref binding for backend materialization.          | `set-template-ref`    | `template-ref`         | Reserved in the op family before the lowering path starts emitting it directly.                |
| `impeto.insert-node`       | Materialize a host node in the current region.                       | `create-element`      | `create-node`          | Used for elements and comments in the first S2 to S3 lowering slice.                           |
| `impeto.prepend-node`      | Materialize a host node before the current insertion point.          | `prepend-node`        | `prepend-node`         | Reserved for anchor-sensitive insertion decisions.                                             |
| `impeto.directive`         | Apply a Vue runtime directive or directive-like marker.              | `apply-directive`     | `directive-effect`     | Covers generic directives plus `v-once`, `v-memo`, `v-show`, and `v-cloak` in the first slice. |
| `impeto.if`                | Select one conditional child region.                                 | `branch`              | `conditional-effect`   | Owns one region per branch; dynamic partition propagates into branch children.                 |
| `impeto.for`               | Iterate a child region for a source collection.                      | `iterate`             | `list-effect`          | Owns the repeated region; keyed and unkeyed update laws are P3-11 work.                        |
| `impeto.create-component`  | Create a component boundary and pass its props, events, and slots.   | `create-component`    | `component-effect`     | Component children inherit dynamic partition in the first lowering slice.                      |
| `impeto.slot-outlet`       | Render or describe a slot outlet/scope bridge.                       | `render-slot`         | `slot-effect`          | Used for S2 slots and slot-scope binding records.                                              |
| `impeto.get-text-child`    | Resolve an existing text child reference.                            | `get-text-child`      | `get-text-child`       | Structural reference op; it should not invent dynamic work by itself.                          |
| `impeto.child-ref`         | Resolve the first child anchor/reference for a region owner.         | `child-ref`           | `child-ref`            | Structural reference op used by backend scheduling and insertion.                              |
| `impeto.next-ref`          | Resolve the next sibling anchor/reference.                           | `next-ref`            | `next-ref`             | Structural reference op used when stable insertion points matter.                              |

The VDOM and Vapor labels above are intentionally small observations, not code
generation byte strings. They are the trace vocabulary used by the first Lean
reference runner so semantic drift appears before optional passes rewrite or
schedule the op stream.

## S2 Lowering Commitments

The first S2 to S3 bridge emits `built` programs. S3 op ids follow S2 page
order: owner op, attached bindings, then owned child regions. Every lowered op
also receives a partition fact beside the program, so SSR and later thin paths
can read canonical partition decisions without traversing S3 ops.

| S2 form                                                        | Impeto op                  |
| -------------------------------------------------------------- | -------------------------- |
| Element                                                        | `impeto.insert-node`       |
| Component                                                      | `impeto.create-component`  |
| Text                                                           | `impeto.set-text`          |
| Interpolation                                                  | `impeto.set-text`          |
| Comment                                                        | `impeto.insert-node`       |
| If                                                             | `impeto.if`                |
| For                                                            | `impeto.for`               |
| Slot                                                           | `impeto.slot-outlet`       |
| Named `v-bind`                                                 | `impeto.set-prop`          |
| Argument-less `v-bind`                                         | `impeto.set-dynamic-props` |
| `v-on`                                                         | `impeto.set-event`         |
| `v-model` / `vue.sync`                                         | `impeto.set-prop`          |
| Slot content/scope binding                                     | `impeto.slot-outlet`       |
| Generic directive / `v-once` / `v-memo` / `v-show` / `v-cloak` | `impeto.directive`         |
| CSS bind carrier                                               | `impeto.set-dynamic-props` |
| `v-html`                                                       | `impeto.set-html`          |
| `v-text`                                                       | `impeto.set-text`          |

Dynamic partition creates an effect scope and an `effect-order` edge from the
previous dynamic op. Static partition leaves the op effectless. This is only the
initial lowering discipline; P3-10 extraction and later scheduling must either
prove the exported partition is preserved or rerun validation before consumers
read it.

## Placement Alternatives

P3-10 keeps the choice of where an op's work runs as an overlay on the graph,
`Program::placements`, never as a rewrite of ops, regions, edges, effect
scopes, or operands. An op without a record runs `inline`, its canonical shape.
`vize_impeto::placement::annotate` records the other shapes; every recorded
alternative preserves meaning, so choosing among them is a cost question only.

| placement | legal when                                                                                                                                                                                                   | update-model effect                                                          |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| `inline`  | always                                                                                                                                                                                                       | the op runs in its own effect scope when dynamic                             |
| `hoist`   | an `insert-node` whose attached ops and nested regions are literal elements, comments, or text; no `ref`, `ref_for`, `ref_key`, `key`, or `is`; owned by an `if`, `for`, slot, or component region           | the subtree is materialized once and re-inserted; its ops leave update paths |
| `cache`   | a dynamic `set-event` with one plain JS handler under a static name, and no `for`, slot, or component region above it                                                                                        | the handler is created once instead of re-bound                              |
| `group`   | a dynamic `set-prop`, `set-dynamic-props`, `set-text`, or `set-html` reading one direct reference (`a` or `a.b`) that its keyed effect-order predecessor reads, in the same root, `if` branch, or `for` item | the op joins the effect unit of the first op of that contiguous run          |

`group` walks only keyed predecessors: a dynamic op with no reactive read
re-runs only when its controlling region re-renders, where op order is fixed.
Contiguity means no other keyed op can be reordered by a group, and an
identical direct reference inside one lexical scope names one binding, so the
unit re-runs exactly when each member would have. Slot and component content
can mix scopes and never groups.

Exported partition facts keep describing canonical S3 because the overlay
never changes an op's effect scope. `S3V010` re-derives every recorded
alternative and every committed choice: a chosen `group` must stay contiguous
with its committed unit, and a chosen `hoist` may not sit inside another. The
Lean reference does not interpret placements yet; the legality rules above are
the semantic argument until extraction output reaches a backend under TS-28.

## Try-Measure-Commit Extraction

`vize_impeto::extract::extract` chooses among recorded alternatives, starting
from the all-inline plan. For each candidate in record order it performs the
placement on a trial plan, simplifies locally (a hoist removes its subtree's
effect units, a group merges its unit into the leader's and unions their keys,
a cache drops its unit), measures, and commits only under the pinned rule.

| metric          | role       | measure                                                                                |
| --------------- | ---------- | -------------------------------------------------------------------------------------- |
| `reactive-edge` | constraint | sum over effect units of the distinct keys each reads                                  |
| `update-path`   | constraint | per unit, members plus live ops an `if`/`for` member re-renders, times the unit's keys |
| `emitted-size`  | objective  | operand bytes + 21 per effect unit + 29 per hoisted root + 27 per cached handler       |

A key is one distinct reactive-read operand (kind plus source text): the fact
approximation in scope. The byte constants are the lengths of the Vapor-shaped
overheads `_renderEffect(() => …)`, `const _hoisted_1 = …` plus its use, and
`_cache[0] || (_cache[0] = …)`. A candidate commits when no metric exceeds its
tier epsilon, constraints judged first, and at least `required_improvements_min`
metrics strictly improve; ties keep the simpler shape. Every measured trial
spends one unit of the component's candidate budget; a candidate blocked by the
committed plan (`subsumed`, `not-contiguous`) or reached after the budget is
spent (`budget-exhausted`) is recorded as missed without measuring. Tiers are
the `[optimization]` rows of `budgets.toml`, synced field for field by
`crates/vize_impeto/tests/optimization_budgets.rs`.

The pass writes only `PlacementRecord::chosen` and is described as
`Preserved::ALL`. `vize_impeto::optimize::OPTIMIZE` is the `s3` pipeline
`annotate-placements` then `extract-placements`; run through the pass manager,
each decision becomes one `s3.extract-placements` remark (registered in
`remarks-format.md`), and `PartitionFacts::stale` stays `None`. Under the
pinned zero epsilons a cache always costs 6 bytes more than the effect unit it
removes, so it is recorded as missed until a budget ratchet decides otherwise.

## Review Point

P3-5 lands before optional passes. A new Impeto op, a changed mnemonic, or a
changed Lean trace label must update this page, the Rust `OpKind` table, the
Lean parser/semantics, and the sync test in the same PR.
