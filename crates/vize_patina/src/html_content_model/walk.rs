//! The stack-of-open-elements walk every scope and loop scan is built from.
//!
//! This kernel depends on nothing but [`Tri`], so Creusot verifies this
//! exact source (`creusot.rs` beside this file): `classify` and `walk` carry
//! their contracts here, and the laws over them are proved for chains of
//! every length. The walk is sound for every concrete stack a possibly
//! truncated chain describes, and a definite verdict on a truncated chain
//! survives any ancestors the context later reveals.

use super::tri::Tri;
#[cfg(creusot)]
use creusot_std::prelude::{Clone, PartialEq, *};

/// How a frame participates in a scan under one namespace.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy, PartialEq, Eq))]
pub(super) enum Role {
    /// The element the scan looks for.
    Target,
    /// A scope boundary, a special element or a marker: the scan ends here.
    Stop,
    /// Neither: the scan continues with the next open element.
    Pass,
}

/// What one frame contributes to a scan.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy, PartialEq, Eq))]
pub(super) enum Step {
    /// The frame decides the scan.
    Decided(Tri),
    /// The scan continues below the frame.
    Pass,
}

/// The roles of one frame, one per possible namespace. Creusot reasons
/// about an iterator through `IteratorSpec`, so the proof crate adds that
/// bound; the product bound is the plain iterator.
#[cfg(not(creusot))]
pub(super) trait Roles: Iterator<Item = Role> {}
#[cfg(not(creusot))]
impl<I: Iterator<Item = Role>> Roles for I {}
#[cfg(creusot)]
pub(super) trait Roles: Iterator<Item = Role> + IteratorSpec {}
#[cfg(creusot)]
impl<I: Iterator<Item = Role> + IteratorSpec> Roles for I {}

/// Roles compare as themselves.
#[cfg(creusot)]
impl DeepModel for Role {
    type DeepModelTy = Role;

    #[logic(open(super))]
    fn deep_model(self) -> Role {
        self
    }
}

/// Steps compare as themselves.
#[cfg(creusot)]
impl DeepModel for Step {
    type DeepModelTy = Step;

    #[logic(open(super))]
    fn deep_model(self) -> Step {
        self
    }
}

/// The step of a frame whose roles under its possible namespaces are
/// `roles`: it passes when it has no possible namespace, it is unknown when
/// two of its namespaces disagree, and otherwise its one role decides.
#[cfg(creusot)]
#[logic(open(super))]
pub(super) fn step_of(roles: Seq<Role>) -> Step {
    pearlite! {
        if roles.len() == 0 {
            Step::Pass
        } else if exists<i: Int> 0 <= i && i < roles.len() && roles[i] != roles[0] {
            Step::Decided(Tri::Maybe)
        } else {
            match roles[0] {
                Role::Target => Step::Decided(Tri::Yes),
                Role::Stop => Step::Decided(Tri::No),
                Role::Pass => Step::Pass,
            }
        }
    }
}

/// One element read from the back is the element itself.
#[cfg(creusot)]
#[logic]
#[ensures(Seq::singleton(value).reverse().len() == 1)]
#[ensures(Seq::singleton(value).reverse()[0] == value)]
pub(super) fn a_single_element_reads_the_same_backwards<T>(value: T) {}

/// `step` is the step of the roles `roles` yields: the roles it yields up to
/// its end, or a prefix of them whose roles already disagree.
#[cfg(creusot)]
#[logic(open(super), prophetic)]
pub(super) fn classifies<I: Roles>(roles: I, step: Step) -> bool {
    pearlite! {
        (exists<seen: Seq<Role>, end: &mut I>
            roles.produces(seen, *end) && end.completed() && step == step_of(seen))
        || (exists<seen: Seq<Role>, rest: I>
            roles.produces(seen, rest) && step == Step::Decided(Tri::Maybe) && step_of(seen) == step)
    }
}

/// `step` is the step of `frame` under `roles`.
#[cfg(creusot)]
#[logic(open(super), prophetic)]
pub(super) fn frame_step<'a, F, I: Roles, R: Fn(&'a F) -> I>(
    roles: R,
    frame: &'a F,
    step: Step,
) -> bool {
    pearlite! { exists<it: I> roles.postcondition((frame,), it) && classifies(it, step) }
}

/// Classify one frame from its role under each namespace it may have.
///
/// A frame with no possible namespace passes. A frame whose role differs
/// across its possible namespaces makes the scan unknown.
#[cfg_attr(creusot, ensures(classifies(roles, result)))]
pub(super) fn classify<I: Roles>(mut roles: I) -> Step {
    #[cfg(creusot)]
    let start = snapshot!(roles);
    #[cfg(creusot)]
    let mut seen = snapshot!(Seq::empty());
    let mut first = None;
    #[cfg_attr(creusot, invariant(start.produces(*seen, roles)))]
    #[cfg_attr(creusot, invariant(match first {
        None => seen.len() == 0,
        Some(known) => seen.len() > 0
            && forall<i: Int> 0 <= i && i < seen.len() ==> seen[i] == known,
    }))]
    #[expect(
        clippy::while_let_on_iterator,
        reason = "the loop invariants name `roles`, which a `for` loop would borrow"
    )]
    while let Some(role) = roles.next() {
        #[cfg(creusot)]
        {
            seen = snapshot!(seen.concat(Seq::singleton(role)));
        }
        match first {
            None => first = Some(role),
            Some(known) if known != role => {
                #[cfg(creusot)]
                proof_assert!(seen[seen.len() - 1] != seen[0]);
                #[cfg(creusot)]
                proof_assert!(start.produces(*seen, roles));
                return Step::Decided(Tri::Maybe);
            }
            Some(_) => {}
        }
    }
    match first {
        None => Step::Pass,
        Some(Role::Target) => Step::Decided(Tri::Yes),
        Some(Role::Stop) => Step::Decided(Tri::No),
        Some(Role::Pass) => Step::Pass,
    }
}

