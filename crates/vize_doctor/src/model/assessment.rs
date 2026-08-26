use serde::{Deserialize, Serialize};
use vize_s0::String;

/// Health category used for reporting, scoring, ownership, and policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorCategory {
    /// SFC, type, control-flow, reactivity, and contract correctness.
    Correctness,
    /// Semantic, keyboard, focus, assistive, locale, and modality support.
    Accessibility,
    /// Build, output, startup, rendering, memory, and interaction performance.
    Performance,
    /// Architecture, ownership, documentation, complexity, and extension health.
    Maintainability,
    /// Trust boundaries, unsafe data flow, permissions, and exploitable behavior.
    Security,
    /// Deployment, observability, lifecycle, resilience, and operational health.
    ProductionReadiness,
}

impl DoctorCategory {
    /// All score categories in stable serialized order.
    pub const ALL: [Self; 6] = [
        Self::Correctness,
        Self::Accessibility,
        Self::Performance,
        Self::Maintainability,
        Self::Security,
        Self::ProductionReadiness,
    ];
}

/// Whether a finding prevents success or guides a non-blocking improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingSeverity {
    /// Failure-severity finding; default blocking still requires high confidence.
    Error,
    /// Likely defect or significant risk that requires attention.
    Warning,
    /// Non-blocking opportunity with concrete supporting evidence.
    Notice,
}

/// Strength of the evidence supporting a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingConfidence {
    /// The analyzer proved the finding from complete semantic information.
    Certain,
    /// Evidence is strong but one bounded runtime condition remains unknown.
    High,
    /// Evidence is useful but depends on several explicit assumptions.
    Medium,
    /// The finding is exploratory and must never block by default.
    Low,
}

/// User and production consequence if a finding occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingImpact {
    /// Application safety, data, or core operation is at immediate risk.
    Critical,
    /// A primary workflow or production invariant is affected.
    High,
    /// A bounded workflow, target, or maintainability invariant is affected.
    Medium,
    /// The consequence is narrow and has a reliable fallback.
    Low,
}

/// Whether an automatic fix can be applied without human judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixSafety {
    /// The edit is semantics-preserving and may be applied automatically.
    Safe,
    /// The edit is concrete but requires review of product behavior.
    ReviewRequired,
    /// No generally correct source edit is available.
    Unavailable,
}

/// Policy governing source-level suppression of a finding.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum SuppressionPolicy {
    /// Suppression is accepted without additional text.
    Allowed,
    /// Suppression requires an authored justification.
    #[default]
    ReasonRequired,
    /// Suppression is rejected because the invariant is mandatory.
    Forbidden,
}

/// Expected cost of producing one rule's analysis capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleCost {
    /// Constant-time lookup over an existing analysis product.
    Trivial,
    /// Local file or node analysis with bounded traversal.
    Low,
    /// Affected-graph analysis across a bounded dependency region.
    Moderate,
    /// Workspace-wide or measurement-backed analysis.
    High,
}

/// Analysis product that provides one piece of supporting evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// Authored or generated source evidence.
    Source,
    /// Type-system evidence.
    Type,
    /// Control-flow evidence.
    ControlFlow,
    /// Reactivity or effect-graph evidence.
    Reactivity,
    /// Component, slot, prop, event, or semantic-tree evidence.
    Component,
    /// CSS cascade, selector, token, layout, or style evidence.
    Css,
    /// Module, route, environment, or build-graph evidence.
    BuildGraph,
    /// Contract, capability, backend, or transport evidence.
    Contract,
    /// Measured build or runtime evidence.
    Measurement,
}

/// Explicit deduction applied to one category's health score.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HealthPenalty {
    /// Deduction from the category score, clamped to `0..=100`.
    pub points: u8,
    /// Human-readable explanation of the scoring consequence.
    pub reason: String,
}

impl HealthPenalty {
    /// Creates an explainable score penalty.
    pub fn new(points: u8, reason: impl Into<String>) -> Self {
        Self {
            points: points.min(100),
            reason: reason.into(),
        }
    }
}

/// Required severity, confidence, impact, and scoring assessment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindingAssessment {
    /// Gate severity.
    pub severity: FindingSeverity,
    /// Strength of supporting evidence.
    pub confidence: FindingConfidence,
    /// Consequence if the issue occurs.
    pub impact: FindingImpact,
    /// Explainable category-score deduction.
    pub penalty: HealthPenalty,
}

impl FindingAssessment {
    /// Creates a complete finding assessment without inferred policy.
    pub const fn new(
        severity: FindingSeverity,
        confidence: FindingConfidence,
        impact: FindingImpact,
        penalty: HealthPenalty,
    ) -> Self {
        Self {
            severity,
            confidence,
            impact,
            penalty,
        }
    }
}
