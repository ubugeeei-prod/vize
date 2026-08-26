use std::collections::{BTreeMap, BTreeSet};

use vize_s0::cstr;

use super::{AiContextError, integrity, runtime_limit};
use crate::ai_context::{
    AiContextPacket, AiEvidenceNode, AiEvidenceNodeKind, AiEvidenceRelation, AiFindingContext,
};

impl AiContextPacket {
    pub(super) fn validate_graph(
        &self,
        finding_ids: &BTreeSet<&str>,
    ) -> Result<(), AiContextError> {
        let mut node_by_id = BTreeMap::new();
        let mut node_counts = BTreeMap::<&str, (usize, usize)>::new();
        for node in &self.evidence_graph.nodes {
            if node.id.is_empty() || node_by_id.insert(node.id.as_str(), node).is_some() {
                return integrity(
                    "AI context contains an empty or duplicate evidence node identifier",
                );
            }
            if !finding_ids.contains(node.finding_id.as_str()) {
                return integrity("AI evidence node references a missing finding");
            }
            match (node.kind, node.evidence_kind) {
                (AiEvidenceNodeKind::Evidence, Some(_)) => {
                    node_counts.entry(node.finding_id.as_str()).or_default().0 += 1;
                }
                (AiEvidenceNodeKind::RelatedLocation, None) => {
                    node_counts.entry(node.finding_id.as_str()).or_default().1 += 1;
                }
                (AiEvidenceNodeKind::Finding, None) => {}
                _ => return integrity("AI evidence node has an invalid evidence domain"),
            }
        }

        self.validate_node_order(&node_counts)?;
        self.validate_graph_edges(&node_by_id)
    }

    fn validate_node_order(
        &self,
        node_counts: &BTreeMap<&str, (usize, usize)>,
    ) -> Result<(), AiContextError> {
        let mut nodes = self.evidence_graph.nodes.iter();
        for finding in &self.findings {
            validate_next_node(&mut nodes, finding, AiEvidenceNodeKind::Finding, None)?;
            let (evidence_count, related_count) = node_counts
                .get(finding.id.as_str())
                .copied()
                .unwrap_or_default();
            if evidence_count > runtime_limit(self.budget.max_evidence_per_finding)
                || related_count > runtime_limit(self.budget.max_related_per_finding)
            {
                return integrity("AI evidence graph exceeds a per-finding node budget");
            }
            for index in 0..evidence_count {
                validate_next_node(
                    &mut nodes,
                    finding,
                    AiEvidenceNodeKind::Evidence,
                    Some(index),
                )?;
            }
            for index in 0..related_count {
                validate_next_node(
                    &mut nodes,
                    finding,
                    AiEvidenceNodeKind::RelatedLocation,
                    Some(index),
                )?;
            }
        }
        if nodes.next().is_some() {
            return integrity("AI evidence graph contains an unexpected node");
        }
        Ok(())
    }

    fn validate_graph_edges(
        &self,
        node_by_id: &BTreeMap<&str, &AiEvidenceNode>,
    ) -> Result<(), AiContextError> {
        let mut edge_targets = BTreeSet::new();
        for edge in &self.evidence_graph.edges {
            if !node_by_id.contains_key(edge.from.as_str()) {
                return integrity("AI evidence edge references a missing node");
            }
            let Some(target) = node_by_id.get(edge.to.as_str()) else {
                return integrity("AI evidence edge references a missing node");
            };
            let expected_relation = match target.kind {
                AiEvidenceNodeKind::Evidence => AiEvidenceRelation::Supports,
                AiEvidenceNodeKind::RelatedLocation => AiEvidenceRelation::Related,
                AiEvidenceNodeKind::Finding => {
                    return integrity("AI evidence edge cannot target a finding root");
                }
            };
            if edge.from != target.finding_id
                || edge.relation != expected_relation
                || !edge_targets.insert(edge.to.as_str())
            {
                return integrity("AI evidence edge does not match its target node");
            }
        }
        let facts = node_by_id
            .iter()
            .filter(|(_, node)| node.kind != AiEvidenceNodeKind::Finding)
            .map(|(id, _)| *id)
            .collect::<BTreeSet<_>>();
        if edge_targets != facts {
            return integrity("AI evidence graph contains an unlinked fact node");
        }
        Ok(())
    }
}

fn validate_next_node<'a>(
    nodes: &mut impl Iterator<Item = &'a AiEvidenceNode>,
    finding: &AiFindingContext,
    kind: AiEvidenceNodeKind,
    index: Option<usize>,
) -> Result<(), AiContextError> {
    let Some(node) = nodes.next() else {
        return integrity("AI evidence graph is missing a deterministic node");
    };
    let expected_id = index.map_or_else(
        || finding.id.clone(),
        |index| match kind {
            AiEvidenceNodeKind::Evidence => cstr!("{}:evidence:{index}", finding.id),
            AiEvidenceNodeKind::RelatedLocation => cstr!("{}:related:{index}", finding.id),
            AiEvidenceNodeKind::Finding => unreachable!("finding roots do not have an index"),
        },
    );
    if node.id != expected_id || node.finding_id != finding.id || node.kind != kind {
        return integrity("AI evidence graph nodes are not deterministically ordered");
    }
    Ok(())
}
