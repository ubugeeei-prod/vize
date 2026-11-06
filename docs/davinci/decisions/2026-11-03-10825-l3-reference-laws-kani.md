# The L3 schedule and checker laws move from Lean to Kani

[#10825](https://github.com/ubugeeei-prod/vize/issues/10825) applies the
maintainer's 2026-10-13 decision that Lean proofs become Rust Kani harnesses
to two of the three theorem modules left in `tests/formal/impeto` after the
[lattice move](./2026-10-13-8840-l3-lattice-laws-kani.md):
`Impeto.ScheduleLaws` (11 theorems) and `Impeto.CheckerLaws` (4).
`Impeto.IncrementalLaws` (9) stays in Lean for now; see the last section.

## Decision

The laws are proved about an independent Rust reference,
`tests/davinci_test_support/src/l3_reference`, behind the test-only
`l3-reference` feature of `davinci_test_support`. Like the Lean package, it
reads only committed pages and never calls `vize_l3`: the crate does not
depend on it. The scheduling contract, the trace semantics and the C-23 graph
checker are dependency-free modules, so
[`l3_reference/kani.rs`](../../../tests/davinci_test_support/src/l3_reference/kani.rs)
includes them directly and the `l3-reference` job of
`.github/workflows/davinci-kani.yml` verifies them on every change to them.

Rust differentials replace the Lean ones. `cargo test -p vize_l2_to_l3
--test l3_reference_fixture` recomputes `schedule-contract.txt` from the
committed graph pages with the reference contract, next to the TS-27
validator that writes it (`--check-schedule-fixtures`). It also runs the C-23
checker over all 51 committed graphs and its seven damage witnesses
(`--check-folios`).

## Each Lean theorem and its replacement

| Lean theorems                                       | Kani harnesses                                                                          |
| --------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `getElem?_of_indexOf?`, `indexOf?_of_getElem?`      | `index_of_is_the_position_of_a_distinct_id`                                             |
| `nodup_of_duplicateCodes`                           | `no_duplicate_code_means_fresh_distinct_ids`                                            |
| `runOps_eq`, `interpret_opId`                       | `the_machine_appends_each_op_in_list_order`                                             |
| `run_opId`                                          | `trace_positions_are_op_positions`                                                      |
| `edgeCodes_nil`                                     | `an_accepted_scheduled_edge_points_forward`                                             |
| `orderCodes_nil`                                    | `acceptance_is_an_empty_verdict`, `acceptance_means_distinct_ops_and_clean_edges`       |
| `accepted_schedule_orders_edges`                    | `accepted_schedule_orders_edges`                                                        |
| `accepted_edges_stay_in_scope`                      | `accepted_edges_stay_in_scope`                                                          |
| `accepted_regrouping_preserves_edges`               | `accepted_regrouping_preserves_edges`                                                   |
| `code_nil`, `violations_nil_iff`, `violation_sound` | `l3v00N_*_is_exact` (one per invariant), `the_checker_reports_codes_in_validator_order` |
| `wellFormed_ordered`                                | `a_well_formed_schedule_points_every_edge_forward`                                      |

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

The kernels are written for CBMC. They avoid vectors grown, cloned or
collected at a symbolic length, references returned from table scans, a
machine struct or `&mut` vector threaded through a loop, and slice `contains`.
Each of these costs an order of magnitude in time or memory.

## What is not Kani yet

The proof-escape audit (`sorry`, `admit`, `axiom`, `native_decide`) has no
counterpart: Kani has no proof holes or postulates to audit.

`Impeto.IncrementalLaws` is owed. A Rust update machine (generic over strings
and payloads, so harnesses can use a 3-letter alphabet) and its harnesses
exist, outside this change. But the smallest of them, `retain_ok`, ran past
900 seconds at 2.9 GB, and that was with previous sibling lists of at most 2
nodes, each a text or an element with up to 2 text children. The tree laws at
depth 2 passed 4 GB in under 2 minutes. CBMC cannot bound recursive,
vector-backed trees of symbolic shape at this size. Until a tree encoding that
CBMC bounds exists, the 9 theorems keep running in the Lean lane, which stays
until the last change of #10825 deletes it.
