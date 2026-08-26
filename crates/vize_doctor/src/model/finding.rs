use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use super::{
    assessment::{DoctorCategory, FindingAssessment, FixSafety, SuppressionPolicy},
    evidence::{
        AnalysisProvenance, FindingContext, FindingEvidence, RelatedLocation, SourceLocation,
        TextEdit,
    },
};

/// Stable fallback used when a finding producer cannot provide a source edit.
pub const DEFAULT_UNAVAILABLE_FIX_REASON: &str = "No automatic fix is available for this finding.";

/// Source fix disposition and its verification plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindingFix {
    /// Safety classification used by CLI and editor consumers.
    pub safety: FixSafety,
    /// User-visible fix title.
    pub title: String,
    /// Deterministically applied source edits. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<TextEdit>,
    /// Post-fix verification commands or checks. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification: Vec<String>,
}

impl FindingFix {
    /// Creates a fix without edits or verification steps.
    pub fn new(safety: FixSafety, title: impl Into<String>) -> Self {
        Self {
            safety,
            title: title.into(),
            edits: Vec::new(),
            verification: Vec::new(),
        }
    }

    /// Creates an explicit no-fix disposition with a user-visible reason.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::new(FixSafety::Unavailable, reason)
    }

    /// Adds a source edit.
    pub fn with_edit(mut self, edit: TextEdit) -> Self {
        self.edits.push(edit);
        self
    }

    /// Adds a post-fix verification step.
    pub fn with_verification(mut self, verification: impl Into<String>) -> Self {
        self.verification.push(verification.into());
        self
    }
}

/// Complete source-aware application health finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DoctorFinding {
    /// Stable machine-readable rule code.
    pub code: String,
    /// Health and ownership category.
    pub category: DoctorCategory,
    /// Required severity, confidence, impact, and score assessment.
    pub assessment: FindingAssessment,
    /// Primary authored source span.
    pub primary: SourceLocation,
    /// Concise user-visible title.
    pub title: String,
    /// Explanation and next action.
    pub message: String,
    /// Concrete failure scenario. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_scenario: Option<String>,
    /// Stable documentation path or URL. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// Related source locations. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedLocation>,
    /// Supporting evidence. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<FindingEvidence>,
    /// Source fix or explicit reason no automatic fix is available.
    ///
    /// The optional wire shape is retained for format-version compatibility;
    /// constructors and reports always populate this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<FindingFix>,
    /// Application graph context. Defaults to empty.
    #[serde(default)]
    pub context: FindingContext,
    /// Analysis capability, cost, and invalidation inputs.
    pub provenance: AnalysisProvenance,
    /// Suppression behavior. Defaults to reason-required.
    #[serde(default)]
    pub suppression: SuppressionPolicy,
}

