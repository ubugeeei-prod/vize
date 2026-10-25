//! Owned plain-module checker work and its static findings.

use super::super::{CorsaTypeAwareSession, LintResult, Linter, TypeAwareFailure};
use super::{Probe, report_probe};
use vize_l0::String;

pub(super) struct PlainQueryPlan {
    pub content: String,
    pub probes: Vec<(u32, Probe)>,
}

pub(in crate::linter) struct PreparedPlainModule {
    result: LintResult,
    plan: Option<PlainQueryPlan>,
    apply_severity: bool,
}

impl PreparedPlainModule {
    pub(super) fn new(
        result: LintResult,
        plan: Option<PlainQueryPlan>,
        apply_severity: bool,
    ) -> Self {
        Self {
            result,
            plan,
            apply_severity,
        }
    }

    pub(in crate::linter) fn virtual_content(&self) -> Option<&str> {
        self.plan.as_ref().map(|plan| plan.content.as_str())
    }

    /// Query the document the caller selected in the published project snapshot.
    pub(in crate::linter) fn evaluate(
        &mut self,
        _linter: &Linter,
        session: &mut CorsaTypeAwareSession,
    ) -> Result<(), String> {
        let Some(plan) = &self.plan else {
            return Ok(());
        };
        let requests = plan
            .probes
            .iter()
            .map(|(offset, kind)| (*offset, false, kind.loads_signatures()))
            .collect::<Vec<_>>();
        let probes = session.probe_types_at_offsets(&plan.content, &requests)?;
        for ((_, kind), probe) in plan.probes.iter().zip(probes) {
            report_probe(&mut self.result, kind, probe.as_ref());
        }
        Ok(())
    }

    pub(in crate::linter) fn finish(
        mut self,
        linter: &Linter,
        failure: Option<TypeAwareFailure>,
    ) -> LintResult {
        if self.plan.is_some()
            && let Some(failure) = failure
        {
            super::super::push_type_aware_failure(&mut self.result, failure);
        }
        if self.apply_severity {
            super::super::super::severity::apply_severity_overrides(
                &mut self.result,
                &linter.severity_overrides,
            );
        }
        self.result
    }
}
