use std::collections::{BTreeMap, BTreeSet};

use vize_s0::{String, cstr};

use crate::{DoctorFinding, DoctorReport, SourceLocation};

use super::{
    contract::{
        AiContextBudget, AiContextOmissions, AiContextPacket, AiEditOperation, AiEditPlan,
        AiEvidenceEdge, AiEvidenceGraph, AiEvidenceNode, AiEvidenceNodeKind, AiEvidenceRelation,
        AiFindingContext, AiSourceSnippet, AiVerificationStep, DOCTOR_AI_CONTEXT_FORMAT_VERSION,
    },
    snippet::extract_source_snippet,
    validation::AiContextError,
};

/// Build a deterministic, provider-neutral AI context packet.
///
/// `sources` maps workspace-relative paths to complete UTF-8 source text. The
/// builder never reads the filesystem, environment, or terminal, which lets
/// editors, CI workers, remote caches, and in-memory analyzers supply source
/// through the same API. Duplicate paths are rejected rather than resolved by
/// input order.
pub fn build_ai_context<'a>(
    report: &DoctorReport,
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
    budget: AiContextBudget,
) -> Result<AiContextPacket, AiContextError> {
    let mut source_by_path = BTreeMap::new();
    for (path, source) in sources {
        if source_by_path.insert(path, source).is_some() {
            return Err(AiContextError::DuplicateSourcePath(path.into()));
        }
    }

    let max_findings = runtime_limit(budget.max_findings);
    let mut builder = PacketBuilder::new(report, source_by_path, budget);
    for (rank, finding) in report.findings().iter().take(max_findings).enumerate() {
        builder.add_finding(rank, finding);
    }
    builder.omissions.dropped_findings = report
        .findings()
        .len()
        .saturating_sub(max_findings)
        .try_into()
        .unwrap_or(u64::MAX);
    builder.finish()
}

struct PacketBuilder<'a> {
    report: &'a DoctorReport,
    source_by_path: BTreeMap<&'a str, &'a str>,
    budget: AiContextBudget,
    findings: Vec<AiFindingContext>,
    graph: AiEvidenceGraph,
    snippets: Vec<AiSourceSnippet>,
    snippet_ids: BTreeMap<SourceLocation, String>,
    requested_locations: BTreeSet<SourceLocation>,
    missing_paths: BTreeSet<String>,
    source_bytes: usize,
    edit_plans: Vec<AiEditPlan>,
    edit_bytes: usize,
    verification_bytes: usize,
    omissions: AiContextOmissions,
}

impl<'a> PacketBuilder<'a> {
    fn new(
        report: &'a DoctorReport,
        source_by_path: BTreeMap<&'a str, &'a str>,
        budget: AiContextBudget,
    ) -> Self {
        Self {
            report,
            source_by_path,
            budget,
            findings: Vec::new(),
            graph: AiEvidenceGraph::default(),
            snippets: Vec::new(),
            snippet_ids: BTreeMap::new(),
            requested_locations: BTreeSet::new(),
            missing_paths: BTreeSet::new(),
            source_bytes: 0,
            edit_plans: Vec::new(),
            edit_bytes: 0,
            verification_bytes: 0,
            omissions: AiContextOmissions::default(),
        }
    }

