//! Children as rendered: a JSX conditional or list yields what it renders.

use super::{MarkupElement, MarkupElementInner};
use crate::markup::jsx_names::{jsx_element_ref, jsx_fragment_ref};
use crate::markup::jsx_roots::walk_expression_container_roots;
use crate::markup::l2::L2Markup;
use crate::markup::l2::children::walk_nodes;
use crate::markup::node::MarkupNode;
use oxc_ast::ast::{JSXChild, JSXExpression};
use vize_l2::op::Op;
use vize_relief::TemplateChildNode;

impl<'a> MarkupElement<'a> {
    /// Like [`Self::walk_children`], but a JSX conditional or list yields the
    /// nodes its branches or body render instead of one scope node.
    pub fn walk_rendered_children(&self, visitor: &mut impl FnMut(MarkupNode<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => relief(&node.children, visitor),
            MarkupElementInner::L2 { op, doc, .. } if !doc.is_template() => {
                l2(doc, op.children(), visitor);
            }
            MarkupElementInner::JsxElement { node, offset } => {
                jsx(&jsx_element_ref(node).children, offset, visitor);
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                jsx(&jsx_fragment_ref(node).children, offset, visitor);
            }
            MarkupElementInner::Authored { .. }
            | MarkupElementInner::L2 { .. }
            | MarkupElementInner::L2Carrier { .. } => self.walk_children(visitor),
        }
    }
}

/// A raw template has no scope nodes; a lowered JSX root's `IfNode` /
/// `ForNode` yields its branch and body children.
fn relief<'a>(children: &'a [TemplateChildNode<'a>], visitor: &mut impl FnMut(MarkupNode<'a>)) {
    for child in children {
        match child {
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    relief(&branch.children, visitor);
                }
            }
            TemplateChildNode::For(for_node) => relief(&for_node.children, visitor),
            _ => visitor(MarkupNode::from_relief_child(child)),
        }
    }
}

/// A JSX projection's `ui.if` / `ui.for` yields its regions' nodes.
fn l2<'a>(doc: &'a L2Markup<'a>, ops: &'a [Op<'a>], visitor: &mut dyn FnMut(MarkupNode<'a>)) {
    for op in ops {
        match op {
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    l2(doc, &branch.region.ops, visitor);
                }
            }
            Op::For(for_op) => l2(doc, &for_op.region.ops, visitor),
            Op::OriginalFor(for_op) => l2(doc, &for_op.region.ops, visitor),
            _ => walk_nodes(doc, core::slice::from_ref(op), visitor),
        }
    }
}

/// Direct JSX, conditional and list expressions yield their rendered roots.
fn jsx<'a>(children: &'a [JSXChild<'a>], offset: u32, visitor: &mut impl FnMut(MarkupNode<'a>)) {
    for child in children {
        if let JSXChild::ExpressionContainer(container) = child
            && matches!(
                &container.expression,
                JSXExpression::JSXElement(_) | JSXExpression::JSXFragment(_)
            )
        {
            walk_expression_container_roots(container, offset, &mut |element| {
                visitor(MarkupNode::Element(element));
            });
            continue;
        }
        match MarkupNode::from_jsx_child(child, offset) {
            MarkupNode::If(_) | MarkupNode::For(_) => {
                if let JSXChild::ExpressionContainer(container) = child {
                    walk_expression_container_roots(container, offset, &mut |element| {
                        visitor(MarkupNode::Element(element));
                    });
                }
            }
            node => visitor(node),
        }
    }
}
