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

Kani file: `lattice/kani.rs` and `lattice/classify_proofs.rs`, both deleted
once every row below was proved. Every domain is finite (4 classes, 8 effects,
4 origins, 4 escapes), and Kani covered each one whole. Creusot states each law
for all values instead of enumerating them: the lemmas live in the proof root
`lattice/creusot.rs`, over the contracts of the shipped `class.rs`, `effect.rs`
and `axis.rs`. An `EffectSet` is observed through membership (`has`, a bit of
its `u16`) only; the set operations, `class_floor` and `classify` state their
results in it. The set operations are proved in Creusot's bitwise mode
(`#[bitwise_proof]`), where `u16` `|`, `&` and `<<` are Why3 bitvector
operations; outside that mode they are uninterpreted. A type invariant keeps
every bit outside the eight offsets clear, which `is_empty` relies on.

| Kani harness                                                 | Law                                                                  | Kani bound   | Creusot replacement                                                                                      | Status |
| ------------------------------------------------------------ | -------------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------------------------------------- | ------ |
| `the_value_axis_is_a_strict_chain`                           | `<=` is a total order and the chain `static < … < unstable`          | whole domain | `lattice::the_value_axis_is_a_strict_chain`; derived order: oracle `the_derived_order_is_the_join_order` | proved |
| `join_is_a_least_upper_bound`                                | `join` is commutative, associative, idempotent and the least bound   | whole domain | `lattice::join_is_a_least_upper_bound` over `ReactivityClass::join`                                      | proved |
| `join_has_its_units_and_is_monotone`                         | `static` is the unit, `unstable` absorbs, `join` is monotone         | whole domain | `lattice::join_has_its_units_and_is_monotone` over `ReactivityClass::join`                               | proved |
| `the_effect_floor_is_the_join_of_the_rule_table`             | the tiered floor equals the join of each member's demand             | 256 masks    | `lattice::the_effect_floor_is_the_join_of_the_rule_table` over `EffectSet::class_floor`                  | proved |
| `adding_an_effect_joins_its_floor`                           | `with(e)` joins `e`'s demand                                         | 256 masks    | `lattice::adding_an_effect_joins_its_floor` over `EffectSet::one`, `with`                                | proved |
| `effect_union_preserves_the_class_floor`                     | the floor is a join homomorphism from union                          | 256² masks   | `lattice::effect_union_preserves_the_class_floor` over `EffectSet::union`                                | proved |
| `a_bound_on_every_effect_bounds_the_set`                     | a bound on every member bounds the set                               | 256 masks    | `lattice::a_bound_on_every_effect_bounds_the_set`                                                        | proved |
| `classify_is_the_least_admissible_class`                     | `classify` is the least class every premise admits                   | whole domain | `lattice::classify_is_the_least_admissible_class` over `classify`                                        | proved |
| `the_provide_inject_cap_never_lowers_and_is_already_implied` | the provide/inject cap never lowers the class and is redundant       | whole domain | `lattice::the_provide_inject_cap_never_lowers_and_is_already_implied`                                    | proved |
| `classification_is_monotone`                                 | more effects, a worse origin or a worse escape never lower the class | whole domain | `lattice::classification_is_monotone`                                                                    | proved |
| `classification_reads_effect_membership_only`                | sets with the same members classify alike                            | whole domain | `lattice::classification_reads_effect_membership_only`                                                   | proved |
| `only_an_inert_local_binding_is_static`                      | `static` exactly for a local, effect-free, non-escaping binding      | whole domain | `lattice::only_an_inert_local_binding_is_static`                                                         | proved |

## L3 reference scheduling contract (`tests/davinci_test_support/src/l3_reference`)

Kani file: `l3_reference/schedule_proofs.rs`, deleted. Bound it had (from the
[L3 reference record](./2026-11-03-10825-l3-reference-laws-kani.md)): up to 3
ops with ids below 3, 2 region slots, 1 effect slot and 1 edge. The Creusot
proofs (package `tests/formal/creusot/reference`, root
`l3_reference/creusot.rs`) hold for any number of ops, regions, effects and
edges, and any ids. Contracts are on the shipped functions of `program.rs`,
`schedule.rs` and `trace.rs`; lemmas are in `creusot.rs`.

