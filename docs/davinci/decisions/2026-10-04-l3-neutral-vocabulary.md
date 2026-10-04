# Framework-neutral L3 vocabulary (2026-10-04)

Decision for [#6857](https://github.com/ubugeeei-prod/vize/issues/6857), a
Stage 2 design-only slice of [#6829](https://github.com/ubugeeei-prod/vize/issues/6829).
Choose neutral reactivity contracts plus a typed Vue Vapor program dialect.
This specifies the destination; it implements no new op or runtime target.

## Present artifacts

`davinci/vize_l3/src/op/kind.rs` defines 16 Vapor-shaped kinds with `l3.*`
mnemonics. `Program` stores compact ops, owned regions, explicit state edges,
effect scopes, value operands and placement records. The validator checks
artifact phases, references, containment, operands and effects.

The separate `decision::DecisionTables` records L2-node static levels,
dynamic bindings, placement and control containment for DOM/SSR/Vapor target
policies. Its native producer in `vize_l2_to_l3::decision` remains a #6839
skeleton. Empty tables are not completed analysis. Existing flat-program
support does not establish that this shared producer or every product is
native.

`lattice` already separates a reactivity class from its proof verdict and
uses compact effect/escape summaries. Its `SourceKind` explicitly describes
Vue `ref`, `reactive`, `computed` and related APIs; `BindingOrigin` also has
Vue-shaped names. Neutrality therefore requires more than renaming `SetProp`.

## Two surfaces stay distinct

Preserve #6839's demand split:

- Every requesting backend gets L2 plus neutral decision tables, keyed by
  ids from that same L2 artifact. DOM and SSR consume those tables directly.
- A flat program is constructed only when the selected target needs it,
  currently Vapor. Neutral reactive nodes and typed dialect nodes share
  that requested program; no second universal graph is built first.

L3 owns static/dynamic decisions, legal placement, dependency order and
update grouping. L4 owns runtime helpers, code emission, DOM patch flags,
cache numbering and module assembly. Target-specific eligibility criteria
remain in L3 policies; vocabulary neutrality does not move them into L4 or
pretend all targets have the same execution model.

## Neutral reactive contracts

These are proposed semantic records/kinds, not existing `OpKind` variants.
A signal record references an already-analyzed source; it does not introduce
a runtime object or wrap every JS variable in a new allocation.

| Contract     | Meaning                                                                                           | Required law                                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Signal       | A reactive source identity with read/write capabilities and lifetime, supplied by native L2 facts | Identity belongs to the original artifact; aliasing and unknown writes are conservative                          |
| Signal read  | Read a signal value inside a tracked or explicitly untracked scope                                | Tracking mode is explicit; source spelling never decides it downstream                                           |
| Signal write | Write through a proven source capability                                                          | Keep evaluation, invalidation and observable write ordering; a readonly or unknown capability cannot be invented |
| Derived      | A proven pure value depending on explicit source/value identities                                 | Reuse requires a proven class/verdict and preserved dependency/lifetime rules; effectful calls are not made pure |
| Effect       | An owned reactive update region with explicit dependencies, lifetime and cleanup obligations      | Preserve creation, update and teardown order; no target scheduler behavior is assumed                            |
| Ordering     | Typed data-dependency and observable-effect-order edges between graph nodes                       | Rescheduling never crosses an observable dependency or changes lexical ownership                                 |

A derived record does not promise universal laziness, caching, equality
comparison or subscription timing. Those are explicit framework contracts
and target policy facts. If a framework's behavior cannot be represented
exactly, its adapter retains a dialect node instead of claiming a generic
`Derived` is equivalent.

Missing capabilities, unknown aliasing and opaque/foreign values keep their
conservative verdict. They cannot become optimizable signals or pure derived
values merely because source text resembles a recognized API call. The
existing proof axis remains separate from classification: only proven facts
may fire an optimization.

## Review of the existing flat kinds

All current flat kinds are retained in a typed `framework::vue::vapor`
program dialect at the destination. Their destination wire prefix is
`vapor.*`; it names the runtime dialect, never the neutral level itself.
Moving a kind does not claim new semantics or change emission order.

| Current kind      | Current mnemonic       | Destination mnemonic      |
| ----------------- | ---------------------- | ------------------------- |
| `SetProp`         | `l3.set-prop`          | `vapor.set-prop`          |
| `SetDynamicProps` | `l3.set-dynamic-props` | `vapor.set-dynamic-props` |
| `SetText`         | `l3.set-text`          | `vapor.set-text`          |
| `SetEvent`        | `l3.set-event`         | `vapor.set-event`         |
| `SetHtml`         | `l3.set-html`          | `vapor.set-html`          |
| `SetTemplateRef`  | `l3.set-template-ref`  | `vapor.set-template-ref`  |
| `InsertNode`      | `l3.insert-node`       | `vapor.insert-node`       |
| `PrependNode`     | `l3.prepend-node`      | `vapor.prepend-node`      |
| `Directive`       | `l3.directive`         | `vapor.directive`         |
| `If`              | `l3.if`                | `vapor.if`                |
| `For`             | `l3.for`               | `vapor.for`               |
| `CreateComponent` | `l3.create-component`  | `vapor.create-component`  |
| `SlotOutlet`      | `l3.slot-outlet`       | `vapor.slot-outlet`       |
| `GetTextChild`    | `l3.get-text-child`    | `vapor.get-text-child`    |
| `ChildRef`        | `l3.child-ref`         | `vapor.child-ref`         |
| `NextRef`         | `l3.next-ref`          | `vapor.next-ref`          |

The flat `If`/`For` kinds describe Vapor realization. Neutral conditional
and loop ownership is already represented by L2 and the shared decision
controls; it is not necessary to promote these flat runtime kinds to core.
The same distinction applies to `CreateComponent` and `SlotOutlet`: shared
semantic component/content contracts do not imply a common runtime mount,
update or fallback implementation.

Use a closed typed sum of core and explicit dialect kinds, with typed
payloads/operand roles. Generic scheduling may read ids, ownership, effect
scope, dependency edges and declared capabilities; it cannot infer Vapor
meaning from a string name. Dialect legalization and L4 target encoding
exhaustively handle their own cases. Unsupported dialects produce a
diagnostic. No placeholder `svelte.*` or Solid runtime implementation lands
with the design.

Neutral wires for actual implemented reactive nodes use `l3.signal-read`,
`l3.signal-write`, `l3.derived` and `l3.effect`. Source identities may remain
side-table records rather than executable `signal-create` operations. Add a
kind only with a production producer and consumer; no unused enum expansion
is required to satisfy this design-only issue.

## Adjacent vocabulary migrations

| Current surface                                                                                                | Destination boundary                                                                                                                                     |
| -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SourceKind::{Ref, ShallowRef, Reactive, ShallowReactive, Computed, Readonly, ShallowReadonly, ToRef, ToRefs}` | Vue adapter recognizes exact native L2 framework facts and supplies neutral capabilities/effect summaries; API names stay `framework::vue`               |
| `BindingOrigin::Prop`, `ProvideInject`, `TemplateRef`                                                          | Neutral `Input`, `Context`, `HostReference`, with typed Vue origin detail retained beside provenance                                                     |
| `ReactivityClass::PropsStable`, `EffectKind::ReadProp`                                                         | Neutral `InputsStable` and `ReadInput`, preserving the current ordering/floor laws; framework adapters establish input stability rather than assuming it |
| `EffectKind` and `EffectSet`                                                                                   | Keep analysis summaries distinct from an executable reactive `Effect` scope; unknown calls/global writes remain conservative                             |
| `EdgeKind::DataDependency`, `EffectOrder`                                                                      | Neutral graph edges, keeping existing validation and observed order                                                                                      |
| `EdgeKind::DomOrder`                                                                                           | Typed `vapor.dom-order` realization constraint; generic scheduling preserves it without interpreting DOM payloads                                        |
| `decision::ControlKind::Slot`                                                                                  | Neutral content control boundary when the content contract is legalized; Vue slot details remain explicit adapter facts                                  |
| `TargetPolicy::{Dom, Ssr, Vapor}`                                                                              | Explicit Vue target-policy identities; future target policies arrive with real eligibility criteria, not generic aliases                                 |

These are semantic migrations, not mechanical spelling substitutions.
`Input` stability is relative to the component's execution model. A Vue prop
or external context is not constant forever. Preserve every current effect,
escape and proof floor until a native fact proves a more precise contract.
Keep Vue readonly wrappers with unknown verdicts conservative.

## Ordering, lifetime and verification

- Preserve source-bound ids, spans and the artifact's region hierarchy. L2
  node ids and flat-program op ids remain distinct, and dangling references
  remain verifier failures.
- `Built`, `Partitioned` and `Scheduled` remain phases of the same demanded
  artifact, not new pipeline levels. Validate data/effect/dialect edges and
  scope containment before committing a schedule. Invalid cycles are
  diagnostics, not permission to discard edges.
- Framework adapters define effect creation/cleanup and scheduler contracts.
  Preserve conditional/loop ownership and branch transitions. A compiler
  schedule must not claim to prove a runtime scheduler it does not model.
- Hoist/cache/inline/group choices remain an overlay with proof witnesses and
  target eligibility. Emitted cache indices and helper order stay L4's
  responsibility; byte-exact Vue output is still required.
- Kani properties run against production Rust with explicit finite domains.
  Retain differential and existing formal checks until replacement coverage
  exists; bounded proofs do not establish unbounded framework equivalence.

## Implementation acceptance and scope

Implement the module/dump move separately from reactive semantic changes.
Update all kind parsers/printers, operand roles, edge handling, verifiers,
placement/extraction, trace labels, protocol versions and fixtures together.
Internal level wires may break; legacy products keep their output/API policy.
The old L3-to-legacy Vapor adapter remains a dev-only differential oracle,
never a native production path or source of shared facts.

#6839 owns shared decision production and on-demand program construction;
#6844 owns native script facts, #6841 framework feature adapters, #6840 direct
L4 target encoding, and #6859 JSX execution semantics. Future frameworks
remain design-only for Vue Fes. This page closes only #6857 after review and
merge, not any provider, target, product or performance completion issue.

Acceptance for implementation requires original JS/TS and Vue JSX/TSX
fixtures covering reactive reads/writes, readonly/unknown sources, derived
purity, conditional effect lifetime, ordering and re-render/run-once
mismatches. DOM/SSR must prove they do not materialize the unused flat graph.
Exact-head Actions and protected-queue full/differential/instruction gates
must pass without increasing budgets, reparsing or serialization between
levels. Designs and legacy-backed adapters earn no native acceptance credit.
