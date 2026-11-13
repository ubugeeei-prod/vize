//! Owned plain-module checker work and its static findings.

use super::super::reactivity_loss::ReactivityLossQuery;
use super::super::{
    CorsaTypeAwareSession, LintResult, Linter, TypeAwareFailure, typescript_eslint,
};
use super::report_probe;
use vize_l0::String;

pub(super) struct PlainQueryPlan {
    pub content: String,
    /// Reactivity-loss probes: offsets of the probe names in `content`.
    pub probes: Vec<(u32, ReactivityLossQuery)>,
    /// `type/no-floating-promises` over the module's own bytes.
    pub typescript: Option<Box<typescript_eslint::Plan>>,
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
    /// The two checks fail independently; the first failure is reported.
    pub(in crate::linter) fn evaluate(
        &mut self,
        linter: &Linter,
        session: &mut CorsaTypeAwareSession,
    ) -> Result<(), String> {
        let Some(plan) = &self.plan else {
            return Ok(());
        };
        let typed = plan.typescript.as_ref().map_or(Ok(()), |typescript| {
            typescript_eslint::evaluate(
                session,
                typescript,
                &plan.content,
                linter,
                &mut self.result,
            )
        });
        let probed = if plan.probes.is_empty() {
            Ok(())
        } else {
            let requests = plan
                .probes
                .iter()
                .map(|(offset, _)| (*offset, false, false))
                .collect::<Vec<_>>();
            session
                .probe_types_at_offsets(&plan.content, &requests)
                .map(|probes| {
                    for ((_, query), probe) in plan.probes.iter().zip(probes) {
                        report_probe(&mut self.result, query, probe.as_ref());
                    }
                })
        };
        typed.and(probed)
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