impl DoctorFinding {
    /// Creates a finding with empty optional evidence and reason-required suppression.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        code: impl Into<String>,
        category: DoctorCategory,
        assessment: FindingAssessment,
        primary: SourceLocation,
        title: impl Into<String>,
        message: impl Into<String>,
        provenance: AnalysisProvenance,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            assessment,
            primary,
            title: title.into(),
            message: message.into(),
            failure_scenario: None,
            documentation: None,
            related: Vec::new(),
            evidence: Vec::new(),
            fix: Some(FindingFix::unavailable(DEFAULT_UNAVAILABLE_FIX_REASON)),
            context: FindingContext::default(),
            provenance,
            suppression: SuppressionPolicy::ReasonRequired,
        }
    }

    /// Adds a concrete failure scenario.
    pub fn with_failure_scenario(mut self, scenario: impl Into<String>) -> Self {
        self.failure_scenario = Some(scenario.into());
        self
    }

    /// Adds stable rule documentation.
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    /// Adds a related source location.
    pub fn with_related(mut self, related: RelatedLocation) -> Self {
        self.related.push(related);
        self
    }

    /// Adds supporting evidence.
    pub fn with_evidence(mut self, evidence: FindingEvidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Attaches a source fix.
    pub fn with_fix(mut self, fix: FindingFix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Attaches application graph context.
    pub fn with_context(mut self, context: FindingContext) -> Self {
        self.context = context;
        self
    }

    /// Overrides the source suppression policy.
    pub const fn with_suppression(mut self, suppression: SuppressionPolicy) -> Self {
        self.suppression = suppression;
        self
    }

    /// Returns a deterministic key for baselines and changed-finding policies.
    pub fn baseline_key(&self) -> String {
        cstr!(
            "{}:{}:{}:{}",
            self.code,
            self.primary.path,
            self.primary.start,
            self.primary.end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EvidenceKind, FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost,
    };

    fn finding() -> DoctorFinding {
        DoctorFinding::new(
            "VIZE_DOCTOR_TEST_001",
            DoctorCategory::Correctness,
            FindingAssessment::new(
                FindingSeverity::Warning,
                FindingConfidence::High,
                FindingImpact::Medium,
                HealthPenalty::new(12, "Test penalty"),
            ),
            SourceLocation::new("src/App.vue", 20, 30),
            "Test finding",
            "Test message",
            AnalysisProvenance::new("test-graph", RuleCost::Low),
        )
    }

    #[test]
    fn normalizes_locations_and_incremental_inputs() {
        let location = SourceLocation::new("src/App.vue", 30, 20);
        let provenance = AnalysisProvenance::new("reactivity-graph", RuleCost::Moderate)
            .with_invalidation_inputs(["src/b.ts", "src/a.ts", "src/b.ts"]);

        assert!(location.is_empty());
        assert_eq!(location.start, 30);
        assert_eq!(location.end, 30);
        assert_eq!(provenance.invalidation_inputs, ["src/a.ts", "src/b.ts"]);
    }

    #[test]
    fn preserves_evidence_fixes_context_and_baseline_identity() {
        let finding = finding()
            .with_failure_scenario("The rendered value stays stale.")
            .with_documentation("/doctor/test-001")
            .with_related(RelatedLocation::new(
                SourceLocation::new("src/state.ts", 4, 9),
                "Reactive owner",
            ))
            .with_evidence(
                FindingEvidence::new(EvidenceKind::Reactivity, "Dependency edge is absent")
                    .with_location(SourceLocation::new("src/App.vue", 20, 30))
                    .with_detail("binding", "count"),
            )
            .with_fix(
                FindingFix::new(FixSafety::Safe, "Move the read")
                    .with_edit(TextEdit::new(
                        SourceLocation::new("src/App.vue", 20, 30),
                        "derived.value",
                    ))
                    .with_verification("vize doctor src/App.vue"),
            )
            .with_context(FindingContext {
                target: Some("web".into()),
                component: Some("App".into()),
                ..FindingContext::default()
            })
            .with_suppression(SuppressionPolicy::Forbidden);

        assert_eq!(
            finding.baseline_key(),
            "VIZE_DOCTOR_TEST_001:src/App.vue:20:30"
        );
        assert_eq!(finding.related.len(), 1);
        assert_eq!(finding.evidence[0].details["binding"], "count");
        assert_eq!(finding.fix.as_ref().unwrap().edits.len(), 1);
        assert_eq!(finding.context.target.as_deref(), Some("web"));
        assert_eq!(finding.suppression, SuppressionPolicy::Forbidden);
    }

    #[test]
    fn clamps_penalties_and_defaults_missing_suppression_policy() {
        assert_eq!(HealthPenalty::new(255, "bounded").points, 100);

        let mut value = serde_json::to_value(finding()).unwrap();
        value.as_object_mut().unwrap().remove("suppression");
        let decoded: DoctorFinding = serde_json::from_value(value).unwrap();

        assert_eq!(decoded.suppression, SuppressionPolicy::ReasonRequired);
    }
    #[test]
    fn new_findings_have_an_explicit_unavailable_fix() {
        let finding = finding();
        let fix = finding.fix.as_ref().unwrap();

        assert_eq!(fix.safety, FixSafety::Unavailable);
        assert_eq!(fix.title, DEFAULT_UNAVAILABLE_FIX_REASON);
        assert!(fix.edits.is_empty());
        assert!(fix.verification.is_empty());
    }
}