    fn add_finding(&mut self, rank: usize, finding: &DoctorFinding) {
        let finding_id = cstr!("finding:{rank}");
        let root_id = finding_id.clone();
        self.graph.nodes.push(AiEvidenceNode {
            id: root_id.clone(),
            finding_id: finding_id.clone(),
            kind: AiEvidenceNodeKind::Finding,
            evidence_kind: None,
            summary: finding.title.clone(),
            location: Some(finding.primary.clone()),
            details: BTreeMap::new(),
        });

        let mut source_candidates = vec![finding.primary.clone()];
        self.add_evidence_nodes(&finding_id, finding, &mut source_candidates);
        self.add_related_nodes(&finding_id, finding, &mut source_candidates);
        let edit_plan_id = self.add_edit_plan(&finding_id, finding, &mut source_candidates);

        let mut source_snippet_ids = Vec::new();
        let mut retained_source_ids = BTreeSet::new();
        for location in source_candidates {
            if let Some(id) = self.add_source_snippet(&location)
                && retained_source_ids.insert(id.clone())
            {
                source_snippet_ids.push(id);
            }
        }

        self.findings.push(AiFindingContext {
            id: finding_id,
            baseline_key: finding.baseline_key(),
            code: finding.code.clone(),
            category: finding.category,
            assessment: finding.assessment.clone(),
            primary: finding.primary.clone(),
            title: finding.title.clone(),
            message: finding.message.clone(),
            failure_scenario: finding.failure_scenario.clone(),
            documentation: finding.documentation.clone(),
            context: finding.context.clone(),
            provenance: finding.provenance.clone(),
            suppression: finding.suppression,
            evidence_root_id: root_id,
            source_snippet_ids,
            edit_plan_id,
        });
    }

    fn add_evidence_nodes(
        &mut self,
        finding_id: &String,
        finding: &DoctorFinding,
        source_candidates: &mut Vec<SourceLocation>,
    ) {
        let retained = finding
            .evidence
            .len()
            .min(runtime_limit(self.budget.max_evidence_per_finding));
        add_omission(
            &mut self.omissions.dropped_evidence_nodes,
            finding.evidence.len() - retained,
        );
        for (index, evidence) in finding.evidence.iter().take(retained).enumerate() {
            let node_id = cstr!("{finding_id}:evidence:{index}");
            self.graph.nodes.push(AiEvidenceNode {
                id: node_id.clone(),
                finding_id: finding_id.clone(),
                kind: AiEvidenceNodeKind::Evidence,
                evidence_kind: Some(evidence.kind),
                summary: evidence.summary.clone(),
                location: evidence.location.clone(),
                details: evidence.details.clone(),
            });
            self.graph.edges.push(AiEvidenceEdge {
                from: finding_id.clone(),
                to: node_id,
                relation: AiEvidenceRelation::Supports,
            });
            if let Some(location) = &evidence.location {
                source_candidates.push(location.clone());
            }
        }
    }

    fn add_related_nodes(
        &mut self,
        finding_id: &String,
        finding: &DoctorFinding,
        source_candidates: &mut Vec<SourceLocation>,
    ) {
        let retained = finding
            .related
            .len()
            .min(runtime_limit(self.budget.max_related_per_finding));
        add_omission(
            &mut self.omissions.dropped_related_nodes,
            finding.related.len() - retained,
        );
        for (index, related) in finding.related.iter().take(retained).enumerate() {
            let node_id = cstr!("{finding_id}:related:{index}");
            self.graph.nodes.push(AiEvidenceNode {
                id: node_id.clone(),
                finding_id: finding_id.clone(),
                kind: AiEvidenceNodeKind::RelatedLocation,
                evidence_kind: None,
                summary: related.message.clone(),
                location: Some(related.location.clone()),
                details: BTreeMap::new(),
            });
            self.graph.edges.push(AiEvidenceEdge {
                from: finding_id.clone(),
                to: node_id,
                relation: AiEvidenceRelation::Related,
            });
            source_candidates.push(related.location.clone());
        }
    }

