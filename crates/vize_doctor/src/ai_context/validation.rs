use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Deserializer, de};
use vize_s0::{String, cstr};

use crate::{DOCTOR_REPORT_FORMAT_VERSION, DOCTOR_SCORING_VERSION};

use super::contract::{
    AiContextBudget, AiContextOmissions, AiContextPacket, AiEditPlan, AiEvidenceGraph,
    AiFindingContext, AiSourceSnippet, DOCTOR_AI_CONTEXT_FORMAT_VERSION,
};

mod graph;

/// Failure to build or decode a provider-neutral AI context packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiContextError {
    /// More than one source input used the same workspace-relative path.
    DuplicateSourcePath(String),
    /// The serialized AI context format is not supported by this build.
    UnsupportedFormatVersion {
        /// Format supported by this build.
        expected: u32,
        /// Format found in the packet.
        actual: u32,
    },
    /// Cross-reference, ordering, or budget metadata is inconsistent.
    Integrity(String),
}

impl fmt::Display for AiContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSourcePath(path) => {
                write!(formatter, "duplicate AI context source path: {path}")
            }
            Self::UnsupportedFormatVersion { expected, actual } => write!(
                formatter,
                "unsupported AI context format version {actual}; expected {expected}"
            ),
            Self::Integrity(message) => formatter.write_str(message),
        }
    }
}

impl Error for AiContextError {}

impl AiContextPacket {
    /// Validate versions, packet-local references, deterministic identity, and budgets.
    ///
    /// Deserialization calls this automatically. Connectors that persist or
    /// transform packets can call it again before sending data to a provider.
    pub fn validate(&self) -> Result<(), AiContextError> {
        if self.format_version != DOCTOR_AI_CONTEXT_FORMAT_VERSION {
            return Err(AiContextError::UnsupportedFormatVersion {
                expected: DOCTOR_AI_CONTEXT_FORMAT_VERSION,
                actual: self.format_version,
            });
        }
        if self.report_format_version != DOCTOR_REPORT_FORMAT_VERSION {
            return integrity("AI context references an unsupported Doctor report format");
        }
        if self.scoring_version != DOCTOR_SCORING_VERSION {
            return integrity("AI context references an unsupported Doctor scoring version");
        }
        if self.findings.len() > runtime_limit(self.budget.max_findings) {
            return integrity("AI context exceeds its finding budget");
        }

        let finding_ids = unique_ids(
            self.findings.iter().map(|finding| finding.id.as_str()),
            "finding",
        )?;
        for (rank, finding) in self.findings.iter().enumerate() {
            if finding.id != cstr!("finding:{rank}") {
                return integrity("AI context finding identifiers do not match deterministic rank");
            }
        }
        self.validate_graph(&finding_ids)?;
        let snippet_ids = self.validate_snippets()?;
        let plan_by_id = self.validate_edit_plans(&finding_ids)?;
        let plan_ids = plan_by_id.keys().copied().collect::<BTreeSet<_>>();
        let mut referenced_snippets = BTreeSet::new();
        let mut referenced_plans = BTreeSet::new();

        for finding in &self.findings {
            if !finding_ids.contains(finding.evidence_root_id.as_str()) {
                return integrity("AI finding references a missing evidence root");
            }
            let mut local_snippets = BTreeSet::new();
            for snippet_id in &finding.source_snippet_ids {
                if !snippet_ids.contains(snippet_id.as_str()) {
                    return integrity("AI finding references a missing source snippet");
                }
                if !local_snippets.insert(snippet_id.as_str()) {
                    return integrity("AI finding repeats a source snippet reference");
                }
                referenced_snippets.insert(snippet_id.as_str());
            }
            if let Some(plan_id) = &finding.edit_plan_id {
                let Some(plan) = plan_by_id.get(plan_id.as_str()) else {
                    return integrity("AI finding references a missing edit plan");
                };
                if plan.finding_id != finding.id || !referenced_plans.insert(plan_id.as_str()) {
                    return integrity("AI finding references an invalid edit plan");
                }
            }
        }
        if referenced_snippets != snippet_ids {
            return integrity("AI context contains an unreferenced source snippet");
        }
        if referenced_plans != plan_ids {
            return integrity("AI context contains an unreferenced edit plan");
        }
        if !is_sorted_unique(&self.omissions.missing_source_paths) {
            return integrity("AI context missing source paths are not sorted and unique");
        }
        Ok(())
    }

