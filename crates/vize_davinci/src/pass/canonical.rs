//! The type-level raw → canonical transition.
//!
//! `architecture.md` fixes that "each stage has a raw→canonical transition
//! (type-level) that mandatory passes perform". [`Raw`] and [`Canonical`] are
//! that transition, and the point of making it type-level is that "only a
//! mandatory pass may canonicalize" stops being a review rule.
//!
//! # How the rule is enforced
//!
//! [`Canonical`]'s field is private, so no other module can build one with a
//! struct literal. The only constructor is [`Canonical::produce`], which is
//! generic over the producing pass and forces
//! [`MandatoryCheck::<P>::PROOF`] — an associated `const` whose body asserts
//! `P::DESC.kind.is_mandatory()`. Associated consts are evaluated per
//! monomorphization, so an optional pass calling `produce` is a **compile
//! error**, not a runtime panic and not a lint.
//!
//! The check hangs off a private marker type rather than a `MandatoryPass`
//! trait deliberately: a trait would be one more thing to implement correctly,
//! and an incorrect implementation would only be caught if someone
//! happened to use it. Here there is nothing to implement — every pass that
//! reaches the transition is checked by reaching it.

use core::marker::PhantomData;

use super::Pass;

/// A stage artifact that no mandatory pass has canonicalized yet.
///
/// Optional passes may read and rewrite raw artifacts freely; what they cannot
/// do is declare the result canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Raw<S>(S);

impl<S> Raw<S> {
    /// Wrap a freshly built artifact. Any pass may do this.
    #[inline]
    pub const fn new(artifact: S) -> Self {
        Raw(artifact)
    }

    /// Borrow the artifact.
    #[inline]
    pub const fn get(&self) -> &S {
        &self.0
    }

    /// Borrow the artifact mutably.
    #[inline]
    pub const fn get_mut(&mut self) -> &mut S {
        &mut self.0
    }

    /// Unwrap the artifact, discarding the raw marker.
    #[inline]
    pub fn into_inner(self) -> S {
        self.0
    }
}

/// A stage artifact a mandatory pass has established the stage invariants on.
///
/// The field is private and the only constructor is [`Canonical::produce`];
/// see the module docs for why that is the whole enforcement mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Canonical<S>(S);

impl<S> Canonical<S> {
    /// Declare `raw` canonical, on behalf of the mandatory pass `P`.
    ///
    /// Instantiating this with an [`Optional`](super::PassKind::Optional) pass
    /// fails to compile.
    #[inline]
    #[must_use]
    pub fn produce<P: Pass>(raw: Raw<S>) -> Self {
        let () = MandatoryCheck::<P>::PROOF;
        Canonical(raw.into_inner())
    }

    /// Borrow the artifact.
    #[inline]
    pub const fn get(&self) -> &S {
        &self.0
    }

    /// Unwrap the artifact, discarding the canonical marker.
    #[inline]
    pub fn into_inner(self) -> S {
        self.0
    }

    /// Demote back to raw, for a stage that mutates after canonicalization.
    ///
    /// Deliberately explicit and deliberately cheap to grep for: losing
    /// canonicality is a real event, so it is a named call rather than an
    /// implicit coercion.
    #[inline]
    #[must_use]
    pub fn into_raw(self) -> Raw<S> {
        Raw(self.0)
    }
}

/// Carrier for the per-pass mandatory assertion.
///
/// Private: nothing outside this module should be able to name it, because
/// naming it is not how the check is meant to be reached — calling
/// [`Canonical::produce`] is.
struct MandatoryCheck<P>(PhantomData<P>);

impl<P: Pass> MandatoryCheck<P> {
    /// Evaluated at monomorphization; an optional pass fails to compile here.
    const PROOF: () = assert!(
        P::DESC.kind.is_mandatory(),
        "only a mandatory pass may perform the raw -> canonical transition"
    );
}

#[cfg(test)]
mod tests {
    use super::{Canonical, Raw};
    use crate::pass::{Fusability, Pass, PassDesc, PassKind, Preserved};

    struct Lowerer;
    impl Pass for Lowerer {
        const DESC: PassDesc = PassDesc::new(
            "lower",
            PassKind::MandatoryLowering,
            Fusability::Barrier,
            Preserved::NONE,
        );
    }

    struct Diagnoser;
    impl Pass for Diagnoser {
        const DESC: PassDesc = PassDesc::new(
            "diagnose",
            PassKind::MandatoryDiagnostic,
            Fusability::Barrier,
            Preserved::ALL,
        );
    }

    #[test]
    fn raw_wraps_and_unwraps_without_ceremony() {
        let mut raw = Raw::new(7u32);
        assert_eq!(*raw.get(), 7);
        *raw.get_mut() += 1;
        assert_eq!(raw.into_inner(), 8);
    }

    #[test]
    fn a_mandatory_lowering_pass_can_canonicalize() {
        let canonical = Canonical::produce::<Lowerer>(Raw::new("s2"));
        assert_eq!(*canonical.get(), "s2");
        assert_eq!(canonical.into_inner(), "s2");
    }

    #[test]
    fn a_mandatory_diagnostic_pass_can_canonicalize_too() {
        let canonical = Canonical::produce::<Diagnoser>(Raw::new(1u8));
        assert_eq!(*canonical.get(), 1);
    }

    #[test]
    fn demotion_back_to_raw_is_explicit_and_lossless() {
        let canonical = Canonical::produce::<Lowerer>(Raw::new(41u32));
        let mut raw = canonical.into_raw();
        *raw.get_mut() += 1;
        assert_eq!(raw.into_inner(), 42);
    }

    // The negative case cannot be written as a test, because it is a compile
    // error: `Canonical::produce::<Optimizer>` for an `Optional` pass fails
    // with
    //
    //     error[E0080]: evaluation panicked: only a mandatory pass may perform
    //     the raw -> canonical transition
    //     note: the above error was encountered while instantiating
    //     `fn Canonical::<u8>::produce::<Optimizer>`
    //
    // That was **verified by compiling it**, not assumed (the P0-7 rule), and
    // the exact rustc output is recorded in
    // `davinci-road/plan/phase-2-records/p2-2.md`. It is not a standing
    // regression test because the workspace has no compile-fail harness
    // (no `trybuild`); adding one is a workspace-wide decision, not this
    // task's, and the gap is named here rather than left implicit.
}
