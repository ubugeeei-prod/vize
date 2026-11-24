//! Creusot proves the laws of the production HTML content-model kernel on
//! its shipped source: the three-valued facts (`tri.rs`) and the
//! stack-of-open-elements walk every scope and loop scan is built from
//! (`walk.rs`). This file is the library root of
//! `tests/formal/creusot/content-model` and is not part of `vize_patina`'s
//! module tree: the modules below are the ones the product compiles, found by
//! ordinary module discovery.
//!
//! The connectives, `classify` and `walk` carry their contracts in the
//! shipped source. Each law below is stated over those contracts and named
//! after the Kani harness it replaces. Unlike the harnesses, it holds without
//! a bound: for chains of every length, frames with any number of possible
//! namespaces, any number of ancestors a truncated context reveals, and any
//! number of dispatches. See docs/davinci/decisions/2026-11-17-creusot.md.

mod tri;
mod walk;

use creusot_std::prelude::*;
use tri::Tri;
use walk::{Role, Step, classify, step_of, walk};

/// A frame as the walk sees it: its role under each namespace it may have,
/// in namespace order, any number of them. `Chain::walk` hands the kernel
/// the same sequence, one role per possible namespace.
pub(crate) struct Possible<'b> {
    roles: &'b [Role],
    next: usize,
}

impl IteratorSpec for Possible<'_> {
    #[logic(open(crate), prophetic)]
    fn completed(&mut self) -> bool {
        pearlite! { resolve(self) && (*self).next@ >= (*self).roles@.len() }
    }

    #[logic(open(crate))]
    fn produces(self, visited: Seq<Role>, o: Self) -> bool {
        pearlite! {
            self.roles == o.roles
                && self.next@ <= o.next@
                && visited.len() == o.next@ - self.next@
                && (self == o || o.next@ <= self.roles@.len())
                && forall<i: Int> 0 <= i && i < visited.len()
                    ==> visited[i] == self.roles[self.next@ + i]
        }
    }

    #[logic(law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) {}

    #[logic(law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<Role>, b: Self, bc: Seq<Role>, c: Self) {}
}

impl Iterator for Possible<'_> {
    type Item = Role;

    #[ensures(match result {
        None => self.completed(),
        Some(role) => (*self).produces(Seq::singleton(role), ^self),
    })]
    fn next(&mut self) -> Option<Role> {
        match self.roles.get(self.next) {
            Some(role) => {
                self.next += 1;
                Some(*role)
            }
            None => None,
        }
    }
}

/// The roles of a frame, from its first possible namespace.
#[ensures(result.roles == *frame && result.next@ == 0)]
fn possible<'b>(frame: &&'b [Role]) -> Possible<'b> {
    Possible { roles: frame, next: 0 }
}

/// Whatever step the walk observes for a frame is the step of its possible
/// roles: the model of a frame is deterministic.
#[logic(prophetic)]
#[ensures(forall<frame: &&[Role], step: Step>
    walk::frame_step(possible, frame, step) ==> step == step_of(frame@))]
pub(crate) fn a_frame_steps_by_its_possible_roles() {}

/// Each possible role of a frame agrees with the frame's step.
#[logic]
#[requires(roles.contains(role))]
#[ensures(match step_of(roles) {
    Step::Pass => role == Role::Pass,
    Step::Decided(Tri::Yes) => role == Role::Target,
    Step::Decided(Tri::No) => role == Role::Stop,
    Step::Decided(Tri::Maybe) => true,
})]
pub(crate) fn each_possible_role_agrees_with_the_step(roles: Seq<Role>, role: Role) {}

/// The parser's own scan over the `depth` outermost elements of a concrete
/// stack of open elements: every element has one namespace and so one role,
/// and the innermost element that is the target or a boundary decides
/// (§13.2.4.2). A stack with neither has no target in scope.
#[logic(open(crate))]
#[variant(depth)]
pub(crate) fn scan(stack: Seq<Role>, depth: Int) -> bool {
    pearlite! {
        if depth <= 0 {
            false
        } else {
            match stack[depth - 1] {
                Role::Target => true,
                Role::Stop => false,
                Role::Pass => scan(stack, depth - 1),
            }
        }
    }
}

