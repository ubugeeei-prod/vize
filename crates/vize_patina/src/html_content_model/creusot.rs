//! Creusot proves the laws of the production HTML content-model facts on
//! their shipped source (`tri.rs`). This file is the library root of
//! `tests/formal/creusot/content-model` and is not part of `vize_patina`'s
//! module tree, as `kani.rs` beside it is not: the module below is the one
//! the product compiles, found by ordinary module discovery.
//!
//! The connectives carry their contracts in the shipped source. Each law
//! below is stated over those contracts and named after the Kani harness it
//! replaces. The verdict rule holds for any number of dispatches, where the
//! harness stopped at four. The stack-of-open-elements walk (`walk.rs`)
//! stays on Kani; see docs/davinci/decisions/2026-11-17-creusot-inventory.md.

mod tri;

use creusot_std::prelude::*;
use tri::Tri;

/// Lean `Tri.not_sound`, `Tri.and_sound` and `Tri.or_sound` (Kani
/// `kleene_connectives_are_sound`): each connective admits the concrete
/// result for every pair of values its operands admit.
#[ensures(forall<a: bool, b: bool> left.admits(a) && right.admits(b)
    ==> result.0.admits(!a) && result.1.admits(a && b) && result.2.admits(a || b))]
pub fn kleene_connectives_are_sound(left: Tri, right: Tri) -> (Tri, Tri, Tri) {
    (left.not(), left.and(right), left.or(right))
}

/// Lean `Tri.or_eq_not_and_not` (Kani
/// `disjunction_is_the_de_morgan_dual_of_conjunction`).
#[ensures(result.0 == result.1)]
pub fn disjunction_is_the_de_morgan_dual_of_conjunction(left: Tri, right: Tri) -> (Tri, Tri) {
    (left.or(right), left.not().and(right.not()).not())
}

/// Lean `Tri.admits_unique` (Kani `a_definite_fact_admits_exactly_one_value`).
#[logic]
#[requires(fact != Tri::Maybe && fact.admits(a) && fact.admits(b))]
#[ensures(a == b)]
pub fn a_definite_fact_admits_exactly_one_value(fact: Tri, a: bool, b: bool) {}

/// Lean `proven_sound` (Kani `a_proven_violation_holds_for_every_dispatch`):
/// a violation is proven when every possible parent dispatch diverges, and
/// then it holds for whichever dispatch the page really takes. The checker
/// folds the per-dispatch facts with `Tri::join` (`check.rs`), as here, for
/// any number of dispatches.
#[requires(concrete.len() == diverges@.len())]
#[requires(forall<i: Int> 0 <= i && i < diverges@.len() ==> diverges[i].admits(concrete[i]))]
#[ensures(result == (diverges@.len() > 0
    && forall<i: Int> 0 <= i && i < diverges@.len() ==> diverges[i] == Tri::Yes))]
#[ensures(result ==> forall<i: Int> 0 <= i && i < concrete.len() ==> concrete[i])]
pub fn a_proven_violation_holds_for_every_dispatch(
    diverges: &[Tri],
    concrete: Snapshot<Seq<bool>>,
) -> bool {
    let mut outcome: Option<Tri> = None;
    let mut index = 0;
    #[invariant(index@ <= diverges@.len())]
    #[invariant(match outcome {
        None => index@ == 0,
        Some(so_far) => index@ > 0 && (so_far == Tri::Yes)
            == (forall<i: Int> 0 <= i && i < index@ ==> diverges[i] == Tri::Yes),
    })]
    #[variant(diverges@.len() - index@)]
    while index < diverges.len() {
        let tri = diverges[index];
        outcome = Some(match outcome {
            None => tri,
            Some(so_far) => so_far.join(tri),
        });
        index += 1;
    }
    let proven = match outcome {
        None => Tri::No,
        Some(tri) => tri,
    };
    proven == Tri::Yes
}
