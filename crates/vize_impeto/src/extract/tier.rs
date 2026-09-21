//! `-O` tiers as budget constants.
//!
//! Every tier runs the same pass; only the budget and the metric tolerances
//! scale (the Flambda2 import). The table mirrors `[optimization]` in
//! `davinci-road/plan/budgets.toml`, which stays the source of truth:
//! `tests/optimization_budgets.rs` reads that file and requires this table to
//! equal it field for field.

use core::fmt;

/// One `-O` tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OptTier {
    O0,
    O1,
    O2,
    O3,
}

/// What a tie between the candidate and the current plan resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TiePolicy {
    /// Keep the current, simpler shape.
    Reject,
}

impl TiePolicy {
    /// Stable `budgets.toml` spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
        }
    }
}

/// One `[optimization]` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptimizationBudget {
    pub tier: OptTier,
    /// Candidates one component may try; every measured trial spends one.
    pub candidate_budget: u32,
    pub emitted_size_epsilon_pct: u32,
    pub reactive_edge_epsilon_pct: u32,
    pub update_path_epsilon_pct: u32,
    /// Metrics that must strictly improve before a candidate commits.
    pub required_improvements_min: u32,
    pub tie_policy: TiePolicy,
}

impl OptTier {
    /// Every tier, lowest first.
    pub const ALL: [Self; 4] = [Self::O0, Self::O1, Self::O2, Self::O3];

    /// Stable spelling, as in `budgets.toml` and `-O` flags.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::O0 => "O0",
            Self::O1 => "O1",
            Self::O2 => "O2",
            Self::O3 => "O3",
        }
    }

    /// Parse the stable spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"O0" => Some(Self::O0),
            b"O1" => Some(Self::O1),
            b"O2" => Some(Self::O2),
            b"O3" => Some(Self::O3),
            _ => None,
        }
    }

    /// The pinned budget row for this tier.
    #[must_use]
    pub const fn budget(self) -> OptimizationBudget {
        let (candidate_budget, required_improvements_min) = match self {
            Self::O0 => (0, 0),
            Self::O1 => (8, 1),
            Self::O2 => (32, 1),
            Self::O3 => (128, 1),
        };
        OptimizationBudget {
            tier: self,
            candidate_budget,
            emitted_size_epsilon_pct: 0,
            reactive_edge_epsilon_pct: 0,
            update_path_epsilon_pct: 0,
            required_improvements_min,
            tie_policy: TiePolicy::Reject,
        }
    }
}

impl fmt::Display for OptTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
