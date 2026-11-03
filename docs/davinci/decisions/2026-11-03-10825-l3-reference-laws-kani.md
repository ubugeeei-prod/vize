# The L3 schedule, checker and update-machine laws move from Lean to Kani

[#10825](https://github.com/ubugeeei-prod/vize/issues/10825) applies the
maintainer's 2026-10-13 decision that Lean proofs become Rust Kani harnesses
to the three theorem modules left in `tests/formal/impeto` after the
[lattice move](./2026-10-13-8840-l3-lattice-laws-kani.md):
`Impeto.ScheduleLaws` (11 theorems), `Impeto.IncrementalLaws` (9) and
`Impeto.CheckerLaws` (4).

## Decision

The laws are proved about an independent Rust reference,
`tests/davinci_test_support/src/l3_reference`, behind the test-only
`l3-reference` feature of `davinci_test_support`. Like the Lean package, it
reads only committed pages and never calls `vize_l3`: the crate does not
depend on it. The scheduling contract, the trace semantics, the C-23 graph
checker and the TS-29 update machine are dependency-free modules, so
[`l3_reference/kani.rs`](../../../tests/davinci_test_support/src/l3_reference/kani.rs)
includes them directly and the `l3-reference` job of
`.github/workflows/davinci-kani.yml` verifies them on every change to them.

The update machine is generic over its strings and payloads. The reference
instantiates it with the rendered view; the harnesses use a 3-letter alphabet
and a 1-bit payload, since no law reads either.

The Rust differentials replace the Lean ones. `cargo test -p vize_l2_to_l3
--test lean_reference_fixture` recomputes `schedule-contract.txt` from the
committed graph pages with the reference contract, next to the TS-27
validator that writes it, and runs the C-23 checker over every committed
graph with its seven damage witnesses (`--check-schedule-fixtures`,
`--check-folios`).

## Each Lean theorem and its replacement

| Lean theorems                                                          | Kani harness                                           |
| ---------------------------------------------------------------------- | ------------------------------------------------------ |
| `getElem?_of_indexOf?`, `indexOf?_of_getElem?`                         | `index_of_is_the_position_of_a_distinct_id`            |
| `nodup_of_duplicateCodes`                                              | `no_duplicate_code_means_fresh_distinct_ids`           |
| `runOps_eq`, `interpret_opId`, `run_opId`                              | `the_machine_executes_ops_in_list_order`               |
| `edgeCodes_nil`                                                        | `an_accepted_scheduled_edge_points_forward`            |
| `orderCodes_nil`                                                       | `acceptance_means_distinct_ops_and_clean_edges`        |
| `accepted_schedule_orders_edges`                                       | `accepted_schedule_orders_edges`                       |
| `accepted_edges_stay_in_scope`                                         | `accepted_edges_stay_in_scope`                         |
| `accepted_regrouping_preserves_edges`                                  | `accepted_regrouping_preserves_edges`                  |
| `code_nil`, `violations_nil_iff`, `violation_sound`                    | `the_checker_reports_exactly_the_violated_invariants`  |
| `wellFormed_ordered`                                                   | `a_well_formed_schedule_points_every_edge_forward`     |
| `retain_ok`                                                            | `retain_keeps_the_addressed_element_or_allocates`      |
| `siblings_erase_of`, `reconcile_erase`                                 | `the_patched_tree_displays_the_view`                   |
| `siblings_allocations_of`, `reconcile_allocations`, `siblings_advance` | `allocation_is_exactly_the_inserted_delta`             |
| `siblings_identity`                                                    | `top_level_identities_are_retained_or_fresh`           |
| `update_tree`, `update_without_insertions`                             | `an_update_is_recompute_and_reorders_allocate_nothing` |

The checker harness is stronger than `violations_nil_iff`: it requires the
exact code list, one code per violated invariant in validator order, against a
declarative statement of each invariant written from `Impeto.Checker`.

## Bounds

The Lean theorems quantify over unbounded graphs and trees, so each harness
states its bound. Graphs have at most 3 ops, 3 regions, 3 edges and 2
effects, with ids below 4 (so every harness also sees unresolved ids) and
unconstrained spans. Update-machine trees have sibling lists of at most 2
nodes and depth 2, with identities below 8, at most one already-seen address
and fuel up to 3. Every other domain is finite and covered whole.

## What is not Kani

The proof-escape audit (`sorry`, `admit`, `axiom`, `native_decide`) has no
counterpart: Kani has no proof holes or postulates to audit. The Lean modules
stay until the reference interpreter and its fixture differentials are ported;
the package is deleted with `davinci-lean.yml` in the last change of #10825.
