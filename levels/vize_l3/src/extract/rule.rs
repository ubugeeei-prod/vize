//! The pinned multi-metric commit rule.
//!
//! A candidate commits only when no metric regresses beyond its tier epsilon
//! and at least `required_improvements_min` metrics strictly improve. Ties
//! resolve by the tier's tie policy, which keeps the simpler current shape.
//! Constraints are judged before the objective, so a rejection names the
//! first constraint it broke.

use super::measure::{Metric, Metrics};
use super::tier::{OptimizationBudget, TiePolicy};

/// Why the rule refused a measured candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rejection {
    /// The metric grew beyond its epsilon.
    Regressed(Metric),
    /// Fewer metrics improved than the tier requires, or none did.
    NoImprovement,
}

/// The tier epsilon for `metric`, in percent.
#[must_use]
pub const fn epsilon_pct(budget: &OptimizationBudget, metric: Metric) -> u32 {
    match metric {
        Metric::EmittedSize => budget.emitted_size_epsilon_pct,
        Metric::ReactiveEdge => budget.reactive_edge_epsilon_pct,
        Metric::UpdatePath => budget.update_path_epsilon_pct,
    }
}

/// Judge `after` against the current plan's `before` under `budget`.
pub fn judge(
    before: Metrics,
    after: Metrics,
    budget: &OptimizationBudget,
) -> Result<(), Rejection> {
    let mut improved = 0u32;
    for metric in Metric::CHECK_ORDER {
        let (was, now) = (before.get(metric), after.get(metric));
        let allowed = was + was * u64::from(epsilon_pct(budget, metric)) / 100;
        if now > allowed {
            return Err(Rejection::Regressed(metric));
        }
        improved += u32::from(now < was);
    }
    let tie = improved == 0 && matches!(budget.tie_policy, TiePolicy::Reject);
    if tie || improved < budget.required_improvements_min {
        return Err(Rejection::NoImprovement);
    }
    Ok(())
}
