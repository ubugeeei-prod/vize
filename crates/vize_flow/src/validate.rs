use crate::{DataEdgeKind, FlowGraph, InvariantViolation, ValidationErrors};

impl FlowGraph {
    /// Audit graph arenas, block adjacency, definitions, and effect endpoints.
    ///
    /// Checked construction keeps these invariants true. This explicit audit
    /// is useful after future decoding or bulk-construction paths are added.
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();
        for (index, block) in self.blocks.iter().enumerate() {
            if block.id.index() != index {
                errors.push(InvariantViolation::ArenaIdMismatch("block", index as u32));
            }
        }
        for (index, node) in self.nodes.iter().enumerate() {
            if node.id.index() != index {
                errors.push(InvariantViolation::ArenaIdMismatch("node", index as u32));
            }
            let occurrences = self
                .blocks
                .get(node.block.index())
                .map(|block| block.nodes.iter().filter(|id| **id == node.id).count())
                .unwrap_or(0);
            if occurrences != 1 {
                errors.push(InvariantViolation::NodeOwnership(node.id));
            }
        }
        self.validate_control_adjacency(&mut errors);
        self.validate_value_definitions(&mut errors);
        for edge in &self.effect_edges {
            if self.effects.get(edge.from.index()).is_none()
                || self.effects.get(edge.to.index()).is_none()
            {
                errors.push(InvariantViolation::EffectEndpoint(edge.id.raw()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::new(errors))
        }
    }

    fn validate_control_adjacency(&self, errors: &mut Vec<InvariantViolation>) {
        for (index, edge) in self.control_edges.iter().enumerate() {
            if edge.id.index() != index {
                errors.push(InvariantViolation::ArenaIdMismatch(
                    "control edge",
                    index as u32,
                ));
            }
            let outgoing = self
                .blocks
                .get(edge.from.index())
                .map(|block| block.outgoing.iter().filter(|id| **id == edge.id).count())
                .unwrap_or(0);
            if outgoing != 1 {
                errors.push(InvariantViolation::ControlAdjacency(
                    "outgoing",
                    edge.id.raw(),
                ));
            }
            let incoming = self
                .blocks
                .get(edge.to.index())
                .map(|block| block.incoming.iter().filter(|id| **id == edge.id).count())
                .unwrap_or(0);
            if incoming != 1 {
                errors.push(InvariantViolation::ControlAdjacency(
                    "incoming",
                    edge.id.raw(),
                ));
            }
        }
    }

    fn validate_value_definitions(&self, errors: &mut Vec<InvariantViolation>) {
        for (index, value) in self.values.iter().enumerate() {
            if value.id.index() != index {
                errors.push(InvariantViolation::ArenaIdMismatch("value", index as u32));
            }
            let definitions: Vec<_> = self
                .data_edges
                .iter()
                .filter(|edge| edge.value == value.id && edge.kind == DataEdgeKind::Definition)
                .collect();
            let agrees = match value.definition {
                Some(node) => definitions.len() == 1 && definitions[0].node == node,
                None => definitions.is_empty(),
            };
            if !agrees {
                errors.push(InvariantViolation::ValueDefinition(value.id));
            }
        }
    }
}
