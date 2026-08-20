//! The budget-counting observer: the walk counter the traversal gate reads.
//!
//! `budgets.toml [traversal]` gates walks per fixture, and
//! `davinci-road/plan/walk-baseline.md` records what the pre-S2 pipeline
//! spends. P2-12a measured that with a temporary probe hard-wired into the
//! atelier stages; this observer is what replaces it once passes run through
//! the manager, and it counts the same quantity by construction rather than by
//! matching definitions across two implementations.
//!
//! **One walk per fusion group.** [`BudgetObserver::walks`] increments at
//! [`PassEvent::is_group_entry`](super::PassEvent::is_group_entry), never per
//! pass, which is the whole reason the event carries its group.

use crate::folio::Folio;

use super::{AnalysisEvent, FailEvent, PassEvent, PassObserver, Pipeline};

/// Counts what the traversal and pass budgets gate.
///
/// Carries a derived folio page (`[budget-observer]`, P2-4), so a run's
/// counts print and parse under the TS-16 round-trip laws like any other
/// stage artifact - which is what lets `davinci-opt` treat the budget dump
/// as a stage (`--stage budget-observer`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Folio)]
pub struct BudgetObserver {
    /// Traversals of the tree: one per fusion group entered.
    pub walks: u32,
    /// Passes run, fused or not.
    pub passes: u32,
    /// Analyses computed.
    pub analyses: u32,
    /// Pipelines started.
    pub pipelines: u32,
    /// Pipelines that stopped on a failure.
    pub failures: u32,
}

impl BudgetObserver {
    /// A zeroed counter set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            walks: 0,
            passes: 0,
            analyses: 0,
            pipelines: 0,
            failures: 0,
        }
    }

    /// Reset every counter, for reuse across files.
    pub const fn reset(&mut self) {
        *self = Self::new();
    }

    /// How many passes shared a walk on average, in hundredths, or `None` when
    /// nothing was walked.
    ///
    /// Reported in hundredths rather than as a float because this number ends
    /// up in gates and committed baselines, and a float there is a
    /// reproducibility problem for the sake of two decimal places.
    #[must_use]
    pub const fn fusion_ratio_hundredths(&self) -> Option<u32> {
        if self.walks == 0 {
            return None;
        }
        Some((self.passes * 100) / self.walks)
    }
}

impl PassObserver for BudgetObserver {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {
        self.pipelines += 1;
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        // One walk per group, counted where the walk opens. Counting per pass
        // here is exactly the lie the group-reporting law exists to prevent.
        if event.is_group_entry() {
            self.walks += 1;
        }
        self.passes += 1;
    }

    fn before_analysis(&mut self, _event: &AnalysisEvent<'_>) {
        self.analyses += 1;
    }

    fn on_fail(&mut self, _event: &FailEvent<'_>) {
        self.failures += 1;
    }
}
