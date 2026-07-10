//! Deterministic semantic-expression joins against the peer Flow product.

use vize_carton::{FxHashMap, source_anchor::SourceAnchor, source_range::SourceRange};
use vize_croquis::{CroquisSemanticSnapshot, SemanticTemplateExpressionSnapshot};
use vize_flow::{BlockId, Dominators, FlowGraph, Node, Reachability};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct ExpressionFlow {
    pub(super) block: BlockId,
    pub(super) immediate_dominator: Option<BlockId>,
    pub(super) reachable: bool,
    order: usize,
}

pub(super) struct PlannedExpression<'a> {
    pub(super) original_index: usize,
    pub(super) expression: &'a SemanticTemplateExpressionSnapshot,
    pub(super) flow: Option<ExpressionFlow>,
}

pub(super) struct ExpressionPlan<'a> {
    pub(super) expressions: Vec<PlannedExpression<'a>>,
    pub(super) reachable_block_count: usize,
    pub(super) dominated_block_count: usize,
    pub(super) mapped_expression_count: usize,
    pub(super) unreachable_expression_count: usize,
}

pub(super) fn plan_expressions<'a>(
    semantics: &'a CroquisSemanticSnapshot,
    flow: Option<&FlowGraph>,
) -> ExpressionPlan<'a> {
    let Some(flow) = flow else {
        return ExpressionPlan {
            expressions: source_order(semantics),
            reachable_block_count: 0,
            dominated_block_count: 0,
            mapped_expression_count: 0,
            unreachable_expression_count: 0,
        };
    };
    let reachability = flow.reachability();
    let dominators = flow.dominators();
    let order: FxHashMap<_, _> = flow
        .reverse_postorder()
        .into_iter()
        .enumerate()
        .map(|(index, block)| (block, index))
        .collect();
    let mut expressions: Vec<_> = semantics
        .template_expressions
        .iter()
        .enumerate()
        .map(|(original_index, expression)| PlannedExpression {
            original_index,
            expression,
            flow: match_expression(
                semantics.source_anchor,
                SourceRange::new(expression.range.start, expression.range.end),
                flow,
                &reachability,
                &dominators,
                &order,
            ),
        })
        .collect();
    expressions.sort_by_key(expression_order);
    let mapped_expression_count = expressions
        .iter()
        .filter(|expression| expression.flow.is_some())
        .count();
    let unreachable_expression_count = expressions
        .iter()
        .filter(|expression| expression.flow.is_some_and(|flow| !flow.reachable))
        .count();
    let dominated_block_count = reachability
        .blocks()
        .filter(|block| dominators.immediate_dominator(*block).is_some())
        .count();
    ExpressionPlan {
        expressions,
        reachable_block_count: reachability.len(),
        dominated_block_count,
        mapped_expression_count,
        unreachable_expression_count,
    }
}

fn source_order(semantics: &CroquisSemanticSnapshot) -> Vec<PlannedExpression<'_>> {
    semantics
        .template_expressions
        .iter()
        .enumerate()
        .map(|(original_index, expression)| PlannedExpression {
            original_index,
            expression,
            flow: None,
        })
        .collect()
}

fn expression_order(expression: &PlannedExpression<'_>) -> (u8, usize, usize) {
    match expression.flow {
        Some(flow) if flow.reachable => (0, flow.order, expression.original_index),
        None => (1, expression.original_index, expression.original_index),
        Some(flow) => (2, flow.block.raw() as usize, expression.original_index),
    }
}

fn match_expression(
    semantic_anchor: Option<SourceAnchor>,
    target: SourceRange,
    flow: &FlowGraph,
    reachability: &Reachability,
    dominators: &Dominators,
    order: &FxHashMap<BlockId, usize>,
) -> Option<ExpressionFlow> {
    let semantic_anchor = semantic_anchor?;
    let node = flow
        .nodes()
        .filter_map(|node| candidate(node, semantic_anchor, target, flow))
        .min_by_key(|candidate| candidate.key)
        .map(|candidate| candidate.node)?;
    let block = node.block();
    let reachable = reachability.contains(block);
    Some(ExpressionFlow {
        block,
        immediate_dominator: reachable
            .then(|| dominators.immediate_dominator(block))
            .flatten(),
        reachable,
        order: order.get(&block).copied().unwrap_or(usize::MAX),
    })
}

struct Candidate<'a> {
    key: (u8, u32, u32),
    node: &'a Node,
}

fn candidate<'a>(
    node: &'a Node,
    semantic_anchor: SourceAnchor,
    target: SourceRange,
    flow: &FlowGraph,
) -> Option<Candidate<'a>> {
    let span = node.provenance().span()?;
    let anchor = flow.source(span.source())?.anchor()?;
    if anchor.source() != semantic_anchor.source()
        || anchor.revision() != semantic_anchor.revision()
    {
        return None;
    }
    let range = anchor.resolve_range(span.range());
    let exact = range == target;
    let contains = range.start <= target.start && range.end >= target.end;
    (exact || contains).then_some(Candidate {
        key: (u8::from(!exact), range.len(), node.id().raw()),
        node,
    })
}
