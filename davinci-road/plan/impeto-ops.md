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

## Review Point

P3-5 lands before optional passes. A new Impeto op, a changed mnemonic, or a
changed Lean trace label must update this page, the Rust `OpKind` table, the
Lean parser/semantics, and the sync test in the same PR.