| Kani harness                                    | Law                                                                 | Kani bound          | Creusot replacement                                                                                     | Status |
| ----------------------------------------------- | ------------------------------------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------- | ------ |
| `index_of_is_the_position_of_a_distinct_id`     | `index_of` returns the position of a distinct id                    | 4 ids               | contract of `program::index_of` (first position; for distinct ids, the id's position)                   | proved |
| `no_duplicate_code_means_fresh_distinct_ids`    | no L3V001 means the ids are distinct and fresh                      | 2 seen, 3 ids       | contract of `schedule::duplicates` (zero exactly for distinct, fresh ids)                               | proved |
| `the_machine_appends_each_op_in_list_order`     | `run_ops` appends one event per op, in order                        | 2 ops after 1 event | contracts of `trace::run_ops` and `trace::interpret` (`observe`)                                        | proved |
| `trace_positions_are_op_positions`              | trace position `i` is op `i`                                        | 3 ops               | contract of `trace::run`                                                                                | proved |
| `an_accepted_scheduled_edge_points_forward`     | a scheduled edge without codes resolves and points forward          | schedule graphs     | `reference::an_accepted_scheduled_edge_points_forward` over the contract of `schedule::edge_codes`      | proved |
| `acceptance_is_an_empty_verdict`                | `accepts` is exactly an empty `order_codes`                         | 2 ops, 1 edge       | contracts of `schedule::accepts` and `schedule::order_codes`: both are `accepted`                       | proved |
| `acceptance_means_distinct_ops_and_clean_edges` | acceptance means distinct op ids and clean edges                    | schedule graphs     | contract of `schedule::accepts`: `accepted` is distinct op ids and every edge `clean` (no `edge_codes`) | proved |
| `accepted_schedule_orders_edges`                | acceptance orders every edge in the trace of both backends          | schedule graphs     | `reference::accepted_schedule_orders_edges` over the contracts of `trace::run` and `schedule::precedes` | proved |
| `accepted_edges_stay_in_scope`                  | accepted effect-scoped edges stay in the effect's region            | schedule graphs     | `reference::accepted_edges_stay_in_scope` over the contract of `schedule::op_inside`                    | proved |
| `accepted_regrouping_preserves_edges`           | an accepted regrouping is a permutation that keeps every edge order | 3 ops               | `reference::accepted_regrouping_preserves_edges`, any permutation of any number of ops                  | proved |

## L3 reference C-23 checker

Kani file: `l3_reference/checker_proofs.rs`, deleted. Bound it had: 3 ops, 2
regions, 1 edge, 1 effect, ids below 3, with the three invariants that nest a
parent walk run once per table size (`exact_at_sizes!`: 12 `l3v006_*`, 24
`l3v007_*` and 48 `l3v004_*` harnesses). Each checker function in
`checker.rs` now has the declarative invariant (Lean `Impeto.Checker`) as its
contract, for programs of any size.

| Kani harness                                       | Law                                                       | Kani bound          | Creusot replacement                                                                     | Status |
| -------------------------------------------------- | --------------------------------------------------------- | ------------------- | --------------------------------------------------------------------------------------- | ------ |
| `the_checker_reports_codes_in_validator_order`     | `checks` lists the codes in validator order               | empty program       | contract of `checker::checks`; `checker::violations` is empty exactly for `well_formed` | proved |
| `l3v001_unique_ids_is_exact`                       | `unique_ids` equals the declarative L3V001                | checker graphs      | contract of `checker::unique_ids` (`ids_unique`)                                        | proved |
| `l3v002_rooted_is_exact`                           | `rooted` equals the declarative L3V002                    | checker graphs      | contract of `checker::rooted` (`has_root`)                                              | proved |
| `l3v005_resolved_is_exact`                         | `resolved` equals the declarative L3V005                  | checker graphs      | contract of `checker::resolved` (`parents_resolve`)                                     | proved |
| `l3v003_ops_resolved_is_exact`                     | `ops_resolved` equals the declarative L3V003              | checker graphs      | contract of `checker::ops_resolved` (`ops_live`)                                        | proved |
| `l3v008_ordered_is_exact`                          | `ordered` equals the declarative L3V008                   | checker graphs      | contract of `checker::ordered` (`edges_forward`)                                        | proved |
| `l3v006_nested_at_*` (12)                          | `nested` equals the declarative L3V006                    | each size ≤ 3/2     | contract of `checker::nested` (`spans_nest`, with `chain_ends` for `reaches_top`)       | proved |
| `l3v007_scoped_at_*` (24)                          | `scoped` equals the declarative L3V007                    | each size ≤ 3/2/1   | contract of `checker::scoped` (`effects_scoped`: `effects_named` and `effects_placed`)  | proved |
| `l3v004_edges_resolved_at_*` (48)                  | `edges_resolved` equals the declarative L3V004            | each size ≤ 3/2/1/1 | contract of `checker::edges_resolved` (`edges_stay`)                                    | proved |
| `a_well_formed_schedule_points_every_edge_forward` | a well-formed scheduled program points every edge forward | checker graphs      | `reference::a_well_formed_schedule_points_every_edge_forward` over `checker::forward`   | proved |

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

Kani file, deleted with its last harness: `html_content_model/kani.rs`. Its
bound was up to 4 known frames, 2 revealed ancestors, 3 namespaces and 4
dispatches. Creusot root: `html_content_model/creusot.rs`. `Tri::not`, `and`,
`or` and `join` carry their soundness and Kleene truth-table contracts in
`tri.rs`. `classify` and `walk` carry theirs in `walk.rs`, for any iterator of
roles and any closure that yields one. The laws instantiate them with a frame
modelled as its sequence of possible roles, any number of them, so they hold
for chains of every length, under any number of revealed ancestors and for
any number of dispatches.

| Kani harness                                               | Law                                                                 | Kani bound           | Creusot replacement                                                                                      | Status |
| ---------------------------------------------------------- | ------------------------------------------------------------------- | -------------------- | -------------------------------------------------------------------------------------------------------- | ------ |
| `kleene_connectives_are_sound`                             | `not`, `and`, `or` are sound for every admitted value               | whole domain         | `content_model::kleene_connectives_are_sound` over `Tri::not`, `and`, `or`                               | proved |
| `disjunction_is_the_de_morgan_dual_of_conjunction`         | `or` is the De Morgan dual of `and`                                 | whole domain         | `content_model::disjunction_is_the_de_morgan_dual_of_conjunction` over `Tri::not`, `and`, `or`           | proved |
| `a_definite_fact_admits_exactly_one_value`                 | a definite fact admits one value                                    | whole domain         | `content_model::a_definite_fact_admits_exactly_one_value` over `Tri::admits`                             | proved |
| `one_frame_decides_soundly_under_each_possible_namespace`  | one frame decides soundly under each namespace                      | 3 namespaces         | `content_model::one_frame_decides_soundly_under_each_possible_namespace` over `classify`, any namespaces | proved |
| `a_frame_without_a_namespace_passes`                       | a frame without a namespace passes                                  | 3 namespaces         | `content_model::a_frame_without_a_namespace_passes` over `classify`                                      | proved |
| `the_walk_is_sound_for_every_stack_the_chain_describes`    | the walk's verdict admits the parser's scan of every concrete stack | 4 frames, 2 revealed | `content_model::the_walk_is_sound_for_every_stack_the_chain_describes` over `walk`, every length         | proved |
| `a_definite_truncated_verdict_survives_revealed_ancestors` | a definite truncated verdict survives revealed ancestors            | 4 frames, 2 revealed | `content_model::a_definite_truncated_verdict_survives_revealed_ancestors` over `walk`, every length      | proved |
| `a_proven_violation_holds_for_every_dispatch`              | a proven violation holds for every dispatch                         | 4 dispatches         | `content_model::a_proven_violation_holds_for_every_dispatch` over `Tri::join`, any number of dispatches  | proved |

Creusot 0.14 has no spec for `Iterator::any` (or the `try_fold` it folds
with), nor for a back-to-front `Enumerate`. So `classify` reads each role once
in a single `next` loop that stops at the first disagreement, and `walk` reads
frames with `next_back`. The maintainer waived IR identity for that rewrite on
2026-11-24 in exchange for no regression on any metric. The pull request that
made it carries the instruction counts, the A/B/A-A lint timings and the
unchanged corpus and differential outputs.

In an optimized build, a `next` loop over the old `Filter` on a namespace
array executes about a third more instructions per scan than `any` did. So
`NsSet::iter` now yields the set's bits directly, lowest first, and the
rewritten scans cost no more than before. The namespace order is still HTML,
SVG, MathML, so every verdict is unchanged. The proofs
show that one element read backwards (`Seq::reverse`) is itself by the
`compute_in_goal` tactic, which `why3find.json` now lists.

## JS string escape gate (`crates/vize_atelier_core/src/codegen/helpers`)

No Kani harness stated this law. `escape_js_string` returns its input
unchanged when no byte of it is flagged by `byte_may_need_js_escaping`, and
that fast path is sound only if every scalar the slow stages rewrite puts a
flagged byte into the UTF-8 text. Creusot root: `helpers/creusot.rs`. The
gate's contract is in `helpers/escape_gate.rs`, the module it moved into so
the proof crate compiles it alone. The rewritten set comes from the slow path
and Unicode, not from the gate: `&` (`decode_html_entities`), `\`, `"` and the
general category `Cc` (U+0000..=U+001F, U+007F..=U+009F); the first UTF-8
byte follows Unicode Table 3-6.

| Kani harness | Law                                                                                                       | Kani bound | Creusot replacement                                                                                  | Status |
| ------------ | --------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------- | ------ |
| none         | every scalar that `decode_html_entities` or `escape_js_string` rewrites starts with a byte the gate flags | —          | `js_escape_gate::no_rewritten_scalar_passes_the_gate` over `byte_may_need_js_escaping`, every scalar | proved |
| none         | the first byte of every scalar's UTF-8 encoding is below 0x80 or in 0xC2..=0xF4                           | —          | `js_escape_gate::every_scalar_starts_with_a_lead_byte`                                               | proved |

## L0 spans and source ranges (`davinci/vize_l0/src`)

Kani file: `span/proofs.rs`, deleted, and the four range and mapping
harnesses of `source_range/proofs.rs`; `source_map_to_generated_*` stays
there under Kani (see
[Laws no harness states yet](#laws-no-harness-states-yet)). Each harness had no
loop, so Kani checked every `u32` and `i32` value. Creusot states each law for
every value instead: the functions in the proof root `creusot.rs` (package
`tests/formal/creusot/coordinates`) call the shipped `span.rs` and
`source_range.rs` and assert the law over their contracts. The contracts
restate the documentation: a span or range is the half-open set of offsets
`[start, end)` (`covers`); `is_empty` covers no offset; `intersects` shares
one; `union` is the least range whose endpoints enclose both;
`intersection` covers exactly the offsets in both, and is `None` when no
offset is in both; `offset` and `extend` move each endpoint in the integers
and clamp it into `u32` (`shifted`); a mapping carries an offset by its
distance from the range start, clamped to the last offset of the other range
(`carried`). `offset_u32` spells `amount.unsigned_abs()` as
`amount.abs_diff(0)`, which Creusot specifies; the LLVM IR is unchanged.

| Kani harness                                              | Law                                                                                                                                                 | Kani bound   | Creusot replacement                                                                                                     | Status |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | ----------------------------------------------------------------------------------------------------------------------- | ------ |
| `span_length_counts_the_offsets_it_covers`                | `Span::len` is the number of offsets in `[start, end)`, zero exactly when the span is empty                                                         | whole domain | `span_length_counts_the_offsets_it_covers` over the `len` and `is_empty` contracts                                      | proved |
| `span_block_relative_form_ignores_where_the_block_starts` | the block-relative span never grows, inverts the shift inside the block and is invariant under moving span and block                                | whole domain | `span_block_relative_form_never_grows_and_inverts_the_shift`, `span_block_relative_form_ignores_where_the_block_starts` | proved |
| `source_range_union_is_the_least_enclosing_range`         | `SourceRange::union` is commutative, associative, idempotent and the least range enclosing both                                                     | whole domain | `source_range_union_is_the_least_enclosing_range` over the `union` contract                                             | proved |
| `source_range_intersection_is_the_offsets_in_both`        | `intersection` holds exactly the offsets in both, is `Some` exactly when they `intersects`, and is a semilattice meet                               | whole domain | `source_range_intersection_is_the_offsets_in_both`, with `a_nonempty_range_is_its_offsets`                              | proved |
| `source_range_shifts_saturate_and_invert`                 | `offset` moves each endpoint by the shift clamped into `u32`; without saturation the opposite shift restores the range; `extend` encloses the range | whole domain | `source_range_shifts_saturate`, `source_range_shifts_invert`                                                            | proved |
| `source_mapping_carries_offsets_between_its_ranges`       | a mapping carries offsets by their distance, clamped into the other range; an equal-length mapping is a bijection                                   | whole domain | `source_mapping_carries_offsets_between_its_ranges` over the `source_to_generated` and `generated_to_source` contracts  | proved |

## Laws no harness states yet

These are added where tractable, after the ports: lossless L1 render equals
source, span monotonicity, and lowering totality. Each gets a row here when
its proof or oracle lands.

The coordinate kernels below are verified by Kani in `davinci-kani.yml`
(`source-coordinates`), each against a spec written from the format's
definition, not from the kernel. Where the bound is "whole domain", the
harness has no loop over its input, or a loop Kani unwinds completely, so
Kani checks every value of the domain and the check is exhaustive. Creusot
does not reach these kernels without changing them: the UTF-16 and
line-terminator kernels go through `str` APIs without a specification
(`as_bytes`, `char_indices`, `is_char_boundary`), the VLQ writer appends to
`CompactString`, and `SourceMap` sorts and searches with `sort_by_key` and
`find_map`, which have no specification either. The span, range and
mapping laws are proved instead; see
[L0 spans and source ranges](#l0-spans-and-source-ranges-davincivize_l0src).

The two `owed` rows are whole-string lookups. Kani harnesses for them
(`LineIndex` over three to five bytes; the emitter's line table, lookup and
segment resolution over three to six bytes) exceeded 8 GB in CBMC and are not
kept: Kani is used here only for small models. They need a Creusot proof over
a byte-slice kernel, or a Kani model of one line step at a time.
`tests/tooling/formal-harness-coverage.test.ts` holds that a workflow runs
every Kani harness in the repository.

| Kani harness                                                        | Law                                                                                                                                                                      | Kani bound                                       | Creusot replacement | Status |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------ | ------------------- | ------ |
| `vlq_digits_decode_to_the_encoded_value`                            | the VLQ writer appends base64 digits only, sets the continuation bit on all but the last, is minimal and decodes back                                                    | whole domain (`i64` except `MIN`)                | none                | kani   |
| `line_break_len_is_the_width_of_the_terminator_at_the_offset`       | the emitter's line step is 2 at CR LF, 1 at LF or a lone CR, 3 at LS or PS, and 0 anywhere else, including inside a scalar or past the end                               | 4 bytes, whole `usize` offset                    | none                | kani   |
| `utf16_offset_is_the_boundary_with_that_many_units`                 | `utf16_offset` returns exactly the scalar boundary with that many UTF-16 units, and `None` for any other column                                                          | 4 bytes, whole `u32` column                      | none                | kani   |
| `utf16_len_counts_the_units_of_every_scalar`                        | `utf16_len` counts two UTF-16 units for a four-byte scalar and one for any other                                                                                         | 4 bytes                                          | none                | kani   |
| `utf16_prefix_counts_the_units_of_whole_scalars`                    | `prefix_len` counts the units of every scalar that starts before the offset, a mid-scalar offset counting that scalar whole, so it never panics and is monotone          | 4 bytes, whole `usize` offset                    | none                | kani   |
| `source_map_to_generated_uses_the_mapping_that_contains_the_offset` | `SourceMap::to_generated` translates through the first mapping in source order whose source range contains the offset; ranges may be empty, inverted or overlap          | any 3 mappings, any insertion order, whole `u32` | none                | kani   |
| `source_map_to_source_shifts_into_the_file_without_overflow`        | `SourceMap::to_source` adds the block offset to the first mapping that holds the generated offset, `None` past `u32::MAX`, and never overflows                           | 2 mappings of any shape, whole `u32` values      | none                | kani   |
| none                                                                | `LineIndex::line_col` is the LF count before the offset and the UTF-16 units from that line's start; `line_col_to_offset` is its inverse on scalar boundaries            | —                                                | owed                | owed   |
| none                                                                | the emitter's line table starts a line after every terminator, and `resolve_in_table` and `resolve_positions` give each segment the line and UTF-16 column of its offset | —                                                | owed                | owed   |

## Kernel items outside the proof

Compiled out of the proof crate with `cfg(not(creusot))` because they are not
part of a law and Creusot cannot verify them. Each is covered by the named
Rust test.

| Item                                                                                    | Why it is outside                          | Rust oracle                                                                                                                                                                                 |
| --------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ReactivityClass::as_str`, `ReactivityClass::from_str`                                  | byte-string matching (`str::as_bytes`)     | `vize_l3` `tests/lattice.rs` `reactivity_folio_roundtrips_canonical_text`                                                                                                                   |
| `ReactivityClass`: derived `Debug`, `PartialOrd`, `Ord`, `Hash`                         | discriminant intrinsics                    | `vize_l3` `tests/lattice.rs` `the_derived_order_is_the_join_order`                                                                                                                          |
| `Tri`: derived `Debug`, `Hash`                                                          | discriminant intrinsics                    | `vize_patina` `html_content_model::tri::tests::truth_tables_are_kleene`                                                                                                                     |
| `Role`, `Step`: derived `Debug`                                                         | discriminant intrinsics                    | none needed: no law, test or diagnostic reads it                                                                                                                                            |
| L3 reference `Phase::parse`, `OpKind::parse`, `EdgeKind::parse`                         | string matching (`str` equality)           | `vize_l2_to_l3` `tests/l3_reference_fixture` `committed_graphs_pass_the_independent_checker`, `schedule_contract_codes_match_the_independent_reference` (every committed graph page parses) |
| L3 reference program and trace types: derived `Debug`, `PartialEq`, `Eq`                | discriminant intrinsics; no law reads them | the same fixture tests compare traces and programs with them                                                                                                                                |
| `EffectKind`, `BindingOrigin`, `EscapeKind`: `as_str`, `from_str`                       | byte-string matching (`str::as_bytes`)     | `vize_l3` `tests/lattice.rs` `every_folio_spelling_roundtrips`                                                                                                                              |
| `EffectSet::print`, `EffectSet::parse`                                                  | `fmt::Write`, `str::split`                 | `vize_l3` `tests/lattice.rs` `every_folio_spelling_roundtrips`                                                                                                                              |
| `EffectKind::bit` in the product (`1 << (self as u8)`; the proof crate's twin matches)  | enum-to-discriminant cast                  | `vize_l3` `tests/lattice.rs` `each_effect_discriminant_is_its_set_offset`                                                                                                                   |
| `EffectKind`, `BindingOrigin`, `EscapeKind`: derived `Debug`, `PartialEq`, `Eq`, `Hash` | discriminant intrinsics                    | `vize_l3` `tests/lattice.rs` `every_folio_spelling_roundtrips`                                                                                                                              |
| `EffectSet`: derived `Debug`, `Default`, `PartialEq`, `Eq`, `Hash`                      | `fmt` and `Hasher` calls                   | `vize_l3` `tests/lattice.rs` `effect_set_derives_observe_membership_only`                                                                                                                   |
| `Span`: derives other than `Clone` and `Copy`; `hash_relative`                          | `Hasher`, `fmt` and `serde` calls          | `vize_l0` `span::tests` `hash_relative_matches_derived_hash_of_rebased_span`                                                                                                                |
| `Span::slice` and its `floor_char_boundary`                                             | `str::is_char_boundary`, `str::get`        | Kani `original_source_span_preserves_the_selected_bytes`; `vize_l0` `span::tests`                                                                                                           |
| `SourceRange`: derives other than `Clone` and `Copy`; `Display`                         | `fmt` and `serde` calls                    | `vize_l0` `tests/source_range_intersections.rs` and `source_range::tests`                                                                                                                   |
| `SourceMapping`, `MappingData`, `SourceMap`: derives                                    | `fmt` calls; no law reads them             | none needed: no law, test or diagnostic reads them                                                                                                                                          |
| `SourceMap` methods                                                                     | `sort_by_key`, `find_map`                  | Kani `source_map_to_generated_uses_the_mapping_that_contains_the_offset`; `test_source_map`                                                                                                 |

## Trusted items

No `#[trusted]` item exists in the proof crate or the annotated kernels.
