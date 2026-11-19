# L3 owns the shared backend decisions (2026-11-19)

Decision for [#6839](https://github.com/ubugeeei-prod/vize/issues/6839)
(gate `levels.l3`), slice L3-1 of the T2–T6 plan. It records which decisions
move into L2→L3 and `vize_l3::decision`, how they are keyed, and the PR
sequence that moves them. It builds on
[native decision ownership](../plan/native-decision-skeleton.md), the
[native artifact decisions](./2026-10-01-l3-native-artifact-decisions.md), the
[neutral vocabulary](./2026-10-04-l3-neutral-vocabulary.md) and the
[input-neutral levels](./2026-11-17-input-neutral-levels.md). Where this record
is more specific than those, it wins; their ownership split (L3 decides
eligibility, order and grouping; L4 owns helpers, numbering, encoding and
module assembly) stands.

## Present state (worktree census, base `a74306cee7`)

Each backend decides on its own, from different inputs:

| Decision                          | Where it is decided today                                                                                                                  | Input                      |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------- |
| Static partition / hoistability   | L2 pass `hoist-static` (`vize_l2::analysis::static_facts`, 798 LOC; adapter `vize_l1_to_l2/src/pass/hoist.rs`) into `L2Facts.static_facts` | L2 ops                     |
| Whole-subtree hoist               | `vize_l4::targets::dom::artifact::vnode_static::can_whole_hoist_static_element`, re-derived from ops (does not read `static_facts`)        | L2 ops                     |
| Static props / children / caching | `targets/dom/artifact/{static_cache,props_static,vnode_static,component}.rs` reading `static_facts`                                        | `static_facts`             |
| Cache eligibility                 | `targets/dom/artifact/{once,memo,on/cache}.rs`                                                                                             | L2 ops                     |
| Cache numbering                   | `targets/dom/artifact/cache_slots.rs`, `cx.rs`                                                                                             | emission order             |
| Patch flags                       | `targets/dom/artifact/patch.rs` (`PatchFactsTable`, keyed by L2 `NodeId`) plus `vize_l3::lattice`                                          | L2 ops                     |
| Block / fragment shape            | `targets/dom/artifact/{vnode,builtin}.rs`; `vize_l2::analysis::fragment_facts` from lowering                                               | L2 ops                     |
| SSR static/dynamic segments       | `vize_l2_to_l3::partition` (`PartitionFacts`, keyed by `OpId`) → `atelier_ssr/src/l4/string_plan`                                          | flat L3 program            |
| Vapor effect units and order      | `atelier_vapor/src/l3/native/emit.rs` (one effect per op), legacy `generate/effect_order.rs` after the `RootIRNode` round trip             | flat L3 program, legacy IR |
| Hoist / Cache / Group placement   | `vize_l3::placement::annotate`: no consumer outside tests and fuzzing                                                                      | flat L3 program            |

`vize_l3::decision` already builds one `DecisionTables` per target policy,
keyed by L2 `NodeId`, with a conservative `static_level`, `output_level`,
`dynamic_bindings`, `placement` (always `Inline`) and control containment.
Outside `vize_l3`, only `dynamic_bindings` is read.

`vize_l3::placement` and L2 `static_facts` disagree on hoist legality. The
placement Hoist is one boolean, requires a control-region owner and blocks
`key` and `is`; `static_facts` has three static levels, a separate props arm,
namespace/native-descendant flags, roots without a control owner, and blocks
only `ref` and `class` keys. Placement Cache covers `set-event` only; the DOM
emitter also caches `v-once`, `v-memo`, root-hoisted props and elements.

## Decision

1. **One producer, keyed by L2 `NodeId`.** Every shared decision is a column
   of `vize_l3::decision::DecisionTables`, computed by the single
   `build_decisions` walk under a `TargetPolicy`. No decision is keyed by a
   flat-program `OpId`; a target that needs the flat program (Vapor reactive
   execution) maps through the lowering's NodeId↔OpId record. DOM and SSR do
   not build the flat program to read a decision.

2. **What moves into L3.** These become `DecisionTables` columns, replacing
   the backend copies listed above:

   | Column             | Meaning                                                                                                                    | Replaces                                                                              |
   | ------------------ | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
   | `static_level`     | The `static_facts` lattice (not-static / dynamic-text / fully-static) plus props-hoistable, nested-static, namespace flags | L2 `hoist-static` pass; the conservative L3 level                                     |
   | `hoist`            | Hoist eligibility: whole subtree, props only, or none; one legality rule for root and control-owned nodes                  | `vnode_static`, `props_static`, `static_cache` decisions; `placement::Hoist` legality |
   | `cache`            | Cache eligibility per node or binding: handler, `once`, `memo`, hoisted props/element                                      | `once.rs`, `memo.rs`, `on/cache.rs` eligibility; `placement::Cache`                   |
   | `dynamic_bindings` | Already present: dynamic attached bindings in authored order                                                               | —                                                                                     |
   | `partition`        | Static/dynamic segment class for string targets                                                                            | `vize_l2_to_l3::partition` (`OpId` keyed)                                             |
   | `effect`           | Effect unit leader and order for reactive targets                                                                          | Vapor `emit.rs` one-effect-per-op, legacy `effect_order.rs`; `placement::Group`       |

3. **What stays in L4.** Cache slot numbering, hoist names, patch-flag bit
   encoding, block/fragment helper choice, runtime helpers and module
   assembly. L4 reads eligibility; it never re-derives it.

4. **Neutral names.** Columns describe meaning (static, hoist, cache,
   partition, effect), not a Vue runtime spelling. `once` and `memo` are
   cache kinds with an explicit invalidation contract, not Vue directive
   names; a `vapor` dialect appears only where the neutral contract cannot
   represent the behavior exactly (per the neutral vocabulary decision).

5. **Replace, never run beside.** A family moves in one PR that switches the
   reader to the L3 column and deletes the backend's own computation in the
   same diff. The L2 `hoist-static` pass and `L2Facts.static_facts` are
   deleted by the PR that moves the last `static_facts` reader (decision 6 of
   the T2–T6 plan's maintainer list; deletion is the default because keeping
   both runs the analysis twice). `vize_l3::placement::annotate` is either
   rebuilt on the same columns or deleted with its tests; it does not survive
   as a second legality rule.

## Sequence

| Slice | Change                                                                                                                                                            | Deletes                                                            | Rows that should fall                                 |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------- |
| L3-2  | `build_decisions` computes `static_level` and `hoist` with the `static_facts` lattice; a dev-test equivalence check runs it beside `static_facts` over the corpus | nothing (test-only comparison, off the hot path)                   | none; no production row may move                      |
| L3-3a | DOM artifact reads `hoist`/`static_level` from L3                                                                                                                 | `hoist-static` pass, `static_facts` module, `L2Facts.static_facts` | `atelier_dom_compile_*`                               |
| L3-3b | DOM artifact reads `cache`                                                                                                                                        | eligibility in `once.rs`, `memo.rs`, `on/cache.rs`                 | `atelier_dom_compile_*`                               |
| L3-3c | Patch-flag inputs come from `dynamic_bindings` plus `static_level`                                                                                                | the derivation half of `patch.rs`; encoding stays                  | `atelier_dom_compile_*`                               |
| L3-4  | SSR string plan reads `partition` from the decision tables                                                                                                        | `vize_l2_to_l3::partition`                                         | `atelier_ssr_compile_*`                               |
| L3-5  | Vapor reads `effect` (with L4-5)                                                                                                                                  | `effect_order.rs`, the per-op effect split, `placement::Group`     | `atelier_vapor_compile_*`, `atelier_vapor_generate_*` |
| L3-6  | Dump/op-kind renames to the neutral names                                                                                                                         | old spellings                                                      | none                                                  |

Every slice keeps DOM/SSR/Vapor output byte-identical on the repository
corpus, the Vue differential corpus and the reach floors
(`docs/davinci/plan/reach-budgets.toml`), and every instruction row at or
below its base.

## Coordination

`vize_l3/src/decision` is also being edited for the input-neutral layout
(`decision/*` placement entries in `docs/davinci/plan/input-placement-owed.json`).
Slices L3-2 onward rebase onto those PRs and add columns without moving files.

## Not decided here

- The `static_facts` lattice is copied semantically, not structurally; whether
  `StaticLevel` gains the props/namespace flags or a second column carries
  them is an implementation choice for L3-2.
- Whether `vize_l3::placement` is rebuilt on the columns or deleted is decided
  in L3-5, once Vapor shows whether it needs alternatives beyond the chosen
  shape.
