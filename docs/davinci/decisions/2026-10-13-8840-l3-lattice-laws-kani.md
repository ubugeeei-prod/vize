# The L3 lattice laws move from Lean to Kani

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record applies
the maintainer's 2026-10-13 decision that Lean proofs become Rust Kani
harnesses to `Impeto.LatticeLaws`, the largest theorem module of the Impeto
package (42 of its 66 named laws). It follows the
[HTML content-model move](./2026-10-13-8840-html-content-model-kani.md).

## Decision

`tests/formal/impeto/Impeto/LatticeLaws.lean` is deleted.
[`tests/formal/kani/l3_lattice.rs`](../../../tests/formal/kani/l3_lattice.rs)
grows from two harnesses to twelve and verifies the shipped evaluator.

The Lean laws were proved about `Impeto.Lattice`, a model written from the
rule table. The Kani harness keeps that independence: it states the rule
table itself, the floor each origin, effect and escape demands, and checks the
Rust evaluator against it.

To let Kani include the evaluator, `BindingOrigin` and `EscapeKind` move
unchanged from `lattice/input.rs` to the dependency-free
[`lattice/axis.rs`](../../../davinci/vize_l3/src/lattice/axis.rs), which also
gains `classify`. `evaluate_binding` calls it. Classes do not change.

## Each Lean theorem and its replacement

| Lean theorems                                                                                                                      | Replacement                                                                |
| ---------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `Class.le_refl`, `le_trans`, `le_antisymm`, `le_total`, `static_le`, `le_unstable`, `chain`                                        | `the_value_axis_is_a_strict_chain`                                         |
| `Class.join_comm`, `join_assoc`, `join_idem`, `le_join_left`, `le_join_right`, `join_le`                                           | `join_is_a_least_upper_bound`                                              |
| `Class.static_join`, `join_static`, `unstable_join`, `join_unstable`, `join_le_iff`, `join_eq_right_iff`, `rank_join`, `join_mono` | `join_has_its_units_and_is_monotone`                                       |
| `effectsFloor_eq_tier`                                                                                                             | `the_effect_floor_is_the_join_of_the_rule_table`                           |
| `effectsFloor_cons`, `floor_le_effectsFloor`                                                                                       | `adding_an_effect_joins_its_floor`                                         |
| `effectsFloor_append`, `effectsFloor_mono`                                                                                         | `effect_union_preserves_the_class_floor`                                   |
| `effectsFloor_le`                                                                                                                  | `a_bound_on_every_effect_bounds_the_set`                                   |
| `classify_admissible`, `classify_least`, `classify_spec`                                                                           | `classify_is_the_least_admissible_class`                                   |
| `classify_eq`, `base_le_classify`, `provideInject_capped`, `provideInject_cap_redundant`                                           | `the_provide_inject_cap_never_lowers_and_is_already_implied`               |
| `classify_monotone`, `classify_add_effect`, `classify_escape_demotes`                                                              | `classification_is_monotone`                                               |
| `classify_effects_extensional`                                                                                                     | `classification_reads_effect_membership_only`                              |
| `classify_static_iff`                                                                                                              | `only_an_inert_local_binding_is_static`                                    |
| `classify_verdict_orthogonal`, `fires_iff`, `fires_only_proven`                                                                    | Rust test `the_verdict_never_changes_the_class_and_only_proven_facts_fire` |

## What is not Kani

The three verdict laws read `Verdict`, which lives in `vize_l0`, so the
dependency-free kernel Kani includes does not contain it. They are checked by
an exhaustive Rust test in `davinci/vize_l3/tests/lattice.rs` over every
binding summary there is: 4 origins, 256 effect sets, 4 escapes and 3
verdicts. `classify` takes no verdict, so orthogonality also holds by its
signature.

Nothing is bounded. Every domain is finite, and the Kani harnesses cover all
of it, as the Lean case splits did.

## Checks that the harness is not vacuous

Six mutations of the kernel each fail the expected harnesses: `capture` in
the props-stable tier, a `prop` origin that is `static`, a `classify` that
ignores escape, a `join` that takes the minimum, a `stored` escape that is
props-stable and a provide/inject cap that joins `unstable`.

## Remaining Lean

`Impeto.Lattice` stays as the independent classifier of the Rust fact-page
differential (`--check-lattice-fixtures`). The package still builds and its
audit still passes: 251 theorems remain, above the required 200.
`ScheduleLaws`, `IncrementalLaws`, `CheckerLaws` and the reference interpreter
with its fixture differentials are the remaining ports.
