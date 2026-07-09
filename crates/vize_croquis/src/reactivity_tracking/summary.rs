use super::{
    BindingState, ReactiveOrigin, ViolationKind, ViolationSeverity, tracker::ReactivityTracker,
};
use serde::Serialize;

/// Aggregated counts by [`ReactiveOrigin`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactiveOriginCounts {
    pub ref_binding: usize,
    pub shallow_ref: usize,
    pub reactive: usize,
    pub shallow_reactive: usize,
    pub readonly: usize,
    pub shallow_readonly: usize,
    pub computed: usize,
    pub to_ref: usize,
    pub to_refs: usize,
    pub inject: usize,
    pub props: usize,
    pub pinia_store: usize,
    pub composable_return: usize,
    pub derived: usize,
    pub unknown: usize,
}

impl ReactiveOriginCounts {
    fn record(&mut self, origin: &ReactiveOrigin) {
        match origin {
            ReactiveOrigin::Ref => self.ref_binding += 1,
            ReactiveOrigin::ShallowRef => self.shallow_ref += 1,
            ReactiveOrigin::Reactive => self.reactive += 1,
            ReactiveOrigin::ShallowReactive => self.shallow_reactive += 1,
            ReactiveOrigin::Readonly => self.readonly += 1,
            ReactiveOrigin::ShallowReadonly => self.shallow_readonly += 1,
            ReactiveOrigin::Computed => self.computed += 1,
            ReactiveOrigin::ToRef => self.to_ref += 1,
            ReactiveOrigin::ToRefs => self.to_refs += 1,
            ReactiveOrigin::Inject => self.inject += 1,
            ReactiveOrigin::Props => self.props += 1,
            ReactiveOrigin::PiniaStore => self.pinia_store += 1,
            ReactiveOrigin::ComposableReturn { .. } => self.composable_return += 1,
            ReactiveOrigin::Derived { .. } => self.derived += 1,
            ReactiveOrigin::Unknown => self.unknown += 1,
        }
    }
}

/// Aggregated counts by [`BindingState`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingStateCounts {
    pub active: usize,
    pub reactivity_lost: usize,
    pub moved: usize,
    pub escaped: usize,
    pub reassigned: usize,
}

impl BindingStateCounts {
    fn record(&mut self, state: BindingState) {
        match state {
            BindingState::Active => self.active += 1,
            BindingState::ReactivityLost => self.reactivity_lost += 1,
            BindingState::Moved => self.moved += 1,
            BindingState::Escaped => self.escaped += 1,
            BindingState::Reassigned => self.reassigned += 1,
        }
    }
}

/// Aggregated counts by [`ViolationSeverity`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViolationSeverityCounts {
    pub error: usize,
    pub warning: usize,
    pub info: usize,
    pub hint: usize,
}

impl ViolationSeverityCounts {
    fn record(&mut self, severity: ViolationSeverity) {
        match severity {
            ViolationSeverity::Error => self.error += 1,
            ViolationSeverity::Warning => self.warning += 1,
            ViolationSeverity::Info => self.info += 1,
            ViolationSeverity::Hint => self.hint += 1,
        }
    }
}

/// Aggregated counts by [`ViolationKind`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViolationKindCounts {
    pub destructuring_loss: usize,
    pub spread_loss: usize,
    pub reassignment: usize,
    pub missing_value_access: usize,
    pub scope_escape: usize,
    pub unsafe_closure_capture: usize,
    pub external_mutation: usize,
    pub wrong_unwrap_context: usize,
    pub pinia_destructure: usize,
    pub props_destructure: usize,
    pub inject_destructure: usize,
    pub to_refs_on_non_reactive: usize,
    pub double_unwrap: usize,
    pub reactive_const: usize,
    pub shallow_deep_mismatch: usize,
}

impl ViolationKindCounts {
    fn record(&mut self, kind: &ViolationKind) {
        match kind {
            ViolationKind::DestructuringLoss { .. } => self.destructuring_loss += 1,
            ViolationKind::SpreadLoss => self.spread_loss += 1,
            ViolationKind::Reassignment => self.reassignment += 1,
            ViolationKind::MissingValueAccess => self.missing_value_access += 1,
            ViolationKind::ScopeEscape { .. } => self.scope_escape += 1,
            ViolationKind::UnsafeClosureCapture => self.unsafe_closure_capture += 1,
            ViolationKind::ExternalMutation => self.external_mutation += 1,
            ViolationKind::WrongUnwrapContext => self.wrong_unwrap_context += 1,
            ViolationKind::PiniaDestructure => self.pinia_destructure += 1,
            ViolationKind::PropsDestructure => self.props_destructure += 1,
            ViolationKind::InjectDestructure => self.inject_destructure += 1,
            ViolationKind::ToRefsOnNonReactive => self.to_refs_on_non_reactive += 1,
            ViolationKind::DoubleUnwrap => self.double_unwrap += 1,
            ViolationKind::ReactiveConst => self.reactive_const += 1,
            ViolationKind::ShallowDeepMismatch => self.shallow_deep_mismatch += 1,
        }
    }
}