/// Passing elements between `at` and `depth` leave the scan to the elements
/// below `at`.
#[logic]
#[variant(depth - at)]
#[requires(0 <= at && at <= depth && depth <= stack.len())]
#[requires(forall<i: Int> at <= i && i < depth ==> stack[i] == Role::Pass)]
#[ensures(scan(stack, depth) == scan(stack, at))]
pub(crate) fn passing_elements_leave_the_scan_below(stack: Seq<Role>, at: Int, depth: Int) {
    pearlite! {
        if at < depth {
            passing_elements_leave_the_scan_below(stack, at, depth - 1)
        }
    }
}

/// `chosen` gives each known frame of the chain `frames` one of its
/// possible roles.
#[logic(open(crate))]
pub(crate) fn describes(frames: Seq<&[Role]>, chosen: Seq<Role>) -> bool {
    pearlite! {
        frames.len() == chosen.len()
            && forall<i: Int> 0 <= i && i < frames.len() ==> frames[i]@.contains(chosen[i])
    }
}

/// The innermost of the `depth` outermost frames that does not pass, or -1.
#[logic(open(crate))]
#[variant(depth)]
pub(crate) fn decider(frames: Seq<&[Role]>, depth: Int) -> Int {
    pearlite! {
        if depth <= 0 {
            -1
        } else if step_of(frames[depth - 1]@) != Step::Pass {
            depth - 1
        } else {
            decider(frames, depth - 1)
        }
    }
}

/// Every frame above the decider passes, and the decider does not.
#[logic]
#[variant(depth)]
#[requires(0 <= depth && depth <= frames.len())]
#[ensures(-1 <= decider(frames, depth) && decider(frames, depth) < depth)]
#[ensures(forall<j: Int> decider(frames, depth) < j && j < depth
    ==> step_of(frames[j]@) == Step::Pass)]
#[ensures(decider(frames, depth) >= 0
    ==> step_of(frames[decider(frames, depth)]@) != Step::Pass)]
pub(crate) fn the_decider_is_the_innermost_frame_that_does_not_pass(
    frames: Seq<&[Role]>,
    depth: Int,
) {
    pearlite! {
        if depth > 0 {
            the_decider_is_the_innermost_frame_that_does_not_pass(frames, depth - 1)
        }
    }
}

/// Revealed ancestors below the known frames do not move a decider found
/// among them.
#[logic]
#[variant(depth)]
#[requires(0 <= depth && depth <= known.len() && known.len() <= composed.len())]
#[requires(forall<i: Int> 0 <= i && i < known.len()
    ==> composed[composed.len() - known.len() + i] == known[i])]
#[ensures(decider(known, depth) >= 0 ==> decider(composed, composed.len() - known.len() + depth)
    == decider(known, depth) + composed.len() - known.len())]
pub(crate) fn revealed_ancestors_keep_the_decider(
    known: Seq<&[Role]>,
    composed: Seq<&[Role]>,
    depth: Int,
) {
    pearlite! {
        if depth > 0 {
            revealed_ancestors_keep_the_decider(known, composed, depth - 1)
        }
    }
}

/// Lean `Tri.not_sound`, `Tri.and_sound` and `Tri.or_sound` (Kani
/// `kleene_connectives_are_sound`): each connective admits the concrete
/// result for every pair of values its operands admit.
#[ensures(forall<a: bool, b: bool> left.admits(a) && right.admits(b)
    ==> result.0.admits(!a) && result.1.admits(a && b) && result.2.admits(a || b))]
pub(crate) fn kleene_connectives_are_sound(left: Tri, right: Tri) -> (Tri, Tri, Tri) {
    (left.not(), left.and(right), left.or(right))
}

