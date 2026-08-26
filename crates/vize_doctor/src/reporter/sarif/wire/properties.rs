//! Vize semantics preserved in SARIF property bags.

use serde::Serialize;

use super::super::plan::SarifPlan;
use crate::{
    DoctorFinding, DoctorReport, FindingContext, FindingEvidence, HealthPenalty, SuppressionPolicy,
};

#[derive(Serialize)]
pub(super) struct SarifResultProperties<'finding> {
    #[serde(rename = "vizeCategory")]
    category: crate::DoctorCategory,
    #[serde(rename = "vizeConfidence")]
    confidence: crate::FindingConfidence,
    #[serde(rename = "vizeImpact")]
    impact: crate::FindingImpact,
    #[serde(rename = "vizePenalty")]
    penalty: &'finding HealthPenalty,
    #[serde(
        rename = "vizeFailureScenario",
        skip_serializing_if = "Option::is_none"
    )]
    failure_scenario: Option<&'finding str>,
    #[serde(rename = "vizeDocumentation", skip_serializing_if = "Option::is_none")]
    documentation: Option<&'finding str>,
    #[serde(rename = "vizeEvidence", skip_serializing_if = "Vec::is_empty")]
    evidence: &'finding Vec<FindingEvidence>,
    #[serde(rename = "vizeContext")]
    context: &'finding FindingContext,
    #[serde(rename = "vizeFixSafety", skip_serializing_if = "Option::is_none")]
    fix_safety: Option<crate::FixSafety>,
    #[serde(rename = "vizeFixTitle", skip_serializing_if = "Option::is_none")]
    fix_title: Option<&'finding str>,
    #[serde(rename = "vizeVerification", skip_serializing_if = "Option::is_none")]
    verification: Option<&'finding [vize_s0::String]>,
    #[serde(rename = "vizeSuppressionPolicy")]
    suppression_policy: SuppressionPolicy,
    #[serde(rename = "vizeProvenance")]
    provenance: &'finding crate::AnalysisProvenance,
    #[serde(rename = "vizeSourceRegionsOmitted", skip_serializing_if = "is_zero")]
    source_regions_omitted: u64,
    #[serde(rename = "vizeFixEditsOmitted", skip_serializing_if = "is_zero")]
    fix_edits_omitted: u64,
}

impl<'finding> SarifResultProperties<'finding> {
    pub(super) fn new<'source>(
        plan: &SarifPlan<'finding, 'source>,
        finding: &'finding DoctorFinding,
    ) -> Self {
        let fix = finding.fix.as_ref();
        let source_regions_omitted = std::iter::once(&finding.primary)
            .chain(finding.related.iter().map(|related| &related.location))
            .chain(
                finding
                    .evidence
                    .iter()
                    .filter_map(|evidence| evidence.location.as_ref()),
            )
            .filter(|location| plan.region(location).is_none())
            .count() as u64;
        let fix_edits_omitted = fix
            .filter(|fix| !plan.can_render_fix(fix))
            .map(|fix| fix.edits.len() as u64)
            .unwrap_or(0);
        Self {
            category: finding.category,
            confidence: finding.assessment.confidence,
            impact: finding.assessment.impact,
            penalty: &finding.assessment.penalty,
            failure_scenario: finding.failure_scenario.as_deref(),
            documentation: finding.documentation.as_deref(),
            evidence: &finding.evidence,
            context: &finding.context,
            fix_safety: fix.map(|fix| fix.safety),
            fix_title: fix.map(|fix| fix.title.as_str()),
            verification: fix
                .map(|fix| fix.verification.as_slice())
                .filter(|items| !items.is_empty()),
            suppression_policy: finding.suppression,
            provenance: &finding.provenance,
            source_regions_omitted,
            fix_edits_omitted,
        }
    }
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Serialize)]
pub(super) struct SarifRunProperties<'report> {
    #[serde(rename = "vizeReportFormatVersion")]
    report_format_version: u32,
    #[serde(rename = "vizeScoringVersion")]
    scoring_version: u32,
    #[serde(rename = "vizeWorkspace")]
    workspace: &'report str,
    #[serde(rename = "vizeHealth")]
    health: &'report crate::DoctorSummary,
}

impl<'report> SarifRunProperties<'report> {
    pub(super) fn new(report: &'report DoctorReport) -> Self {
        Self {
            report_format_version: report.format_version(),
            scoring_version: report.scoring_version(),
            workspace: report.workspace(),
            health: report.summary(),
        }
    }
}
