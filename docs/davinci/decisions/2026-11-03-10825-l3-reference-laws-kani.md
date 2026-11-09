# The L3 reference laws move from Lean to Kani

[#10825](https://github.com/ubugeeei-prod/vize/issues/10825) applies the
maintainer's 2026-10-13 decision that Lean proofs become Rust Kani harnesses
to the three theorem modules left in `tests/formal/l3` after the
[lattice move](./2026-10-13-8840-l3-lattice-laws-kani.md):
`Impeto.ScheduleLaws` (11 theorems), `Impeto.CheckerLaws` (4), and
`Impeto.IncrementalLaws` (9).

## Decision

The laws are proved about an independent Rust reference,
`tests/level_test_support/src/l3_reference`, behind the test-only
`l3-reference` feature of `level_test_support`. Like the Lean package, it
reads only committed pages and never calls `vize_l3`: the crate does not
depend on it. The scheduling contract, trace semantics, C-23 graph checker,
and incremental update machine are dependency-free modules, so
[`l3_reference/kani.rs`](../../../tests/level_test_support/src/l3_reference/kani.rs)
includes them directly and the `l3-reference` job of
`.github/workflows/level-kani.yml` verifies them on every change to them.

Rust differentials replace the Lean ones. `cargo test -p vize_l2_to_l3
--test l3_reference_fixture` recomputes `schedule-contract.txt` from the
committed graph pages with the reference contract, next to the TS-27
validator that writes it (`--check-schedule-fixtures`). It also runs the C-23
checker over all 51 committed graphs and its seven damage witnesses
(`--check-folios`).

## Each Lean theorem and its replacement

| Lean theorems                                                          | Kani harnesses                                                                                                                                            |
| ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `getElem?_of_indexOf?`, `indexOf?_of_getElem?`                         | `index_of_is_the_position_of_a_distinct_id`                                                                                                               |
| `nodup_of_duplicateCodes`                                              | `no_duplicate_code_means_fresh_distinct_ids`                                                                                                              |
| `runOps_eq`, `interpret_opId`                                          | `the_machine_appends_each_op_in_list_order`                                                                                                               |
| `run_opId`                                                             | `trace_positions_are_op_positions`                                                                                                                        |
| `edgeCodes_nil`                                                        | `an_accepted_scheduled_edge_points_forward`                                                                                                               |
| `orderCodes_nil`                                                       | `acceptance_is_an_empty_verdict`, `acceptance_means_distinct_ops_and_clean_edges`                                                                         |
| `accepted_schedule_orders_edges`                                       | `accepted_schedule_orders_edges`                                                                                                                          |
| `accepted_edges_stay_in_scope`                                         | `accepted_edges_stay_in_scope`                                                                                                                            |
| `accepted_regrouping_preserves_edges`                                  | `accepted_regrouping_preserves_edges`                                                                                                                     |
| `code_nil`, `violations_nil_iff`, `violation_sound`                    | `l3v00N_*_is_exact` (one per invariant), `the_checker_reports_codes_in_validator_order`                                                                   |
| `wellFormed_ordered`                                                   | `a_well_formed_schedule_points_every_edge_forward`                                                                                                        |
| `retain_ok`                                                            | `retain_keeps_the_addressed_element`, `retain_preserves_the_previous_child_slice`, `retain_rejects_a_changed_tag`, `retain_allocates_one_missing_address` |
| `siblings_erase_of`, `reconcile_erase`                                 | `a_text_sibling_preserves_display_and_allocator`, `the_patched_tree_displays_the_view`                                                                    |
| `siblings_allocations_of`, `reconcile_allocations`, `siblings_advance` | `allocation_is_exactly_the_inserted_delta`, `the_patched_tree_displays_the_view`                                                                          |
| `siblings_identity`                                                    | `top_level_identities_are_retained_or_fresh`                                                                                                              |
| `update_tree`, `update_without_insertions`                             | `the_patched_tree_displays_the_view`, `an_update_is_recompute_and_reorders_allocate_nothing`                                                              |

The checker harnesses are stronger than `violations_nil_iff`: every invariant
equals a declarative statement written from `Impeto.Checker`, and the codes
follow validator order, so the code list is exact, not only its emptiness.
The regrouping harness witnesses the permutation of events with the
regrouping's own permutation.

## Bounds

The Lean theorems quantify over unbounded graphs, so each harness states its
bound. The scheduling laws read regions and effects only by id lookup. Their
graphs have up to 3 ops, ids below 3, and 2 region slots and 1 effect slot of
fixed size, where an id of 3 resolves for no lookup, so the fixed tables cover
every shorter one. They have one edge: every scheduling law is a conjunction
over edges, and no edge reads another. The checker graphs have up to 3 ops, 2
regions, 1 edge and 1 effect. The invariants that nest a parent walk inside
quantifiers (L3V004, L3V006, L3V007) are checked once for every table size up
to that bound. Every other domain is finite and covered whole.

The incremental laws use fixed structural representatives because recursive
vector shape is not a useful symbolic domain for CBMC. They cover an empty or
text sibling, one retained-or-inserted element, a retained non-empty child
slice, a depth-two retained root whose child is replaced, and a two-root keyed
reorder. Addresses and tags range over three values where their equality is
the subject of the law, payloads cover both booleans, and identities are below
eight. The depth-two harness calls the shipped `reconcile`; the reorder
harness calls its shipped `retain` primitive twice. The `update_tree` result
then follows compositionally: accepted `update` returns only a `reconcile`
result whose erased tree equals the requested view. Stateful fixture and loop
witnesses execute the shipped `update` wrapper itself.

The kernels are written for CBMC. They avoid vectors grown, cloned or
collected at a symbolic length, references returned from table scans, a
machine struct or `&mut` vector threaded through a loop, and slice `contains`.
Each of these costs an order of magnitude in time or memory.

## Proof escape and the former symbolic-tree attempt

The proof-escape audit (`sorry`, `admit`, `axiom`, `native_decide`) has no
counterpart: Kani has no proof holes or postulates to audit.

The first incremental port made recursive vector shape symbolic. The smallest
`retain_ok` harness ran past 900 seconds at 2.9 GB, and a depth-two tree passed
4 GB in under 2 minutes. Those runs did not establish a bound and are not
proof evidence. The fixed-shape harnesses above retain symbolic semantic
values while removing that unbounded encoding choice; every harness now fits
the Kani lane.