/// Lean `Tri.or_eq_not_and_not` (Kani
/// `disjunction_is_the_de_morgan_dual_of_conjunction`).
#[ensures(result.0 == result.1)]
pub(crate) fn disjunction_is_the_de_morgan_dual_of_conjunction(left: Tri, right: Tri) -> (Tri, Tri) {
    (left.or(right), left.not().and(right.not()).not())
}

/// Lean `Tri.admits_unique` (Kani `a_definite_fact_admits_exactly_one_value`).
#[logic]
#[requires(fact != Tri::Maybe && fact.admits(a) && fact.admits(b))]
#[ensures(a == b)]
pub(crate) fn a_definite_fact_admits_exactly_one_value(fact: Tri, a: bool, b: bool) {}

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
pub(crate) fn a_proven_violation_holds_for_every_dispatch(
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

/// The inductive step of Lean `walk_sound` (Kani
/// `one_frame_decides_soundly_under_each_possible_namespace`), for a frame
/// with any number of possible namespaces and any role under each: whatever
/// the frame decides holds under each namespace it may have, and it passes
/// only when it passes under each of them.
#[ensures(result == step_of(frame@))]
#[ensures(forall<i: Int> 0 <= i && i < frame@.len() ==> match result {
    Step::Decided(Tri::Yes) => frame[i] == Role::Target,
    Step::Decided(Tri::No) => frame[i] == Role::Stop,
    Step::Decided(Tri::Maybe) => true,
    Step::Pass => frame[i] == Role::Pass,
})]
pub(crate) fn one_frame_decides_soundly_under_each_possible_namespace(frame: &[Role]) -> Step {
    let step = classify(possible(&frame));
    proof_assert!({ a_frame_steps_by_its_possible_roles(); true });
    step
}

/// A frame without a namespace passes, as the Lean `walk` skips it (Kani
/// `a_frame_without_a_namespace_passes`).
#[requires(frame@.len() == 0)]
#[ensures(result == Step::Pass)]
pub(crate) fn a_frame_without_a_namespace_passes(frame: &[Role]) -> Step {
    let step = classify(possible(&frame));
    proof_assert!({ a_frame_steps_by_its_possible_roles(); true });
    step
}

/// Lean `walk_sound` (Kani `the_walk_is_sound_for_every_stack_the_chain_describes`),
/// for chains of every length: the verdict admits the parser's scan over
/// every concrete stack the chain describes, and a proven scan names the
/// frame that proves it, and only then. A concrete stack gives each known
/// frame one of its possible roles (`chosen`); below a truncated chain it
/// has any ancestors at all (`revealed`), below a document none.
#[requires(describes(frames@, *chosen))]
#[requires(truncated || revealed.len() == 0)]
#[ensures(result.0.admits(scan(revealed.concat(*chosen), revealed.len() + chosen.len())))]
#[ensures((result.1 == None) == (result.0 != Tri::Yes))]
#[ensures(forall<index: usize> result.1 == Some(index)
    ==> index@ < frames@.len() && step_of(frames[index@]@) == Step::Decided(Tri::Yes))]
