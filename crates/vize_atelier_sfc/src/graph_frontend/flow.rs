use vize_carton::source_anchor::SourceAnchor;
use vize_flow::{
    BlockId, ControlEdgeKind, EffectKind, FlowGraph, NodeKind, Provenance, SourceId, TerminatorKind,
};
use vize_relief::{
    ElementType, ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId, SnapshotExpression,
    SnapshotFor, SnapshotIf, SnapshotIfBranch, SnapshotProp, SnapshotTextCallContent,
};

use super::{
    SfcGraphAdapterError,
    expression::{compound_code, expression_code},
    flow_control::add_jump,
    flow_facts::FlowFacts,
    provenance::{add_flow_source, flow_provenance},
};

/// Project a cached Relief syntax product into single-unit control/data/effect flow.
pub fn project_relief_snapshot_to_flow(
    snapshot: &ReliefSnapshot,
) -> Result<FlowGraph, SfcGraphAdapterError> {
    FlowProjector::new(snapshot, None)?.project()
}

pub(crate) fn project_relief_snapshot_to_flow_with_anchor(
    snapshot: &ReliefSnapshot,
    anchor: SourceAnchor,
) -> Result<FlowGraph, SfcGraphAdapterError> {
    FlowProjector::new(snapshot, Some(anchor))?.project()
}

struct FlowProjector<'a> {
    snapshot: &'a ReliefSnapshot,
    graph: FlowGraph,
    source: SourceId,
    facts: FlowFacts,
}

impl<'a> FlowProjector<'a> {
    fn new(
        snapshot: &'a ReliefSnapshot,
        anchor: Option<SourceAnchor>,
    ) -> Result<Self, SfcGraphAdapterError> {
        let mut graph = FlowGraph::new();
        let source = add_flow_source(&mut graph, anchor)?;
        graph.set_block_provenance(
            graph.entry_block(),
            flow_provenance(snapshot.location(), source),
        )?;
        Ok(Self {
            snapshot,
            graph,
            source,
            facts: FlowFacts::default(),
        })
    }

    fn project(mut self) -> Result<FlowGraph, SfcGraphAdapterError> {
        let end = self.lower_nodes(self.graph.entry_block(), self.snapshot.children())?;
        let exit = self.graph.add_block(Provenance::Synthetic)?;
        self.graph.add_node(
            end,
            NodeKind::Terminator(TerminatorKind::Return),
            Provenance::Synthetic,
        )?;
        self.graph
            .add_control_edge(end, exit, ControlEdgeKind::Return, Provenance::Synthetic)?;
        self.graph.validate()?;
        Ok(self.graph)
    }

    fn lower_nodes(
        &mut self,
        mut block: BlockId,
        nodes: &[ReliefSnapshotNodeId],
    ) -> Result<BlockId, SfcGraphAdapterError> {
        for id in nodes {
            block = self.lower_node(block, *id)?;
        }
        Ok(block)
    }

    fn lower_node(
        &mut self,
        block: BlockId,
        id: ReliefSnapshotNodeId,
    ) -> Result<BlockId, SfcGraphAdapterError> {
        let snapshot = self.snapshot;
        let node = snapshot
            .node(id)
            .ok_or(SfcGraphAdapterError::MissingSnapshotNode(id))?;
        match node {
            ReliefSnapshotNode::Element(element) => {
                for property in &element.props {
                    self.lower_property_uses(block, property)?;
                }
                let effect = match element.tag_type {
                    ElementType::Component | ElementType::Slot => EffectKind::Call,
                    ElementType::Element | ElementType::Template => EffectKind::Write,
                };
                self.add_effect_node(block, effect, &element.location)?;
                self.lower_nodes(block, element.children())
            }
            ReliefSnapshotNode::Text(text) => {
                self.add_effect_node(block, EffectKind::Write, &text.location)?;
                Ok(block)
            }
            ReliefSnapshotNode::Comment(comment) => {
                self.add_effect_node(block, EffectKind::Write, &comment.location)?;
                Ok(block)
            }
            ReliefSnapshotNode::Interpolation(interpolation) => {
                let node = self.add_expression_use(block, &interpolation.content)?;
                self.graph.add_effect(
                    node,
                    EffectKind::Write,
                    None,
                    flow_provenance(&interpolation.location, self.source),
                )?;
                Ok(block)
            }
            ReliefSnapshotNode::If(if_node) => self.lower_if(block, if_node),
            ReliefSnapshotNode::IfBranch(branch) => self.lower_standalone_branch(block, branch),
            ReliefSnapshotNode::For(for_node) => self.lower_for(block, for_node),
            ReliefSnapshotNode::TextCall(call) => {
                match &call.content {
                    SnapshotTextCallContent::Text(_) => {
                        self.add_effect_node(block, EffectKind::Write, &call.location)?;
                    }
                    SnapshotTextCallContent::Interpolation(interpolation) => {
                        self.add_expression_use(block, &interpolation.content)?;
                    }
                    SnapshotTextCallContent::Compound(compound) => {
                        let code = compound_code(compound);
                        self.add_code_use(block, code, &compound.location)?;
                    }
                }
                Ok(block)
            }
            ReliefSnapshotNode::CompoundExpression(compound) => {
                let code = compound_code(compound);
                self.add_code_use(block, code, &compound.location)?;
                Ok(block)
            }
            ReliefSnapshotNode::Hoisted(hoist) => {
                self.add_effect_node(block, EffectKind::Read, &hoist.location)?;
                Ok(block)
            }
        }
    }

