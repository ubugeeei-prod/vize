# Creusot inventory: every Kani harness and its replacement

Companion of [Creusot replaces Kani](./2026-11-17-creusot.md). Each row is a
Kani harness on `main` when the decision was taken: the law it states, the
bound Kani checks it under, and the Creusot proof that replaces it. A row's
status is `proved` only when `cargo creusot --replay` discharges every goal of
the named contract or lemma in
[`davinci-creusot.yml`](../../../.github/workflows/davinci-creusot.yml). It is
`oracle` when a named Rust test covers the law instead, and `owed` until one
of the two lands. It is `kani` when the maintainer kept the Kani harness as
the law's verifier: the incremental update machine, by the
[2026-11-18 decision](./2026-11-17-creusot.md#kani-stays-for-the-incremental-update-machine).
A Kani harness is deleted only when its row is `proved` or `oracle`; Kani
itself stays for the `kani` rows.

## L3 lattice (`davinci/vize_l3/src/lattice`)

Kani file: `lattice/kani.rs` and `lattice/classify_proofs.rs`. Every domain is
finite (4 classes, 8 effects, 4 origins, 4 escapes), and Kani covers each one
whole. Creusot states each law for all values instead of enumerating them.

| Kani harness                                                 | Law                                                                  | Kani bound   | Creusot replacement                                                                                      | Status |
| ------------------------------------------------------------ | -------------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------------------------------------- | ------ |
| `the_value_axis_is_a_strict_chain`                           | `<=` is a total order and the chain `static < … < unstable`          | whole domain | `lattice::the_value_axis_is_a_strict_chain`; derived order: oracle `the_derived_order_is_the_join_order` | proved |
| `join_is_a_least_upper_bound`                                | `join` is commutative, associative, idempotent and the least bound   | whole domain | `lattice::join_is_a_least_upper_bound` over `ReactivityClass::join`                                      | proved |
| `join_has_its_units_and_is_monotone`                         | `static` is the unit, `unstable` absorbs, `join` is monotone         | whole domain | `lattice::join_has_its_units_and_is_monotone` over `ReactivityClass::join`                               | proved |
| `the_effect_floor_is_the_join_of_the_rule_table`             | the tiered floor equals the join of each member's demand             | 256 masks    | owed                                                                                                     | owed   |
| `adding_an_effect_joins_its_floor`                           | `with(e)` joins `e`'s demand                                         | 256 masks    | owed                                                                                                     | owed   |
| `effect_union_preserves_the_class_floor`                     | the floor is a join homomorphism from union                          | 256² masks   | owed                                                                                                     | owed   |
| `a_bound_on_every_effect_bounds_the_set`                     | a bound on every member bounds the set                               | 256 masks    | owed                                                                                                     | owed   |
| `classify_is_the_least_admissible_class`                     | `classify` is the least class every premise admits                   | whole domain | owed                                                                                                     | owed   |
| `the_provide_inject_cap_never_lowers_and_is_already_implied` | the provide/inject cap never lowers the class and is redundant       | whole domain | owed                                                                                                     | owed   |
| `classification_is_monotone`                                 | more effects, a worse origin or a worse escape never lower the class | whole domain | owed                                                                                                     | owed   |
| `classification_reads_effect_membership_only`                | sets with the same members classify alike                            | whole domain | owed                                                                                                     | owed   |
| `only_an_inert_local_binding_is_static`                      | `static` exactly for a local, effect-free, non-escaping binding      | whole domain | owed                                                                                                     | owed   |

## L3 reference scheduling contract (`tests/davinci_test_support/src/l3_reference`)

Kani file: `l3_reference/schedule_proofs.rs`. Bound (from the
[L3 reference record](./2026-11-03-10825-l3-reference-laws-kani.md)): up to 3
ops with ids below 3, 2 region slots, 1 effect slot and 1 edge.

| Kani harness                                    | Law                                                                 | Kani bound          | Creusot replacement | Status |
| ----------------------------------------------- | ------------------------------------------------------------------- | ------------------- | ------------------- | ------ |
| `index_of_is_the_position_of_a_distinct_id`     | `index_of` returns the position of a distinct id                    | 4 ids               | owed                | owed   |
| `no_duplicate_code_means_fresh_distinct_ids`    | no L3V001 means the ids are distinct and fresh                      | 2 seen, 3 ids       | owed                | owed   |
| `the_machine_appends_each_op_in_list_order`     | `run_ops` appends one event per op, in order                        | 2 ops after 1 event | owed                | owed   |
| `trace_positions_are_op_positions`              | trace position `i` is op `i`                                        | 3 ops               | owed                | owed   |
| `an_accepted_scheduled_edge_points_forward`     | a scheduled edge without codes resolves and points forward          | schedule graphs     | owed                | owed   |
| `acceptance_is_an_empty_verdict`                | `accepts` is exactly an empty `order_codes`                         | 2 ops, 1 edge       | owed                | owed   |
| `acceptance_means_distinct_ops_and_clean_edges` | acceptance means distinct op ids and clean edges                    | schedule graphs     | owed                | owed   |
| `accepted_schedule_orders_edges`                | acceptance orders every edge in the trace of both backends          | schedule graphs     | owed                | owed   |
| `accepted_edges_stay_in_scope`                  | accepted effect-scoped edges stay in the effect's region            | schedule graphs     | owed                | owed   |
| `accepted_regrouping_preserves_edges`           | an accepted regrouping is a permutation that keeps every edge order | 3 ops               | owed                | owed   |

## L3 reference C-23 checker

Kani file: `l3_reference/checker_proofs.rs`. Bound: 3 ops, 2 regions, 1 edge,
1 effect, ids below 3. The three invariants that nest a parent walk run once
per table size (`exact_at_sizes!`: 12 `l3v006_*`, 24 `l3v007_*` and 48
`l3v004_*` harnesses).

| Kani harness                                       | Law                                                       | Kani bound          | Creusot replacement | Status |
| -------------------------------------------------- | --------------------------------------------------------- | ------------------- | ------------------- | ------ |
| `the_checker_reports_codes_in_validator_order`     | `checks` lists the codes in validator order               | empty program       | owed                | owed   |
| `l3v001_unique_ids_is_exact`                       | `unique_ids` equals the declarative L3V001                | checker graphs      | owed                | owed   |
| `l3v002_rooted_is_exact`                           | `rooted` equals the declarative L3V002                    | checker graphs      | owed                | owed   |
| `l3v005_resolved_is_exact`                         | `resolved` equals the declarative L3V005                  | checker graphs      | owed                | owed   |
| `l3v003_ops_resolved_is_exact`                     | `ops_resolved` equals the declarative L3V003              | checker graphs      | owed                | owed   |
| `l3v008_ordered_is_exact`                          | `ordered` equals the declarative L3V008                   | checker graphs      | owed                | owed   |
| `l3v006_nested_at_*` (12)                          | `nested` equals the declarative L3V006                    | each size ≤ 3/2     | owed                | owed   |
| `l3v007_scoped_at_*` (24)                          | `scoped` equals the declarative L3V007                    | each size ≤ 3/2/1   | owed                | owed   |
| `l3v004_edges_resolved_at_*` (48)                  | `edges_resolved` equals the declarative L3V004            | each size ≤ 3/2/1/1 | owed                | owed   |
| `a_well_formed_schedule_points_every_edge_forward` | a well-formed scheduled program points every edge forward | checker graphs      | owed                | owed   |

## L3 reference incremental update machine

Kani file: `l3_reference/incremental_proofs.rs`
([#11631](https://github.com/ubugeeei-prod/vize/issues/11631)). Bound: fixed
tree shapes (an empty or text sibling, one retained-or-inserted element, a
retained non-empty child slice, a depth-two retained root, a two-root keyed
reorder), with 3 addresses and tags, both payloads and identities below 8.
Kani stays the verifier of this area: its trees recurse through `Vec`, which
Creusot accepts only with a trusted declaration, and the proofs admit none.

| Kani harness                                           | Law                                                           | Kani bound      | Creusot replacement | Status |
| ------------------------------------------------------ | ------------------------------------------------------------- | --------------- | ------------------- | ------ |
| `retain_keeps_the_addressed_element`                   | a matching sibling keeps its identity, children and allocator | 1 sibling       | none                | kani   |
| `retain_preserves_the_previous_child_slice`            | `retain` returns the stored child slice                       | 1 fixed sibling | none                | kani   |
| `retain_rejects_a_changed_tag`                         | a changed tag at a retained address is rejected               | 1 sibling       | none                | kani   |
| `retain_allocates_one_missing_address`                 | a missing address takes exactly the next identity             | 1 sibling       | none                | kani   |
| `a_text_sibling_preserves_display_and_allocator`       | a text sibling displays itself and allocates nothing          | 1 text          | none                | kani   |
| `top_level_identities_are_retained_or_fresh`           | every identity is retained or fresh                           | 1 element       | none                | kani   |
| `the_patched_tree_displays_the_view`                   | the patched tree displays the requested view                  | depth 2         | none                | kani   |
| `an_update_is_recompute_and_reorders_allocate_nothing` | an update equals recompute; a reorder allocates nothing       | 2 roots         | none                | kani   |
| `allocation_is_exactly_the_inserted_delta`             | allocation is exactly the inserted delta                      | 1 insertion     | none                | kani   |

## HTML content-model kernel (`crates/vize_patina/src/html_content_model`)

Kani file: `html_content_model/kani.rs`. Bound: up to 4 known frames, 2
revealed ancestors, 3 namespaces and 4 dispatches.

| Kani harness                                               | Law                                                                 | Kani bound           | Creusot replacement | Status |
| ---------------------------------------------------------- | ------------------------------------------------------------------- | -------------------- | ------------------- | ------ |
| `kleene_connectives_are_sound`                             | `not`, `and`, `or` are sound for every admitted value               | whole domain         | owed                | owed   |
| `disjunction_is_the_de_morgan_dual_of_conjunction`         | `or` is the De Morgan dual of `and`                                 | whole domain         | owed                | owed   |
| `a_definite_fact_admits_exactly_one_value`                 | a definite fact admits one value                                    | whole domain         | owed                | owed   |
| `one_frame_decides_soundly_under_each_possible_namespace`  | one frame decides soundly under each namespace                      | 3 namespaces         | owed                | owed   |
| `a_frame_without_a_namespace_passes`                       | a frame without a namespace passes                                  | 3 namespaces         | owed                | owed   |
| `the_walk_is_sound_for_every_stack_the_chain_describes`    | the walk's verdict admits the parser's scan of every concrete stack | 4 frames, 2 revealed | owed                | owed   |
| `a_definite_truncated_verdict_survives_revealed_ancestors` | a definite truncated verdict survives revealed ancestors            | 4 frames, 2 revealed | owed                | owed   |
| `a_proven_violation_holds_for_every_dispatch`              | a proven violation holds for every dispatch                         | 4 dispatches         | owed                | owed   |

## Laws no harness states yet

These are added where tractable, after the ports: lossless L1 render equals
source, span monotonicity, and lowering totality. Each gets a row here when
its proof or oracle lands.

## Kernel items outside the proof

Compiled out of the proof crate with `cfg(not(creusot))` because they are not
part of a law and Creusot cannot verify them. Each is covered by the named
Rust test.

| Item                                                            | Why it is outside                      | Rust oracle                                                               |
| --------------------------------------------------------------- | -------------------------------------- | ------------------------------------------------------------------------- |
| `ReactivityClass::as_str`, `ReactivityClass::from_str`          | byte-string matching (`str::as_bytes`) | `vize_l3` `tests/lattice.rs` `reactivity_folio_roundtrips_canonical_text` |
| `ReactivityClass`: derived `Debug`, `PartialOrd`, `Ord`, `Hash` | discriminant intrinsics                | `vize_l3` `tests/lattice.rs` `the_derived_order_is_the_join_order`        |

## Trusted items

No `#[trusted]` item exists in the proof crate or the annotated kernels.