pub(crate) fn the_walk_is_sound_for_every_stack_the_chain_describes(
    frames: &[&[Role]],
    truncated: bool,
    revealed: Snapshot<Seq<Role>>,
    chosen: Snapshot<Seq<Role>>,
) -> (Tri, Option<usize>) {
    let exhausted = if truncated { Tri::Maybe } else { Tri::No };
    let verdict = walk(frames, exhausted, possible);
    let stop = snapshot!(decider(frames@, frames@.len()));
    let stack = snapshot!(revealed.concat(*chosen));
    let below = snapshot!(revealed.len());
    proof_assert!({ a_frame_steps_by_its_possible_roles(); true });
    proof_assert!({ the_decider_is_the_innermost_frame_that_does_not_pass(frames@, frames@.len()); true });
    proof_assert!(stack.len() == *below + chosen.len() && chosen.len() == frames@.len());
    proof_assert!(forall<j: Int> 0 <= j && j < chosen.len() ==> stack[*below + j] == chosen[j]);
    proof_assert!(*stop == -1 ==> verdict == (exhausted, None));
    proof_assert!(*stop >= 0 ==> Step::Decided(verdict.0) == step_of(frames[*stop]@));
    proof_assert!(forall<j: Int> *stop < j && j < frames@.len() ==> step_of(frames[j]@) == Step::Pass);
    proof_assert!(forall<j: Int> *stop < j && j < frames@.len() ==> {
        each_possible_role_agrees_with_the_step(frames[j]@, chosen[j]);
        chosen[j] == Role::Pass
    });
    proof_assert!(forall<i: Int> *below + *stop < i && i < stack.len() ==> stack[i] == Role::Pass);
    proof_assert!({ passing_elements_leave_the_scan_below(*stack, *below + *stop + 1, stack.len()); true });
    proof_assert!(*stop >= 0 ==> {
        each_possible_role_agrees_with_the_step(frames[*stop]@, chosen[*stop]);
        true
    });
    proof_assert!(verdict.0 == Tri::Yes ==> *stop >= 0 && stack[*below + *stop] == Role::Target);
    proof_assert!(verdict.0 == Tri::No && *stop >= 0 ==> stack[*below + *stop] == Role::Stop);
    proof_assert!(verdict.0 == Tri::No && *stop == -1 ==> *below == 0);
    proof_assert!(verdict.0 != Tri::Maybe ==> scan(*stack, *below + *stop + 1) == (verdict.0 == Tri::Yes));
    verdict
}

/// Lean `walk_stable` (Kani
/// `a_definite_truncated_verdict_survives_revealed_ancestors`): a definite
/// verdict on a truncated chain survives any number of ancestors the context
/// later reveals, whatever lies below them. Frames are outermost first, so
/// the revealed ancestors go in front.
#[requires(known@.len() <= composed@.len())]
#[requires(forall<i: Int> 0 <= i && i < known@.len()
    ==> composed[composed@.len() - known@.len() + i] == known[i])]
#[ensures(result.0.0 != Tri::Maybe ==> result.1.0 == result.0.0 && match (result.0.1, result.1.1) {
    (None, None) => true,
    (Some(before), Some(after)) => after@ == before@ + composed@.len() - known@.len(),
    _ => false,
})]
pub(crate) fn a_definite_truncated_verdict_survives_revealed_ancestors(
    known: &[&[Role]],
    composed: &[&[Role]],
    below: Tri,
) -> ((Tri, Option<usize>), (Tri, Option<usize>)) {
    let before = walk(known, Tri::Maybe, possible);
    let after = walk(composed, below, possible);
    proof_assert!({ a_frame_steps_by_its_possible_roles(); true });
    proof_assert!({ the_decider_is_the_innermost_frame_that_does_not_pass(known@, known@.len()); true });
    proof_assert!({ the_decider_is_the_innermost_frame_that_does_not_pass(composed@, composed@.len()); true });
    proof_assert!({ revealed_ancestors_keep_the_decider(known@, composed@, known@.len()); true });
    let first = snapshot!(decider(known@, known@.len()));
    let second = snapshot!(decider(composed@, composed@.len()));
    let shift = snapshot!(composed@.len() - known@.len());
    proof_assert!(*first == -1 ==> before == (Tri::Maybe, None));
    proof_assert!(*first >= 0 ==> Step::Decided(before.0) == step_of(known[*first]@));
    proof_assert!(*first >= 0 ==> match before.1 {
        Some(index) => index@ == *first && before.0 == Tri::Yes,
        None => before.0 != Tri::Yes,
    });
    proof_assert!(*second >= 0 ==> Step::Decided(after.0) == step_of(composed[*second]@));
    proof_assert!(*second >= 0 ==> match after.1 {
        Some(index) => index@ == *second && after.0 == Tri::Yes,
        None => after.0 != Tri::Yes,
    });
    proof_assert!(before.0 != Tri::Maybe ==> *first >= 0 && *second == *first + *shift);
    proof_assert!(before.0 != Tri::Maybe ==> composed[*second] == known[*first]);
    (before, after)
}
