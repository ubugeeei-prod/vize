#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, de};
use vize_s0::String;

use crate::{
    DEFAULT_UNAVAILABLE_FIX_REASON, DoctorCategory, DoctorFinding, FindingConfidence, FindingFix,
    FindingSeverity,
};

/// Current serialized doctor report format.
pub const DOCTOR_REPORT_FORMAT_VERSION: u32 = 2;

/// Current explainable health-scoring model.
pub const DOCTOR_SCORING_VERSION: u32 = 1;

/// Score and deduction summary for one doctor category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CategoryHealth {
    /// Health score in the inclusive range `0..=100`.
    pub score: u8,
    /// Total uncapped penalty points supplied by findings.
    pub penalty: u32,
    /// Number of findings assigned to this category.
    pub findings: u32,
}

impl Default for CategoryHealth {
    fn default() -> Self {
        Self {
            score: 100,
            penalty: 0,
            findings: 0,
        }
    }
}

/// Finding counts grouped by gate severity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindingCounts {
    /// Findings classified with error severity.
    pub errors: u32,
    /// Significant non-error risks.
    pub warnings: u32,
    /// Non-blocking evidence-backed opportunities.
    pub notices: u32,
}

impl FindingCounts {
    /// Returns the total finding count with saturating arithmetic.
    pub const fn total(self) -> u32 {
        self.errors
            .saturating_add(self.warnings)
            .saturating_add(self.notices)
    }
}

/// Explainable report-level health and gate summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DoctorSummary {
    /// Equal-weight average of all category scores.
    pub overall_score: u8,
    /// Stable per-category scores and penalties.
    pub categories: BTreeMap<DoctorCategory, CategoryHealth>,
    /// Counts grouped by severity.
    pub counts: FindingCounts,
    /// Whether a certain or high-confidence error blocks default success.
    pub has_blocking_errors: bool,
}

impl DoctorSummary {
    fn from_findings(findings: &[DoctorFinding]) -> Self {
        let mut categories = DoctorCategory::ALL
            .into_iter()
            .map(|category| (category, CategoryHealth::default()))
            .collect::<BTreeMap<_, _>>();
        let mut counts = FindingCounts::default();
        let mut has_blocking_errors = false;

        for finding in findings {
            let health = categories
                .get_mut(&finding.category)
                .expect("every doctor category has a score entry");
            health.penalty = health
                .penalty
                .saturating_add(u32::from(finding.assessment.penalty.points));
            health.findings = health.findings.saturating_add(1);

            match finding.assessment.severity {
                FindingSeverity::Error => {
                    counts.errors = counts.errors.saturating_add(1);
                    has_blocking_errors |= matches!(
                        finding.assessment.confidence,
                        FindingConfidence::Certain | FindingConfidence::High
                    );
                }
                FindingSeverity::Warning => {
                    counts.warnings = counts.warnings.saturating_add(1);
                }
                FindingSeverity::Notice => {
                    counts.notices = counts.notices.saturating_add(1);
                }
            }
        }

        for health in categories.values_mut() {
            health.score = 100_u8.saturating_sub(health.penalty.min(100) as u8);
        }
        let score_total = categories
            .values()
            .fold(0_u32, |total, health| total + u32::from(health.score));
        let overall_score = (score_total / DoctorCategory::ALL.len() as u32) as u8;

        Self {
            overall_score,
            categories,
            counts,
            has_blocking_errors,
        }
    }
}

/// Deterministically ranked, versioned whole-application health report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    /// Serialized report format.
    format_version: u32,
    /// Explainable scoring model version.
    scoring_version: u32,
    /// Stable workspace or application identifier.
    workspace: String,
    /// Findings ranked by severity, impact, confidence, category, and source.
    findings: Vec<DoctorFinding>,
    /// Health and gate summary derived from findings.
    summary: DoctorSummary,
}

impl DoctorReport {
    /// Creates a deterministic report from findings in any input order.
    pub fn new(
        workspace: impl Into<String>,
        findings: impl IntoIterator<Item = DoctorFinding>,
    ) -> Self {
        let findings = normalize_findings(findings.into_iter().collect());
        Self::from_normalized_findings(workspace, findings)
    }

    /// Scores findings already normalized by [`normalize_findings`].
    ///
    /// This crate-private entry point lets validated capability snapshots avoid
    /// repeating their ordering and deduplication pass when materialized as a
    /// whole-application report.
    pub(crate) fn from_normalized_findings(
        workspace: impl Into<String>,
        findings: Vec<DoctorFinding>,
    ) -> Self {
        let summary = DoctorSummary::from_findings(&findings);
        Self {
            format_version: DOCTOR_REPORT_FORMAT_VERSION,
            scoring_version: DOCTOR_SCORING_VERSION,
            workspace: workspace.into(),
            findings,
            summary,
        }
    }

    /// Returns the serialized report format.
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the explainable scoring model version.
    pub const fn scoring_version(&self) -> u32 {
        self.scoring_version
    }

    /// Returns the stable workspace or application identifier.
    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    /// Returns deterministically ranked findings.
    pub fn findings(&self) -> &[DoctorFinding] {
        &self.findings
    }

    /// Returns the health and gate summary derived from findings.
    pub const fn summary(&self) -> &DoctorSummary {
        &self.summary
    }
}

pub(crate) fn normalize_findings(mut findings: Vec<DoctorFinding>) -> Vec<DoctorFinding> {
    for finding in &mut findings {
        if finding.fix.is_none() {
            finding.fix = Some(FindingFix::unavailable(DEFAULT_UNAVAILABLE_FIX_REASON));
        }
        finding.related.sort();
        finding.evidence.sort();
        finding.provenance.invalidation_inputs.sort();
        finding.provenance.invalidation_inputs.dedup();
        finding
            .provenance
            .invalidation_fingerprints
            .retain(|input, _| {
                finding
                    .provenance
                    .invalidation_inputs
                    .binary_search(input)
                    .is_ok()
            });
        if let Some(fix) = &mut finding.fix {
            fix.edits.sort();
            fix.verification.sort();
            fix.verification.dedup();
        }
    }
    findings.sort_by(compare_findings);
    findings
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DoctorReportWire {
    format_version: u32,
    scoring_version: u32,
    workspace: String,
    findings: Vec<DoctorFinding>,
    summary: DoctorSummary,
}

impl<'de> Deserialize<'de> for DoctorReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DoctorReportWire::deserialize(deserializer)?;
        if wire.format_version != DOCTOR_REPORT_FORMAT_VERSION {
            return Err(de::Error::custom(
                "unsupported doctor report format version",
            ));
        }
        if wire.scoring_version != DOCTOR_SCORING_VERSION {
            return Err(de::Error::custom("unsupported doctor scoring version"));
        }

        let report = Self::new(wire.workspace, wire.findings);
        if wire.summary != report.summary {
            return Err(de::Error::custom(
                "doctor report summary does not match its findings",
            ));
        }
        Ok(report)
    }
}

fn compare_findings(left: &DoctorFinding, right: &DoctorFinding) -> std::cmp::Ordering {
    (
        left.assessment.severity,
        left.assessment.impact,
        left.assessment.confidence,
        left.category,
        &left.primary.path,
        left.primary.start,
        left.primary.end,
        &left.code,
        &left.message,
    )
        .cmp(&(
            right.assessment.severity,
            right.assessment.impact,
            right.assessment.confidence,
            right.category,
            &right.primary.path,
            right.primary.start,
            right.primary.end,
            &right.code,
            &right.message,
        ))
        .then_with(|| left.cmp(right))
}
