use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vize_s0::String;

use crate::{
    AnalysisProvenance, DoctorCategory, EvidenceKind, FindingAssessment, FindingContext, FixSafety,
    SourceLocation, SuppressionPolicy,
};

/// Current serialized provider-neutral AI context format.
pub const DOCTOR_AI_CONTEXT_FORMAT_VERSION: u32 = 1;

/// Hard limits applied while constructing an [`AiContextPacket`].
///
/// Limits are evaluated in deterministic report order, so higher-ranked
/// findings receive context first. A zero value intentionally disables that
/// resource. Source, replacement, and command byte limits count UTF-8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiContextBudget {
    /// Maximum findings. Defaults to 32.
    pub max_findings: u64,
    /// Maximum evidence facts retained for each finding. Defaults to 8.
    pub max_evidence_per_finding: u64,
    /// Maximum related locations retained for each finding. Defaults to 8.
    pub max_related_per_finding: u64,
    /// Maximum distinct source snippets in the packet. Defaults to 64.
    pub max_source_snippets: u64,
    /// Maximum combined source-snippet bytes. Defaults to 128 KiB.
    pub max_source_bytes: u64,
    /// Maximum bytes in one source snippet. Defaults to 4 KiB.
    pub max_source_bytes_per_snippet: u64,
    /// Maximum edit operations retained for each finding. Defaults to 16.
    pub max_edits_per_finding: u64,
    /// Maximum combined replacement-source bytes. Defaults to 64 KiB.
    pub max_edit_bytes: u64,
    /// Maximum verification steps retained for each finding. Defaults to 16.
    pub max_verification_steps_per_finding: u64,
    /// Maximum combined verification-command bytes. Defaults to 16 KiB.
    pub max_verification_bytes: u64,
}

impl Default for AiContextBudget {
    fn default() -> Self {
        Self {
            max_findings: 32,
            max_evidence_per_finding: 8,
            max_related_per_finding: 8,
            max_source_snippets: 64,
            max_source_bytes: 128 * 1024,
            max_source_bytes_per_snippet: 4 * 1024,
            max_edits_per_finding: 16,
            max_edit_bytes: 64 * 1024,
            max_verification_steps_per_finding: 16,
            max_verification_bytes: 16 * 1024,
        }
    }
}

/// Counts and paths explaining information omitted by packet budgets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiContextOmissions {
    /// Lower-ranked findings excluded by `max_findings`.
    pub dropped_findings: u64,
    /// Evidence facts excluded by per-finding limits.
    pub dropped_evidence_nodes: u64,
    /// Related locations excluded by per-finding limits.
    pub dropped_related_nodes: u64,
    /// Distinct source requests excluded by count or byte limits.
    pub dropped_source_snippets: u64,
    /// Sorted source paths requested by findings but absent from input sources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_source_paths: Vec<String>,
    /// Whole edit operations omitted by edit count or byte limits.
    pub dropped_edit_operations: u64,
    /// Whole verification steps omitted by count or byte limits.
    pub dropped_verification_steps: u64,
}

/// AI-facing finding data and references to its graph, source, and edit nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiFindingContext {
    /// Packet-local stable identifier, derived from deterministic report rank.
    pub id: String,
    /// Stable baseline identity from the Doctor finding contract.
    pub baseline_key: String,
    /// Stable machine-readable rule code.
    pub code: String,
    /// Health category.
    pub category: DoctorCategory,
    /// Severity, confidence, impact, and health penalty.
    pub assessment: FindingAssessment,
    /// Primary authored location.
    pub primary: SourceLocation,
    /// Concise finding title.
    pub title: String,
    /// Explanation and recommended next action.
    pub message: String,
    /// Concrete failure scenario, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_scenario: Option<String>,
    /// Stable documentation path or URL, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// Application graph context.
    pub context: FindingContext,
    /// Analysis capability, cost, and invalidation inputs.
    pub provenance: AnalysisProvenance,
    /// Suppression behavior required by policy.
    pub suppression: SuppressionPolicy,
    /// Root node in [`AiEvidenceGraph`].
    pub evidence_root_id: String,
    /// Packet source snippets relevant to this finding, in priority order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_snippet_ids: Vec<String>,
    /// Packet edit plan for this finding, when a fix disposition exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_plan_id: Option<String>,
}

/// Semantic role of a node in an [`AiEvidenceGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiEvidenceNodeKind {
    /// Root node representing the complete finding assertion.
    Finding,
    /// Structured evidence emitted by an analysis capability.
    Evidence,
    /// Related source relationship that explains the assertion.
    RelatedLocation,
}

/// One deterministic fact in an AI evidence graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiEvidenceNode {
    /// Packet-local stable node identifier.
    pub id: String,
    /// Owning finding identifier.
    pub finding_id: String,
    /// Semantic node role.
    pub kind: AiEvidenceNodeKind,
    /// Analysis evidence domain for evidence nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_kind: Option<EvidenceKind>,
    /// Concise factual statement.
    pub summary: String,
    /// Supporting authored location, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// Stable structured evidence details.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, String>,
}