/// The walk over `frames` stops at `stop` (-1 when every frame passes) with
/// `verdict`: every frame above `stop` passes, `stop` decides the verdict,
/// and only a proven verdict names its frame.
#[cfg(creusot)]
#[logic(open(super), prophetic)]
pub(super) fn walked<'a, F, I: Roles, R: Fn(&'a F) -> I>(
    roles: R,
    frames: &'a [F],
    exhausted: Tri,
    stop: Int,
    verdict: (Tri, Option<usize>),
) -> bool {
    pearlite! {
        -1 <= stop && stop < frames@.len()
            && (forall<j: Int> stop < j && j < frames@.len()
                ==> frame_step(roles, &frames[j], Step::Pass))
            && (stop == -1 ==> verdict == (exhausted, None))
            && (stop >= 0 ==> frame_step(roles, &frames[stop], Step::Decided(verdict.0))
                && match verdict.1 {
                    Some(index) => index@ == stop && verdict.0 == Tri::Yes,
                    None => verdict.0 != Tri::Yes,
                })
    }
}

/// Walk `frames` from the innermost (last) outwards. `roles` yields a
/// frame's role under each of its possible namespaces, and `exhausted` is
/// the verdict when every known frame passes.
///
/// Returns the verdict and, when it is `Yes`, the index of the deciding
/// frame.
#[cfg_attr(creusot, requires(forall<frame: &'a F> roles.precondition((frame,))))]
#[cfg_attr(creusot, ensures(exists<stop: Int> walked(roles, frames, exhausted, stop, result)))]
pub(super) fn walk<'a, F, I, R>(frames: &'a [F], exhausted: Tri, roles: R) -> (Tri, Option<usize>)
where
    I: Roles,
    R: Fn(&'a F) -> I,
{
    let mut rest = frames.iter();
    #[cfg(creusot)]
    let mut head = snapshot!(rest);
    #[cfg_attr(creusot, invariant(*head == rest))]
    #[cfg_attr(creusot, invariant(rest@.to_ref_seq().len() <= frames@.len()))]
    #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < rest@.to_ref_seq().len()
        ==> rest@.to_ref_seq()[j] == frames.to_ref_seq()[j]))]
    #[cfg_attr(creusot, invariant(forall<j: Int> rest@.to_ref_seq().len() <= j && j < frames@.len()
        ==> frame_step(roles, &frames[j], Step::Pass)))]
    while let Some(frame) = rest.next_back() {
        let index = rest.len();
        #[cfg(creusot)]
        proof_assert!(head@.to_ref_seq() == rest@.to_ref_seq().concat(Seq::singleton(frame).reverse()));
        #[cfg(creusot)]
        proof_assert!({
            a_single_element_reads_the_same_backwards(frame);
            true
        });
        #[cfg(creusot)]
        proof_assert!(index@ == rest@.to_ref_seq().len());
        #[cfg(creusot)]
        proof_assert!(head@.to_ref_seq().len() == index@ + 1 && head@.to_ref_seq()[index@] == frame);
        #[cfg(creusot)]
        proof_assert!(index@ < frames@.len() && frame == frames.to_ref_seq()[index@]);
        #[cfg(creusot)]
        proof_assert!(frame == &frames[index@]);
        let step = classify(roles(frame));
        #[cfg(creusot)]
        proof_assert!(frame_step(roles, &frames[index@], step));
        match step {
            Step::Decided(Tri::Yes) => {
                #[cfg(creusot)]
                proof_assert!(walked(roles, frames, exhausted, index@, (Tri::Yes, Some(index))));
                return (Tri::Yes, Some(index));
            }
            Step::Decided(verdict) => {
                #[cfg(creusot)]
                proof_assert!(walked(roles, frames, exhausted, index@, (verdict, None)));
                return (verdict, None);
            }
            Step::Pass => {}
        }
        #[cfg(creusot)]
        {
            head = snapshot!(rest);
        }
    }
    #[cfg(creusot)]
    proof_assert!(head@@.len() == 0 && head@.to_ref_seq().len() == 0);
    #[cfg(creusot)]
    proof_assert!(forall<j: Int> -1 < j && j < frames@.len() ==> frame_step(roles, &frames[j], Step::Pass));
    #[cfg(creusot)]
    proof_assert!(walked(roles, frames, exhausted, -1, (exhausted, None)));
    (exhausted, None)
}