    fn add_edit_plan(
        &mut self,
        finding_id: &String,
        finding: &DoctorFinding,
        source_candidates: &mut Vec<SourceLocation>,
    ) -> Option<String> {
        let fix = finding.fix.as_ref()?;
        let plan_id = cstr!("{finding_id}:edit-plan");
        let mut operations = Vec::new();
        let retained_edits = fix
            .edits
            .len()
            .min(runtime_limit(self.budget.max_edits_per_finding));
        add_omission(
            &mut self.omissions.dropped_edit_operations,
            fix.edits.len() - retained_edits,
        );
        for edit in fix.edits.iter().take(retained_edits) {
            let replacement_bytes = edit.replacement.len();
            if replacement_bytes
                > runtime_limit(self.budget.max_edit_bytes).saturating_sub(self.edit_bytes)
            {
                self.omissions.dropped_edit_operations =
                    self.omissions.dropped_edit_operations.saturating_add(1);
                continue;
            }
            self.edit_bytes += replacement_bytes;
            source_candidates.push(edit.location.clone());
            operations.push(AiEditOperation {
                location: edit.location.clone(),
                replacement: edit.replacement.clone(),
            });
        }

        let mut verification = Vec::new();
        let retained_steps = fix.verification.len().min(runtime_limit(
            self.budget.max_verification_steps_per_finding,
        ));
        add_omission(
            &mut self.omissions.dropped_verification_steps,
            fix.verification.len() - retained_steps,
        );
        for command in fix.verification.iter().take(retained_steps) {
            let command_bytes = command.len();
            if command_bytes
                > self
                    .budget
                    .max_verification_bytes
                    .try_into()
                    .unwrap_or(usize::MAX)
                    .saturating_sub(self.verification_bytes)
            {
                self.omissions.dropped_verification_steps =
                    self.omissions.dropped_verification_steps.saturating_add(1);
                continue;
            }
            self.verification_bytes += command_bytes;
            verification.push(AiVerificationStep {
                command: command.clone(),
                expected_exit_code: 0,
            });
        }

        self.edit_plans.push(AiEditPlan {
            id: plan_id.clone(),
            finding_id: finding_id.clone(),
            safety: fix.safety,
            title: fix.title.clone(),
            operations,
            verification,
        });
        Some(plan_id)
    }

    fn add_source_snippet(&mut self, location: &SourceLocation) -> Option<String> {
        if let Some(id) = self.snippet_ids.get(location) {
            return Some(id.clone());
        }
        if !self.requested_locations.insert(location.clone()) {
            return None;
        }
        let Some(source) = self.source_by_path.get(location.path.as_str()).copied() else {
            self.missing_paths.insert(location.path.clone());
            return None;
        };
        if self.snippets.len() >= runtime_limit(self.budget.max_source_snippets) {
            self.omissions.dropped_source_snippets =
                self.omissions.dropped_source_snippets.saturating_add(1);
            return None;
        }
        let remaining =
            runtime_limit(self.budget.max_source_bytes).saturating_sub(self.source_bytes);
        let max_bytes = remaining.min(runtime_limit(self.budget.max_source_bytes_per_snippet));
        let id = cstr!("source:{}", self.snippets.len());
        let Some(snippet) = extract_source_snippet(id.clone(), location, source, max_bytes) else {
            self.omissions.dropped_source_snippets =
                self.omissions.dropped_source_snippets.saturating_add(1);
            return None;
        };
        self.source_bytes += snippet.text.len();
        self.snippet_ids.insert(location.clone(), id.clone());
        self.snippets.push(snippet);
        Some(id)
    }

    fn finish(mut self) -> Result<AiContextPacket, AiContextError> {
        self.omissions.missing_source_paths = self.missing_paths.into_iter().collect();
        let packet = AiContextPacket {
            format_version: DOCTOR_AI_CONTEXT_FORMAT_VERSION,
            report_format_version: self.report.format_version(),
            scoring_version: self.report.scoring_version(),
            workspace: self.report.workspace().into(),
            budget: self.budget,
            findings: self.findings,
            evidence_graph: self.graph,
            source_snippets: self.snippets,
            edit_plans: self.edit_plans,
            omissions: self.omissions,
        };
        packet.validate()?;
        Ok(packet)
    }
}

fn runtime_limit(limit: u64) -> usize {
    limit.try_into().unwrap_or(usize::MAX)
}

fn add_omission(total: &mut u64, count: usize) {
    *total = total.saturating_add(count.try_into().unwrap_or(u64::MAX));
}