    fn validate_snippets(&self) -> Result<BTreeSet<&str>, AiContextError> {
        if self.source_snippets.len() > runtime_limit(self.budget.max_source_snippets) {
            return integrity("AI context exceeds its source snippet count budget");
        }
        let snippet_ids = unique_ids(
            self.source_snippets
                .iter()
                .map(|snippet| snippet.id.as_str()),
            "source snippet",
        )?;
        let mut source_bytes = 0_usize;
        for (index, snippet) in self.source_snippets.iter().enumerate() {
            if snippet.id != cstr!("source:{index}") {
                return integrity("AI source snippet identifiers are not deterministic");
            }
            if snippet.text.len() > runtime_limit(self.budget.max_source_bytes_per_snippet) {
                return integrity("AI source snippet exceeds its per-snippet byte budget");
            }
            source_bytes = source_bytes.saturating_add(snippet.text.len());
            if snippet.content_end.saturating_sub(snippet.content_start) as usize
                != snippet.text.len()
            {
                return integrity("AI source snippet byte range does not match its text");
            }
            if snippet.focus_start < snippet.content_start
                || snippet.focus_end < snippet.focus_start
                || snippet.focus_end > snippet.content_end
            {
                return integrity("AI source snippet focus is outside its content range");
            }
        }
        if source_bytes > runtime_limit(self.budget.max_source_bytes) {
            return integrity("AI context exceeds its total source byte budget");
        }
        Ok(snippet_ids)
    }

    fn validate_edit_plans<'a>(
        &'a self,
        finding_ids: &BTreeSet<&str>,
    ) -> Result<BTreeMap<&'a str, &'a AiEditPlan>, AiContextError> {
        let mut plan_by_id = BTreeMap::new();
        let mut plan_findings = BTreeSet::new();
        let mut edit_bytes = 0_usize;
        let mut verification_bytes = 0_usize;
        for plan in &self.edit_plans {
            if plan.id.is_empty() || plan_by_id.insert(plan.id.as_str(), plan).is_some() {
                return integrity("AI context contains an empty or duplicate edit plan identifier");
            }
            if !finding_ids.contains(plan.finding_id.as_str()) {
                return integrity("AI edit plan references a missing finding");
            }
            if !plan_findings.insert(plan.finding_id.as_str()) {
                return integrity("AI finding has more than one edit plan");
            }
            if plan.id != cstr!("{}:edit-plan", plan.finding_id) {
                return integrity("AI edit plan identifier does not match its finding");
            }
            if plan.operations.len() > runtime_limit(self.budget.max_edits_per_finding) {
                return integrity("AI edit plan exceeds its operation count budget");
            }
            if plan.verification.len()
                > runtime_limit(self.budget.max_verification_steps_per_finding)
            {
                return integrity("AI edit plan exceeds its verification count budget");
            }
            edit_bytes = plan.operations.iter().fold(edit_bytes, |total, operation| {
                total.saturating_add(operation.replacement.len())
            });
            verification_bytes = plan
                .verification
                .iter()
                .fold(verification_bytes, |total, step| {
                    total.saturating_add(step.command.len())
                });
        }
        if edit_bytes > runtime_limit(self.budget.max_edit_bytes) {
            return integrity("AI context exceeds its replacement-source byte budget");
        }
        if verification_bytes > runtime_limit(self.budget.max_verification_bytes) {
            return integrity("AI context exceeds its verification-command byte budget");
        }
        Ok(plan_by_id)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AiContextPacketWire {
    format_version: u32,
    report_format_version: u32,
    scoring_version: u32,
    workspace: String,
    budget: AiContextBudget,
    findings: Vec<AiFindingContext>,
    evidence_graph: AiEvidenceGraph,
    source_snippets: Vec<AiSourceSnippet>,
    edit_plans: Vec<AiEditPlan>,
    omissions: AiContextOmissions,
}

impl<'de> Deserialize<'de> for AiContextPacket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AiContextPacketWire::deserialize(deserializer)?;
        let packet = Self {
            format_version: wire.format_version,
            report_format_version: wire.report_format_version,
            scoring_version: wire.scoring_version,
            workspace: wire.workspace,
            budget: wire.budget,
            findings: wire.findings,
            evidence_graph: wire.evidence_graph,
            source_snippets: wire.source_snippets,
            edit_plans: wire.edit_plans,
            omissions: wire.omissions,
        };
        packet.validate().map_err(de::Error::custom)?;
        Ok(packet)
    }
}

fn unique_ids<'a>(
    ids: impl IntoIterator<Item = &'a str>,
    kind: &str,
) -> Result<BTreeSet<&'a str>, AiContextError> {
    let mut unique = BTreeSet::new();
    for id in ids {
        if id.is_empty() || !unique.insert(id) {
            return integrity(cstr!(
                "AI context contains an empty or duplicate {kind} identifier"
            ));
        }
    }
    Ok(unique)
}

fn is_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

pub(super) fn runtime_limit(limit: u64) -> usize {
    limit.try_into().unwrap_or(usize::MAX)
}

pub(super) fn integrity<T>(message: impl Into<String>) -> Result<T, AiContextError> {
    Err(AiContextError::Integrity(message.into()))
}
