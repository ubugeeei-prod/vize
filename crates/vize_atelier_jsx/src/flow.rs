//! JSX-owned syntax to frontend-neutral control/data/effect flow.

use vize_carton::source_range::SourceRange;
use vize_flow::{
    BlockId, ControlEdgeKind, DataUseKind, EffectKind, FlowGraph, FlowResult, NodeKind, Provenance,
    SourceId, TerminatorKind,
};

use crate::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxBinding, JsxSyntaxExpression,
    JsxSyntaxNode, JsxSyntaxSnapshot, JsxSyntaxSpan,
};

/// Project an owned JSX syntax product into the shared Flow representation.
pub fn project_jsx_syntax_to_flow(snapshot: &JsxSyntaxSnapshot) -> FlowResult<FlowGraph> {
    let mut graph = FlowGraph::new();
    let name = snapshot.filename.as_deref().unwrap_or("<jsx>");
    let source = match snapshot.source_anchor {
        Some(anchor) => graph.add_source_with_anchor(name, anchor)?,
        None => graph.add_source(name)?,
    };
    let end = u32::try_from(snapshot.source.len()).unwrap_or(u32::MAX);
    graph.set_block_provenance(
        graph.entry_block(),
        Provenance::source(source, SourceRange::new(0, end)),
    )?;
    let entry = graph.entry_block();
    let mut lowerer = FlowLowerer { graph, source };
    lowerer.lower_nodes(entry, &snapshot.roots)?;
    debug_assert!(lowerer.graph.validate().is_ok());
    Ok(lowerer.graph)
}

struct FlowLowerer {
    graph: FlowGraph,
    source: SourceId,
}

impl FlowLowerer {
    fn lower_nodes(&mut self, mut block: BlockId, nodes: &[JsxSyntaxNode]) -> FlowResult<BlockId> {
        for node in nodes {
            block = self.lower_node(block, node)?;
        }
        Ok(block)
    }

    fn lower_node(&mut self, block: BlockId, node: &JsxSyntaxNode) -> FlowResult<BlockId> {
        match node {
            JsxSyntaxNode::Element(element) => {
                let operation = self.graph.add_node(
                    block,
                    NodeKind::Operation,
                    self.provenance(element.span),
                )?;
                self.graph.add_effect(
                    operation,
                    if element.component {
                        EffectKind::Call
                    } else {
                        EffectKind::Allocate
                    },
                    None,
                    self.provenance(element.span),
                )?;
                for attribute in &element.attributes {
                    match attribute {
                        JsxSyntaxAttribute::Attribute {
                            value: JsxSyntaxAttributeValue::Expression(expression),
                            ..
                        }
                        | JsxSyntaxAttribute::Spread { expression, .. } => {
                            self.add_expression(block, expression)?;
                        }
                        _ => {}
                    }
                }
                self.lower_nodes(block, &element.children)
            }
            JsxSyntaxNode::Fragment { children, .. } => self.lower_nodes(block, children),
            JsxSyntaxNode::Expression { expression, .. } => {
                self.add_expression(block, expression)?;
                Ok(block)
            }
            JsxSyntaxNode::If { branches, span } => self.lower_if(block, branches, *span),
            JsxSyntaxNode::For {
                source,
                value,
                index,
                body,
                span,
            } => self.lower_for(block, source, value.as_ref(), index.as_ref(), body, *span),
            JsxSyntaxNode::Text { span, .. } | JsxSyntaxNode::Comment { span, .. } => {
                self.graph
                    .add_node(block, NodeKind::Operation, self.provenance(*span))?;
                Ok(block)
            }
        }
    }

    fn lower_if(
        &mut self,
        block: BlockId,
        branches: &[crate::JsxSyntaxBranch],
        span: JsxSyntaxSpan,
    ) -> FlowResult<BlockId> {
        if branches.is_empty() {
            return Ok(block);
        }
        let provenance = self.provenance(span);
        self.graph.add_node(
            block,
            NodeKind::Terminator(TerminatorKind::Branch),
            provenance,
        )?;
        let merge = self.graph.add_block(provenance)?;
        let mut has_else = false;
        for (index, branch) in branches.iter().enumerate() {
            if let Some(condition) = &branch.condition {
                self.add_expression(block, condition)?;
            } else {
                has_else = true;
            }
            let branch_block = self.graph.add_block(self.provenance(branch.span))?;
            self.graph.add_control_edge(
                block,
                branch_block,
                if index == 0 {
                    ControlEdgeKind::TrueBranch
                } else {
                    ControlEdgeKind::FalseBranch
                },
                self.provenance(branch.span),
            )?;
            let end = self.lower_nodes(branch_block, &branch.body)?;
            self.graph
                .add_control_edge(end, merge, ControlEdgeKind::Normal, provenance)?;
        }
        if !has_else {
            self.graph
                .add_control_edge(block, merge, ControlEdgeKind::FalseBranch, provenance)?;
        }
        Ok(merge)
    }

    fn lower_for(
        &mut self,
        block: BlockId,
        source: &JsxSyntaxExpression,
        value: Option<&JsxSyntaxBinding>,
        index: Option<&JsxSyntaxBinding>,
        body: &[JsxSyntaxNode],
        span: JsxSyntaxSpan,
    ) -> FlowResult<BlockId> {
        let provenance = self.provenance(span);
        let header = self.graph.add_block(provenance)?;
        self.graph
            .add_control_edge(block, header, ControlEdgeKind::Normal, provenance)?;
        self.add_expression(header, source)?;
        self.graph.add_node(
            header,
            NodeKind::Terminator(TerminatorKind::Branch),
            provenance,
        )?;
        let body_block = self.graph.add_block(provenance)?;
        let exit = self.graph.add_block(provenance)?;
        self.graph
            .add_control_edge(header, body_block, ControlEdgeKind::TrueBranch, provenance)?;
        self.graph
            .add_control_edge(header, exit, ControlEdgeKind::FalseBranch, provenance)?;
        for binding in [value, index].into_iter().flatten() {
            self.add_binding(body_block, binding)?;
        }
        let body_end = self.lower_nodes(body_block, body)?;
        self.graph
            .add_control_edge(body_end, header, ControlEdgeKind::LoopBack, provenance)?;
        Ok(exit)
    }

    fn add_binding(&mut self, block: BlockId, binding: &JsxSyntaxBinding) -> FlowResult<()> {
        let provenance = self.provenance(binding.span);
        let node = self
            .graph
            .add_node(block, NodeKind::Operation, provenance)?;
        let symbol = self.graph.add_symbol(provenance)?;
        let value = self.graph.define_value(node, Some(symbol), provenance)?;
        self.graph
            .add_data_use(value, node, DataUseKind::Use, provenance)?;
        Ok(())
    }

    fn add_expression(
        &mut self,
        block: BlockId,
        expression: &JsxSyntaxExpression,
    ) -> FlowResult<()> {
        let provenance = self.provenance(expression.span);
        let node = self
            .graph
            .add_node(block, NodeKind::Operation, provenance)?;
        self.graph
            .add_effect(node, expression_effect(&expression.code), None, provenance)?;
        Ok(())
    }

    const fn provenance(&self, span: JsxSyntaxSpan) -> Provenance {
        Provenance::source(self.source, SourceRange::new(span.start, span.end))
    }
}

fn expression_effect(code: &str) -> EffectKind {
    if code.contains("await ") {
        EffectKind::Await
    } else if code.contains('=') {
        EffectKind::Write
    } else if code.contains('(') {
        EffectKind::Call
    } else {
        EffectKind::Read
    }
}
