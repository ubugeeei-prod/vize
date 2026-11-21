//! Local control-flow scoring shared by standalone and composed traversals.
//! Expand in the owner so the standalone hot path retains its original lowering.

macro_rules! structure {
    ($span:expr, $kind:expr, $id:ident, $depth:ident, $facts:ident) => {
        $facts.push($crate::analysis::template_complexity::Contribution {
            span: $span,
            kind: $kind,
            op: $id,
            nesting: $depth,
            cyclomatic: 1,
            cognitive: 1 + $depth,
        })
    };
}
pub(super) use structure;

macro_rules! branch {
    ($branch:ident, $index:ident, $id:ident, $depth:ident, $facts:ident) => {
        match $branch.condition {
            Some(condition) => {
                let (kind, cognitive) = if $index == 0 {
                    (
                        $crate::analysis::template_complexity::DecisionKind::If,
                        1 + $depth,
                    )
                } else {
                    (
                        $crate::analysis::template_complexity::DecisionKind::ElseIf,
                        1,
                    )
                };
                $facts.push($crate::analysis::template_complexity::Contribution {
                    span: condition.span(),
                    kind,
                    op: $id,
                    nesting: $depth,
                    cyclomatic: 1,
                    cognitive,
                });
                // A condition evaluates before its branch is entered.
                score(condition, $id, $depth, $facts);
            }
            None => $facts.push($crate::analysis::template_complexity::Contribution {
                span: $branch.span,
                kind: $crate::analysis::template_complexity::DecisionKind::Else,
                op: $id,
                nesting: $depth,
                cyclomatic: 0,
                cognitive: 1,
            }),
        }
    };
}
pub(super) use branch;

#[cfg(any(test, feature = "composed-template-analysis"))]
use super::{ComplexityFacts, DecisionKind, NodeId, score};

#[cfg(any(test, feature = "composed-template-analysis"))]
#[inline(always)]
pub(in crate::analysis) fn score_structure(
    span: vize_l0::Span,
    kind: DecisionKind,
    id: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    structure!(span, kind, id, depth, facts);
}

#[cfg(any(test, feature = "composed-template-analysis"))]
#[inline(always)]
pub(in crate::analysis) fn score_branch(
    branch: &crate::op::IfBranch<'_>,
    index: usize,
    id: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    branch!(branch, index, id, depth, facts);
}
