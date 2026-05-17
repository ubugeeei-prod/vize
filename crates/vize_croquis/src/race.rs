//! Race-condition risk tracking for Vue reactive flows.
//!
//! The tracker stores parser-derived facts. Cross-file analyzers decide how
//! strict each fact should be for lint/typecheck callers.

use vize_carton::{CompactString, cstr};

/// A potentially racing async mutation pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaceConditionRiskKind {
    /// `watch(..., async () => { await ...; state.value = ... })` without cleanup.
    AsyncWatcherMutation {
        watcher_name: CompactString,
        async_operation: CompactString,
        mutated_targets: Vec<CompactString>,
    },
    /// `watchEffect(async () => { ... })` or async work inside watchEffect.
    AsyncWatchEffect {
        async_operation: CompactString,
        mutated_targets: Vec<CompactString>,
    },
    /// Lifecycle hook callback mutates reactive state after an async boundary.
    AsyncLifecycleMutation {
        hook_name: CompactString,
        async_operation: CompactString,
        mutated_targets: Vec<CompactString>,
    },
    /// Scheduled callback mutates reactive state without an obvious cleanup path.
    ScheduledMutation {
        scheduler_name: CompactString,
        mutated_targets: Vec<CompactString>,
    },
    /// Promise continuation mutates reactive state without cancellation/cleanup.
    PromiseContinuationMutation {
        async_operation: CompactString,
        mutated_targets: Vec<CompactString>,
    },
}

impl RaceConditionRiskKind {
    /// Reactive or injected roots mutated by this risk.
    pub fn mutated_targets(&self) -> &[CompactString] {
        match self {
            Self::AsyncWatcherMutation {
                mutated_targets, ..
            }
            | Self::AsyncWatchEffect {
                mutated_targets, ..
            }
            | Self::AsyncLifecycleMutation {
                mutated_targets, ..
            }
            | Self::ScheduledMutation {
                mutated_targets, ..
            }
            | Self::PromiseContinuationMutation {
                mutated_targets, ..
            } => mutated_targets,
        }
    }

    /// Human-readable async context for diagnostics.
    pub fn async_context(&self) -> CompactString {
        match self {
            Self::AsyncWatcherMutation {
                watcher_name,
                async_operation,
                ..
            } => cstr!("{watcher_name} {async_operation}"),
            Self::AsyncWatchEffect {
                async_operation, ..
            } => cstr!("watchEffect {async_operation}"),
            Self::AsyncLifecycleMutation {
                hook_name,
                async_operation,
                ..
            } => cstr!("{hook_name} {async_operation}"),
            Self::ScheduledMutation { scheduler_name, .. } => scheduler_name.clone(),
            Self::PromiseContinuationMutation {
                async_operation, ..
            } => async_operation.clone(),
        }
    }
}

/// A parser-detected race-condition risk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaceConditionRisk {
    pub kind: RaceConditionRiskKind,
    pub start: u32,
    pub end: u32,
}

/// Tracks race-condition risks found during script parsing.
#[derive(Debug, Default)]
pub struct RaceConditionTracker {
    risks: Vec<RaceConditionRisk>,
}

impl RaceConditionTracker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, kind: RaceConditionRiskKind, start: u32, end: u32) {
        self.risks.push(RaceConditionRisk { kind, start, end });
    }

    #[inline]
    pub fn risks(&self) -> &[RaceConditionRisk] {
        &self.risks
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.risks.is_empty()
    }

    /// Shift all stored source offsets by `delta`.
    pub fn shift_offsets(&mut self, delta: u32) {
        for risk in &mut self.risks {
            risk.start = risk.start.saturating_add(delta);
            risk.end = risk.end.saturating_add(delta);
        }
    }

    /// Merge another tracker into this one.
    pub fn extend(&mut self, other: Self) {
        self.risks.extend(other.risks);
    }
}

#[cfg(test)]
mod tests {
    use super::{RaceConditionRiskKind, RaceConditionTracker};
    use vize_carton::CompactString;

    #[test]
    fn test_race_condition_tracker() {
        let mut tracker = RaceConditionTracker::new();
        tracker.record(
            RaceConditionRiskKind::AsyncWatcherMutation {
                watcher_name: CompactString::new("watch"),
                async_operation: CompactString::new("await"),
                mutated_targets: vec![CompactString::new("state")],
            },
            10,
            40,
        );

        assert_eq!(tracker.risks().len(), 1);
        assert_eq!(tracker.risks()[0].kind.mutated_targets()[0], "state");
        assert!(!tracker.is_empty());
    }
}
