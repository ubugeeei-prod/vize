//! [`VerifyObserver`] — the between-pass integration, and the cfg shape
//! that keeps it out of release builds.
//!
//! # Runs between passes, through the P2-3 observer
//!
//! The observer follows the `FolioObserver` precedent exactly: the
//! `PassObserver` hooks do not carry the artifact, so the checks are
//! methods called **by whoever has it** — from a step body beside the
//! pipeline, or after a run — while the trait impl keeps the rigor
//! bookkeeping composable through `Pair` and driven by `run_pipeline`'s
//! `after_pass`. Call [`VerifyObserver::check`] after each pass, or at
//! [`PassEvent::is_group_exit`] minimum: a fused group is one walk, and
//! mid-group the artifact is mid-walk (`prior-art-toolchains.md` § GHC,
//! "after each fused group minimum").
//!
//! # Rigor per `PassKind`
//!
//! [`Rigor::Raw`] until a [`PassKind::MandatoryLowering`] pass completes —
//! the kind that canonicalizes — then [`Rigor::Canonical`] for the rest of
//! the run. `MandatoryDiagnostic` and `Optional` passes never change the
//! rigor: they must preserve whatever state the artifact is in.
//!
//! # Verification never ships (guardrail 5)
//!
//! Every check body and the rigor field exist only under
//! `debug_assertions`, the same cfg the arena-generation stamp uses
//! (`crates/vize_carton/src/allocator/generation.rs`). In release the
//! struct is a ZST with empty bodies — const-asserted below — so a release
//! pipeline makes **zero** verifier calls, and by the P2-3 zero-cost
//! measurement (static dispatch; empty bodies inline away; the
//! `davinci_pipeline_no_observer` bench is alloc-identical to the
//! unobserved pipeline) the empty shape costs nothing to carry.
//!
//! # Failure is a panic naming the pass
//!
//! A violated invariant is a compiler bug, not a user diagnostic, so a
//! failed check panics with the aggregated page-order report headed by the
//! offending pass — `` S2 verifier: {n} violation(s) after
//! `{stage}.{pass}` `` — the debug/CI behaviour `-dcore-lint` imports.
//! [`VerifyObserver::check_live`] delegates to
//! `Allocator::assert_stamp_current`, whose own P1-11 message names the
//! outlived-compile contract; it stays that mechanism's panic on purpose.

use vize_davinci::pass::{PassEvent, PassObserver};
use vize_davinci::side_table::SideTable;
use vize_s0::{Allocator, ArenaStamp};

use crate::folio::DisegnoFolio;

#[cfg(debug_assertions)]
use super::{Rigor, Violation, verify, verify_table};
#[cfg(debug_assertions)]
use vize_davinci::pass::PassKind;
#[cfg(debug_assertions)]
use vize_s0::{String, cstr};

/// Runs the S2 invariant checks between passes, debug/CI builds only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyObserver {
    /// Which invariant set applies; escalated by [`VerifyObserver::note`].
    #[cfg(debug_assertions)]
    rigor: Rigor,
}

/// The cfg shape, asserted: in release the observer is a ZST whose check
/// bodies are empty, so no verifier call exists to make (guardrail 5).
#[cfg(debug_assertions)]
const _: () = assert!(size_of::<VerifyObserver>() == 1);
#[cfg(not(debug_assertions))]
const _: () = assert!(size_of::<VerifyObserver>() == 0);

impl VerifyObserver {
    /// A verifier at [`Rigor::Raw`], for the start of a pipeline.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            rigor: Rigor::Raw,
        }
    }

    /// The rigor the passes seen so far establish.
    #[cfg(debug_assertions)]
    #[must_use]
    pub const fn rigor(&self) -> Rigor {
        self.rigor
    }

    /// Record that `event`'s pass completed, escalating the rigor if it
    /// was a canonicalizing pass.
    ///
    /// This is the `after_pass` hook as a callable method, because a step
    /// body holding the artifact cannot reach an observer `run_pipeline`
    /// has mutably borrowed: a caller wiring verification into the step
    /// calls `note` then [`VerifyObserver::check`] itself.
    pub fn note(&mut self, event: &PassEvent<'_>) {
        #[cfg(debug_assertions)]
        if event.desc().kind == PassKind::MandatoryLowering {
            self.rigor = Rigor::Canonical;
        }
        #[cfg(not(debug_assertions))]
        let _ = event;
    }

    /// Check the tree-shaped invariants of `folio` at the current rigor.
    ///
    /// # Panics
    ///
    /// In debug builds, panics with the aggregated page-order report
    /// naming `event`'s pass when any invariant is violated. Empty in
    /// release builds.
    pub fn check(&self, event: &PassEvent<'_>, folio: &DisegnoFolio) {
        #[cfg(debug_assertions)]
        {
            fail(event, &verify(folio, self.rigor));
        }
        #[cfg(not(debug_assertions))]
        let _ = (event, folio);
    }

    /// Check that every id `table` references resolves in `folio`.
    ///
    /// One call per side table: tables are typed, and which tables exist
    /// is the caller's knowledge.
    ///
    /// # Panics
    ///
    /// In debug builds, panics with the aggregated report naming
    /// `event`'s pass when a reference dangles. Empty in release builds.
    pub fn check_table<T>(
        &self,
        event: &PassEvent<'_>,
        folio: &DisegnoFolio,
        table: &SideTable<T>,
    ) {
        #[cfg(debug_assertions)]
        {
            fail(event, &verify_table(folio, table));
        }
        #[cfg(not(debug_assertions))]
        let _ = (event, folio, table);
    }

    /// Check that the arena tree behind the artifact is still live: its
    /// stamp must name `allocator`'s current generation.
    ///
    /// This reuses the P1-11 mechanism rather than inventing one, so the
    /// failure is `assert_stamp_current`'s own panic ("arena-backed value
    /// outlived its compile"). One stamp covers the whole artifact today
    /// because `ExprSlot` is zero-sized; **this method is the P2-5b
    /// seam** — when `ExprRef` gives expression positions identity, the
    /// walk validates each position's stamp here, attributed to `event`'s
    /// pass, and the signature already carries everything that needs.
    ///
    /// # Panics
    ///
    /// In debug builds, panics via `Allocator::assert_stamp_current` when
    /// `stamp` predates a reset. Empty in release builds (the stamp
    /// itself is also empty there).
    pub fn check_live(&self, event: &PassEvent<'_>, allocator: &Allocator, stamp: ArenaStamp) {
        #[cfg(debug_assertions)]
        {
            let _ = event;
            allocator.assert_stamp_current(stamp);
        }
        #[cfg(not(debug_assertions))]
        let _ = (event, allocator, stamp);
    }
}

impl Default for VerifyObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl PassObserver for VerifyObserver {
    fn after_pass(&mut self, event: &PassEvent<'_>) {
        self.note(event);
    }
}

/// Panic with the aggregated report when `violations` is non-empty.
#[cfg(debug_assertions)]
fn fail(event: &PassEvent<'_>, violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }
    let report = report(event, violations);
    panic!("{report}");
}

/// The aggregated failure report: a header naming the pass, then one line
/// per violation in page order.
#[cfg(debug_assertions)]
fn report(event: &PassEvent<'_>, violations: &[Violation]) -> String {
    use core::fmt::Write;

    let mut text = cstr!(
        "S2 verifier: {} violation(s) after `{}.{}`",
        violations.len(),
        event.pipeline.stage,
        event.desc().name
    );
    for violation in violations {
        // Writing into a CompactString is infallible.
        let _ = write!(text, "\n{violation}");
    }
    text
}
