# The HTML content-model proofs move from Lean to Kani

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record applies
the maintainer's 2026-10-13 decision that Lean proofs become Rust Kani
harnesses to the first of the two Lean packages. It follows the
[retirement ratchet](./2026-10-13-8840-retired-tooling-languages.md).

## Decision

`tests/formal/html-content-model` is deleted.
[`tests/formal/kani/html_content_model.rs`](../../../tests/formal/kani/html_content_model.rs)
replaces it and runs in the `html-content-model` job of the Davinci Kani
workflow on every pull request that touches the kernel.

The Lean package proved its theorems about a restatement of the checker. The
Kani harness includes the shipped source with `#[path]` and verifies that. To
make this possible, the stack-of-open-elements walk moves out of `chain.rs`
into the dependency-free
[`walk.rs`](../../../crates/vize_patina/src/html_content_model/walk.rs).
`Chain::walk` passes it the frames, the verdict below them and each frame's
role per namespace. The checker's verdicts do not change.

## Each Lean theorem and its replacement

| Lean theorem                                     | Kani harness                                                                                                                                             |
| ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Tri.not_sound`, `Tri.and_sound`, `Tri.or_sound` | `kleene_connectives_are_sound`                                                                                                                           |
| `Tri.or_eq_not_and_not`                          | `disjunction_is_the_de_morgan_dual_of_conjunction`                                                                                                       |
| `Tri.admits_unique`                              | `a_definite_fact_admits_exactly_one_value`                                                                                                               |
| `walk_sound`                                     | `one_frame_decides_soundly_under_each_possible_namespace`, `a_frame_without_a_namespace_passes`, `the_walk_is_sound_for_every_stack_the_chain_describes` |
| `walk_stable`                                    | `a_definite_truncated_verdict_survives_revealed_ancestors`                                                                                               |
| `proven_sound`                                   | `a_proven_violation_holds_for_every_dispatch`                                                                                                            |

The `Tri` harnesses are exhaustive, as the Lean case splits were.

## What is bounded

Lean proved `walk_sound` and `walk_stable` by induction for chains of any
length. Kani explores bounded inputs, so the replacement has two parts.

The step is verified without a bound: for a frame with any set of possible
namespaces and any role under each, whatever the frame decides holds under
each namespace it may have, and it passes only when it passes under all of
them. The walk applies exactly this step to one frame after another, so
soundness for longer chains follows from the step.

The whole walk is then verified for every chain of up to four known frames,
over a document or a truncated context that reveals up to two more ancestors.
This covers the loop itself: its order, its early exits and the index it
reports. Stability is verified for the same sizes. Real chains are longer, and
the Rust differential test still checks whole verdicts on 92,196 chains.

`proven_sound` is verified for up to four parent dispatches, which is the most
a frame can produce.

Lean also checked that every definition terminates. The Rust kernel is a
`for` loop over a slice and an iterator over at most three namespaces, and
Kani's unwinding assertions confirm those bounds.

## Checks that the harness is not vacuous

Five mutations of the kernel each fail the expected harnesses: a stop that
decides `Yes`, ignoring a namespace disagreement, walking outermost first,
`and(Maybe, Yes) = Yes` and `join(Yes, Maybe) = Yes`.

## Remaining Lean

`tests/formal/impeto` and the `Davinci Lean` workflow remain until the Impeto
laws and reference differentials are ported.