/// Meaning of a directed [`AiEvidenceEdge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiEvidenceRelation {
    /// The target fact supports the source assertion.
    Supports,
    /// The target source relationship is relevant to the source assertion.
    Related,
}

/// Directed relationship between two evidence graph nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiEvidenceEdge {
    /// Source node identifier.
    pub from: String,
    /// Target node identifier.
    pub to: String,
    /// Semantic relationship.
    pub relation: AiEvidenceRelation,
}

/// Deterministically ordered evidence nodes and edges.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiEvidenceGraph {
    /// Nodes ordered by finding rank, then evidence priority.
    pub nodes: Vec<AiEvidenceNode>,
    /// Edges ordered with their target nodes.
    pub edges: Vec<AiEvidenceEdge>,
}

/// UTF-8 source excerpt with absolute focus and content byte ranges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiSourceSnippet {
    /// Packet-local stable snippet identifier.
    pub id: String,
    /// Workspace-relative source path.
    pub path: String,
    /// Inclusive absolute byte offset of `text` in the source file.
    pub content_start: u32,
    /// Exclusive absolute byte offset of `text` in the source file.
    pub content_end: u32,
    /// Inclusive absolute byte offset of the requested focus.
    pub focus_start: u32,
    /// Exclusive absolute byte offset of the requested focus.
    pub focus_end: u32,
    /// Exact UTF-8 source excerpt.
    pub text: String,
    /// Whether authored content exists before this excerpt.
    pub truncated_before: bool,
    /// Whether authored content exists after this excerpt.
    pub truncated_after: bool,
    /// Whether the source budget could not contain the complete focus span.
    pub focus_truncated: bool,
}

impl AiSourceSnippet {
    /// Return the focus start relative to `text`.
    pub const fn relative_focus_start(&self) -> u32 {
        self.focus_start.saturating_sub(self.content_start)
    }

    /// Return the focus end relative to `text`.
    pub const fn relative_focus_end(&self) -> u32 {
        self.focus_end.saturating_sub(self.content_start)
    }
}

/// One complete source replacement in an [`AiEditPlan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiEditOperation {
    /// Authored span to replace.
    pub location: SourceLocation,
    /// Complete replacement source; never a byte-truncated fragment.
    pub replacement: String,
}

/// One opaque, provider-neutral verification request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiVerificationStep {
    /// Exact command or check supplied by the rule.
    ///
    /// Consumers must treat this as untrusted data and require explicit
    /// authorization before execution.
    pub command: String,
    /// Expected successful process exit code. Defaults to zero.
    pub expected_exit_code: i32,
}

/// Safety-classified edit and verification plan for one finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiEditPlan {
    /// Packet-local stable plan identifier.
    pub id: String,
    /// Finding this plan addresses.
    pub finding_id: String,
    /// Whether edits are safe, review-required, or unavailable.
    pub safety: FixSafety,
    /// User-visible fix title or explicit unavailable reason.
    pub title: String,
    /// Complete, deterministically ordered edit operations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<AiEditOperation>,
    /// Post-edit verification requests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification: Vec<AiVerificationStep>,
}

/// Versioned, deterministic context shared by every AI connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiContextPacket {
    pub(super) format_version: u32,
    pub(super) report_format_version: u32,
    pub(super) scoring_version: u32,
    pub(super) workspace: String,
    pub(super) budget: AiContextBudget,
    pub(super) findings: Vec<AiFindingContext>,
    pub(super) evidence_graph: AiEvidenceGraph,
    pub(super) source_snippets: Vec<AiSourceSnippet>,
    pub(super) edit_plans: Vec<AiEditPlan>,
    pub(super) omissions: AiContextOmissions,
}

impl AiContextPacket {
    /// Return the serialized AI context format.
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Return the source Doctor report format.
    pub const fn report_format_version(&self) -> u32 {
        self.report_format_version
    }

    /// Return the source Doctor scoring version.
    pub const fn scoring_version(&self) -> u32 {
        self.scoring_version
    }

    /// Return the stable workspace identifier.
    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    /// Return the construction limits recorded with the packet.
    pub const fn budget(&self) -> AiContextBudget {
        self.budget
    }

    /// Return findings in deterministic Doctor rank order.
    pub fn findings(&self) -> &[AiFindingContext] {
        &self.findings
    }

    /// Return the evidence graph for retained findings.
    pub const fn evidence_graph(&self) -> &AiEvidenceGraph {
        &self.evidence_graph
    }

    /// Return source snippets in first-reference order.
    pub fn source_snippets(&self) -> &[AiSourceSnippet] {
        &self.source_snippets
    }

    /// Return safety-classified edit plans in finding order.
    pub fn edit_plans(&self) -> &[AiEditPlan] {
        &self.edit_plans
    }

    /// Return explicit omission accounting.
    pub const fn omissions(&self) -> &AiContextOmissions {
        &self.omissions
    }
}