/// Stable, consumer-friendly summary of a [`ReactivityTracker`] run.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityTrackerSummary {
    pub binding_count: usize,
    pub violation_count: usize,
    pub use_site_count: usize,
    pub scope_count: usize,
    pub max_scope_depth: u32,
    pub bindings_by_origin: ReactiveOriginCounts,
    pub bindings_by_state: BindingStateCounts,
    pub violations_by_severity: ViolationSeverityCounts,
}

/// Reactivity tracker summary plus derived violation-kind counters.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityTrackerDetailedSummary {
    #[serde(flatten)]
    pub summary: ReactivityTrackerSummary,
    pub violations_by_kind: ViolationKindCounts,
}

impl ReactivityTracker {
    /// Build a stable summary for downstream diagnostics and reports.
    pub fn summary(&self) -> ReactivityTrackerSummary {
        let mut summary = ReactivityTrackerSummary {
            binding_count: self.bindings.len(),
            violation_count: self.violations.len(),
            scope_count: self.scopes.len(),
            max_scope_depth: self
                .scopes
                .iter()
                .map(|scope| scope.depth)
                .max()
                .unwrap_or_default(),
            ..ReactivityTrackerSummary::default()
        };

        for binding in self.bindings.values() {
            summary.bindings_by_origin.record(&binding.origin);
            summary.bindings_by_state.record(binding.state);
            summary.use_site_count += binding.use_sites.len();
        }

        for violation in &self.violations {
            summary.violations_by_severity.record(violation.severity);
        }

        summary
    }

    /// Count violations by kind without changing the stable summary struct shape.
    pub fn violation_kind_counts(&self) -> ViolationKindCounts {
        let mut counts = ViolationKindCounts::default();

        for violation in &self.violations {
            counts.record(&violation.kind);
        }

        counts
    }

    /// Build a detailed summary for serialized reports that need violation kinds.
    pub fn detailed_summary(&self) -> ReactivityTrackerDetailedSummary {
        ReactivityTrackerDetailedSummary {
            summary: self.summary(),
            violations_by_kind: self.violation_kind_counts(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reactivity_tracking::{ReactiveOrigin, ReactivityTracker, UseSiteKind};
    use vize_carton::CompactString;

    #[test]
    fn summary_counts_bindings_use_sites_and_violations() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let state = tracker.add_binding(
            CompactString::new("state"),
            ReactiveOrigin::Reactive,
            false,
            0,
            10,
        );
        let count = tracker.add_binding(
            CompactString::new("count"),
            ReactiveOrigin::Ref,
            false,
            11,
            20,
        );
        let store = tracker.add_binding(
            CompactString::new("store"),
            ReactiveOrigin::PiniaStore,
            false,
            21,
            30,
        );

        tracker.mark_reactivity_lost(state);
        tracker.mark_escaped(store);
        tracker.record_use(
            state,
            UseSiteKind::Destructure {
                extracted_props: vec![CompactString::new("name")],
            },
            31,
            50,
        );
        tracker.record_use(count, UseSiteKind::Read, 51, 56);
        tracker.record_use(
            store,
            UseSiteKind::ExternalEscape {
                target: CompactString::new("window"),
            },
            57,
            70,
        );

        let summary = tracker.summary();

        assert_eq!(summary.binding_count, 3);
        assert_eq!(summary.violation_count, 3);
        assert_eq!(summary.use_site_count, 3);
        assert_eq!(summary.scope_count, 2);
        assert_eq!(summary.max_scope_depth, 1);
        assert_eq!(summary.bindings_by_origin.reactive, 1);
        assert_eq!(summary.bindings_by_origin.ref_binding, 1);
        assert_eq!(summary.bindings_by_origin.pinia_store, 1);
        assert_eq!(summary.bindings_by_state.active, 1);
        assert_eq!(summary.bindings_by_state.reactivity_lost, 1);
        assert_eq!(summary.bindings_by_state.escaped, 1);
        assert_eq!(summary.violations_by_severity.error, 1);
        assert_eq!(summary.violations_by_severity.warning, 1);
        assert_eq!(summary.violations_by_severity.hint, 1);

        let violations_by_kind = tracker.violation_kind_counts();
        assert_eq!(violations_by_kind.destructuring_loss, 1);
        assert_eq!(violations_by_kind.external_mutation, 1);
        assert_eq!(violations_by_kind.missing_value_access, 1);

        let json = serde_json::to_string(&tracker.detailed_summary()).unwrap();
        assert!(json.contains(r#""bindingsByOrigin""#));
        assert!(json.contains(r#""missingValueAccess":1"#));
    }
}
