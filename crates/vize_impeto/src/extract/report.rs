//! Extraction decisions and their structured remarks (P3-13).
//!
//! Every candidate yields exactly one [`Decision`], and [`Extraction::remark`]
//! turns each into one remark named after the placement (`hoist`, `cache`,
//! `group`): `applied` when committed, `missed` otherwise, with the args
//! `reason`, `emitted-size`, `reactive-edges`, `update-path`, `budget-left`.
//! The vocabulary is registered in `docs/davinci/plan/remarks-format.md`.

use alloc::vec::Vec;
use core::fmt;

use vize_davinci::pass::{Remark, RemarkArg, RemarkSink};
use vize_s0::Span;

use super::measure::{Metric, Metrics};
use super::tier::OptTier;
use crate::op::OpId;
use crate::placement::Placement;

/// The pass name decisions and remarks carry.
pub const EXTRACT_PASS: &str = "extract-placements";

/// Whether the candidate's placement was committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionKind {
    Applied,
    Missed,
}

impl DecisionKind {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Missed => "missed",
        }
    }
}

/// Why a candidate was applied or missed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    /// Measured and accepted by the rule.
    Committed,
    /// Measured; the metric grew beyond its epsilon.
    Regressed(Metric),
    /// Measured; too few metrics improved.
    NoImprovement,
    /// Not measured: the component's candidate budget was spent.
    BudgetExhausted,
    /// Not measured: the committed plan breaks the group's contiguity.
    NotContiguous,
    /// Not measured: an enclosing committed hoist already moves this op.
    Subsumed,
}

impl Reason {
    /// Every reason, for parsing.
    pub const ALL: [Self; 8] = [
        Self::Committed,
        Self::Regressed(Metric::ReactiveEdge),
        Self::Regressed(Metric::UpdatePath),
        Self::Regressed(Metric::EmittedSize),
        Self::NoImprovement,
        Self::BudgetExhausted,
        Self::NotContiguous,
        Self::Subsumed,
    ];

    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Regressed(Metric::ReactiveEdge) => "regressed-reactive-edge",
            Self::Regressed(Metric::UpdatePath) => "regressed-update-path",
            Self::Regressed(Metric::EmittedSize) => "regressed-emitted-size",
            Self::NoImprovement => "no-improvement",
            Self::BudgetExhausted => "budget-exhausted",
            Self::NotContiguous => "not-contiguous",
            Self::Subsumed => "subsumed",
        }
    }

    /// Parse the stable spelling.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|reason| reason.as_str() == text)
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Signed metric change one trial measured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Delta {
    pub emitted_size: i64,
    pub reactive_edges: i64,
    pub update_path: i64,
}

impl Delta {
    /// `after - before`, metric by metric.
    #[must_use]
    pub fn between(before: Metrics, after: Metrics) -> Self {
        let diff = |metric| after.get(metric) as i64 - before.get(metric) as i64;
        Self {
            emitted_size: diff(Metric::EmittedSize),
            reactive_edges: diff(Metric::ReactiveEdge),
            update_path: diff(Metric::UpdatePath),
        }
    }
}

/// One candidate's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub op: OpId,
    pub span: Span,
    pub placement: Placement,
    pub kind: DecisionKind,
    pub reason: Reason,
    /// Zero when the candidate was not measured.
    pub delta: Delta,
    /// Candidates the component may still try after this one.
    pub budget_left: u32,
}

/// The result of one extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extraction {
    pub tier: OptTier,
    pub candidate_budget: u32,
    pub budget_left: u32,
    /// The canonical all-inline plan.
    pub before: Metrics,
    /// The committed plan.
    pub after: Metrics,
    /// One decision per candidate, in record order.
    pub decisions: Vec<Decision>,
}

impl Extraction {
    /// Emit one structured remark per decision. Argument construction is
    /// guarded on the sink, so a detached run builds nothing.
    pub fn remark<S: RemarkSink>(&self, sink: &mut S) {
        if !S::ENABLED {
            return;
        }
        for decision in &self.decisions {
            let args = [
                RemarkArg::str("reason", decision.reason.as_str()),
                RemarkArg::int("emitted-size", decision.delta.emitted_size),
                RemarkArg::int("reactive-edges", decision.delta.reactive_edges),
                RemarkArg::int("update-path", decision.delta.update_path),
                RemarkArg::int("budget-left", i64::from(decision.budget_left)),
            ];
            let name = decision.placement.as_str();
            let remark = match decision.kind {
                DecisionKind::Applied => Remark::applied(name, decision.span, &args),
                DecisionKind::Missed => Remark::missed(name, decision.span, &args),
            };
            sink.emit(&remark);
        }
    }
}