    fn lower_if(
        &mut self,
        block: BlockId,
        node: &SnapshotIf,
    ) -> Result<BlockId, SfcGraphAdapterError> {
        let mut branches = Vec::with_capacity(node.branches().len());
        for id in node.branches() {
            let Some(ReliefSnapshotNode::IfBranch(branch)) = self.snapshot.node(*id) else {
                return Err(SfcGraphAdapterError::ExpectedIfBranch(*id));
            };
            branches.push(branch);
        }
        self.lower_branches(block, &branches, &node.location)
    }

    fn lower_standalone_branch(
        &mut self,
        block: BlockId,
        branch: &SnapshotIfBranch,
    ) -> Result<BlockId, SfcGraphAdapterError> {
        self.lower_branches(block, &[branch], &branch.location)
    }

    fn lower_branches(
        &mut self,
        block: BlockId,
        branches: &[&SnapshotIfBranch],
        location: &vize_relief::SourceLocation,
    ) -> Result<BlockId, SfcGraphAdapterError> {
        let provenance = flow_provenance(location, self.source);
        let merge = self.graph.add_block(provenance)?;
        if branches.is_empty() {
            self.graph
                .add_control_edge(block, merge, ControlEdgeKind::Normal, provenance)?;
            return Ok(merge);
        }
        let mut decision = block;
        for (index, branch) in branches.iter().enumerate() {
            let branch_provenance = flow_provenance(&branch.location, self.source);
            let body = self.graph.add_block(branch_provenance)?;
            if let Some(condition) = &branch.condition {
                self.add_expression_use(decision, condition)?;
                self.graph.add_node(
                    decision,
                    NodeKind::Terminator(TerminatorKind::Branch),
                    branch_provenance,
                )?;
                self.graph.add_control_edge(
                    decision,
                    body,
                    ControlEdgeKind::TrueBranch,
                    branch_provenance,
                )?;
                let otherwise = if index + 1 == branches.len() {
                    merge
                } else {
                    self.graph.add_block(branch_provenance)?
                };
                self.graph.add_control_edge(
                    decision,
                    otherwise,
                    ControlEdgeKind::FalseBranch,
                    branch_provenance,
                )?;
                decision = otherwise;
            } else {
                self.graph.add_control_edge(
                    decision,
                    body,
                    ControlEdgeKind::Normal,
                    branch_provenance,
                )?;
            }
            let end = self.lower_nodes(body, branch.children())?;
            add_jump(
                &mut self.graph,
                end,
                merge,
                ControlEdgeKind::Normal,
                branch_provenance,
            )?;
        }
        Ok(merge)
    }

    fn lower_for(
        &mut self,
        block: BlockId,
        node: &SnapshotFor,
    ) -> Result<BlockId, SfcGraphAdapterError> {
        let provenance = flow_provenance(&node.location, self.source);
        let header = self.graph.add_block(provenance)?;
        add_jump(
            &mut self.graph,
            block,
            header,
            ControlEdgeKind::Normal,
            provenance,
        )?;
        self.add_expression_use(header, &node.source)?;
        self.graph.add_node(
            header,
            NodeKind::Terminator(TerminatorKind::Branch),
            provenance,
        )?;
        let body = self.graph.add_block(provenance)?;
        let exit = self.graph.add_block(provenance)?;
        self.graph
            .add_control_edge(header, body, ControlEdgeKind::TrueBranch, provenance)?;
        self.graph
            .add_control_edge(header, exit, ControlEdgeKind::FalseBranch, provenance)?;
        for binding in [
            node.value_alias
                .as_ref()
                .or(node.parse_result.value.as_ref()),
            node.key_alias.as_ref().or(node.parse_result.key.as_ref()),
            node.object_index_alias
                .as_ref()
                .or(node.parse_result.index.as_ref()),
        ]
        .into_iter()
        .flatten()
        {
            self.add_binding_definition(body, binding)?;
        }
        let end = self.lower_nodes(body, node.children())?;
        add_jump(
            &mut self.graph,
            end,
            header,
            ControlEdgeKind::LoopBack,
            provenance,
        )?;
        Ok(exit)
    }

    fn lower_property_uses(
        &mut self,
        block: BlockId,
        property: &SnapshotProp,
    ) -> Result<(), SfcGraphAdapterError> {
        let SnapshotProp::Directive(directive) = property else {
            return Ok(());
        };
        if let Some(expression) = &directive.expression {
            self.add_expression_use(block, expression)?;
        }
        if let Some(argument) = &directive.argument
            && !matches!(argument, SnapshotExpression::Simple(simple) if simple.is_static)
        {
            self.add_expression_use(block, argument)?;
        }
        Ok(())
    }

    fn add_expression_use(
        &mut self,
        block: BlockId,
        expression: &SnapshotExpression,
    ) -> Result<vize_flow::NodeId, SfcGraphAdapterError> {
        self.add_code_use(block, expression_code(expression), expression.location())
    }

    fn add_code_use(
        &mut self,
        block: BlockId,
        code: vize_carton::String,
        location: &vize_relief::SourceLocation,
    ) -> Result<vize_flow::NodeId, SfcGraphAdapterError> {
        Ok(self
            .facts
            .add_code_use(&mut self.graph, self.source, block, code, location)?)
    }

    fn add_binding_definition(
        &mut self,
        block: BlockId,
        binding: &SnapshotExpression,
    ) -> Result<(), SfcGraphAdapterError> {
        self.facts
            .add_binding_definition(&mut self.graph, self.source, block, binding)?;
        Ok(())
    }

    fn add_effect_node(
        &mut self,
        block: BlockId,
        effect: EffectKind,
        location: &vize_relief::SourceLocation,
    ) -> Result<(), SfcGraphAdapterError> {
        let provenance = flow_provenance(location, self.source);
        let node = self
            .graph
            .add_node(block, NodeKind::Operation, provenance)?;
        self.graph.add_effect(node, effect, None, provenance)?;
        Ok(())
    }
}
