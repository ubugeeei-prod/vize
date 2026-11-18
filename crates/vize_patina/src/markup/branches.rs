//! Exclusive branches: which conditional alternatives each element renders in.
//!
//! Rules that compare elements across a document (duplicate landmarks,
//! duplicate ids) must not pair elements that never render together. Every
//! backend answers with the scopes the visitor presents: a `v-if` chain (the
//! same chain scan as [`super::relief_scopes`]), a lowered JSX `IfNode`, and
//! an L2 `ui.if` / `ui.and`. A TSRX host loop's body and its `@empty` region
//! are alternatives too.

use super::element::MarkupElement;
use super::l2::children::if_range;
use super::l2::walk::{L2Step, scope_region};
use super::l2::{L2ElementOp, L2Markup};
use super::relief_scopes::{BranchKind, ReliefChain, branch_of};
use super::{MarkupDocument, MarkupDocumentInner, loc_to_range};
use crate::ir::ByteRange;
use vize_l2::op::Op;
use vize_relief::{ElementNode, TemplateChildNode};

/// One alternative an element renders inside: the scope's range and the
/// alternative's index in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkupBranch {
    scope: ByteRange,
    index: usize,
}

impl MarkupBranch {
    /// Whether elements on these two paths can render together: no scope
    /// picks a different alternative for each.
    pub fn can_coexist(left: &[Self], right: &[Self]) -> bool {
        left.iter().all(|choice| {
            right
                .iter()
                .all(|other| choice.scope != other.scope || choice.index == other.index)
        })
    }
}

type Visitor<'v, 'a> = dyn FnMut(MarkupElement<'a>, &[MarkupBranch]) + 'v;

impl<'a> MarkupDocument<'a> {
    /// Walk every element in tree order with the alternatives it renders in,
    /// outermost first.
    ///
    /// A document read straight from the OXC AST has no scopes, so every
    /// element reports an empty path there; branch-sensitive rules lint the
    /// lowered document (`Rule::jsx_needs_lowering`).
    pub fn walk_branched_elements(
        &self,
        visitor: &mut impl FnMut(MarkupElement<'a>, &[MarkupBranch]),
    ) {
        let mut path = Vec::new();
        match self.inner {
            MarkupDocumentInner::Relief(root) => relief(&root.children, &mut path, visitor),
            MarkupDocumentInner::Jsx { .. } => self.walk_elements(&mut |element| {
                visitor(element, &[]);
            }),
            MarkupDocumentInner::L2(doc) => l2(doc, &doc.root.ops, &mut path, visitor),
        }
    }
}

fn relief<'a>(
    children: &'a [TemplateChildNode<'a>],
    path: &mut Vec<MarkupBranch>,
    visitor: &mut Visitor<'_, 'a>,
) {
    let mut index = 0;
    while let Some(child) = children.get(index) {
        match child {
            TemplateChildNode::Element(element)
                if matches!(branch_of(element), Some((BranchKind::If, _))) =>
            {
                let chain = ReliefChain::scan(children, index);
                let scope = chain.range();
                let mut branch = 0;
                chain.walk_branches(&mut |element| {
                    path.push(MarkupBranch {
                        scope,
                        index: branch,
                    });
                    relief_element(element, path, visitor);
                    path.pop();
                    branch += 1;
                });
                index = chain.end;
                continue;
            }
            TemplateChildNode::Element(element) => relief_element(element, path, visitor),
            TemplateChildNode::If(if_node) => {
                let scope = loc_to_range(&if_node.loc);
                for (index, branch) in if_node.branches.iter().enumerate() {
                    path.push(MarkupBranch { scope, index });
                    relief(&branch.children, path, visitor);
                    path.pop();
                }
            }
            TemplateChildNode::For(for_node) => relief(&for_node.children, path, visitor),
            _ => {}
        }
        index += 1;
    }
}

fn relief_element<'a>(
    element: &'a ElementNode<'a>,
    path: &mut Vec<MarkupBranch>,
    visitor: &mut Visitor<'_, 'a>,
) {
    visitor(MarkupElement::new(element), path);
    relief(&element.children, path, visitor);
}

fn l2<'a>(
    doc: &'a L2Markup<'a>,
    ops: &'a [Op<'a>],
    path: &mut Vec<MarkupBranch>,
    visitor: &mut Visitor<'_, 'a>,
) {
    for op in ops {
        match op {
            Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                let Some(element) = L2ElementOp::from_op(op) else {
                    continue;
                };
                let step =
                    L2Step::Element(MarkupElement::from_l2(element, doc), element.children());
                l2_step(doc, step, path, visitor);
            }
            Op::If(if_op) => {
                let scope = if_range(doc, if_op);
                for (index, branch) in if_op.branches.iter().enumerate() {
                    path.push(MarkupBranch { scope, index });
                    let step = scope_region(doc, branch.span, &branch.region.ops);
                    l2_step(doc, step, path, visitor);
                    path.pop();
                }
            }
            Op::And(and) => {
                path.push(MarkupBranch {
                    scope: doc.scope_range(and.span, and.span),
                    index: 0,
                });
                l2_step(
                    doc,
                    scope_region(doc, and.span, &and.region.ops),
                    path,
                    visitor,
                );
                path.pop();
            }
            Op::For(for_op) => {
                let body = scope_region(doc, for_op.span, &for_op.region.ops);
                let Some(empty) = for_op.empty_region() else {
                    l2_step(doc, body, path, visitor);
                    continue;
                };
                let scope = doc.scope_range(for_op.span, for_op.span);
                path.push(MarkupBranch { scope, index: 0 });
                l2_step(doc, body, path, visitor);
                path.pop();
                path.push(MarkupBranch { scope, index: 1 });
                l2(doc, &empty.ops, path, visitor);
                path.pop();
            }
            Op::OriginalFor(for_op) => {
                let step = scope_region(doc, for_op.span, &for_op.region.ops);
                l2_step(doc, step, path, visitor);
            }
            Op::Text(_) | Op::Interpolation(_) | Op::JsxChildValue(_) | Op::Comment(_) => {}
        }
    }
}

fn l2_step<'a>(
    doc: &'a L2Markup<'a>,
    step: L2Step<'a>,
    path: &mut Vec<MarkupBranch>,
    visitor: &mut Visitor<'_, 'a>,
) {
    match step {
        L2Step::Element(element, region) => {
            visitor(element, path);
            l2(doc, region, path, visitor);
        }
        L2Step::Region(region) => l2(doc, region, path, visitor),
    }
}
