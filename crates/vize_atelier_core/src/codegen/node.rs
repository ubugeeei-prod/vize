//! Node generation functions.

use crate::TemplateChildNode;
use crate::relief_projection::ReliefRenderOp;

use super::children::{generate_comment, generate_text};
use super::context::CodegenContext;
use super::element::generate_element;
use super::interpolation::emit_interpolation_value;
use super::v_for::generate_for_from_relief_op;
use super::v_if::generate_if;
use vize_carton::ToCompactString;

/// Generate node code.
///
/// Classifies the node through the Relief projection render-IR
/// ([`ReliefRenderOp::from_template_child`]) and dispatches via [`dispatch_relief_op`].
/// Output is byte-for-byte unchanged.
pub fn generate_node(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    dispatch_relief_op(ctx, ReliefRenderOp::from_template_child(node), node);
}

/// Emit one already-classified Relief projection operation.
///
/// Split from [`generate_node`] so a caller that already holds the op — a child
/// iterator walking the Relief projection stream — can dispatch without re-running
/// [`ReliefRenderOp::from_template_child`]. This is the seam for driving structural
/// emission from the Relief projection stream (#1756): the concrete Relief `node` is
/// still read for the details Relief projection does not yet carry (element props, hoist
/// bodies, control-flow subtrees), so output is byte-for-byte identical to
/// matching the op inline.
pub(crate) fn dispatch_relief_op<'a>(
    ctx: &mut CodegenContext,
    op: ReliefRenderOp<'a>,
    node: &'a TemplateChildNode<'a>,
) {
    match op {
        ReliefRenderOp::Element { .. } => {
            if let TemplateChildNode::Element(el) = node {
                generate_element(ctx, el);
            }
        }
        // Text and comments are now emitted entirely from the Relief projection operation — the
        // Relief node is no longer read for these.
        ReliefRenderOp::Text { content, span } => {
            generate_text(ctx, content, &span.start);
        }
        ReliefRenderOp::Comment {
            content,
            is_directive,
            span,
        } => {
            // `@vize:` directive comments are stripped from build output.
            if !is_directive {
                generate_comment(ctx, content, &span.start);
            }
        }
        // Interpolations emit from the Relief projection operation's expression node and raw flag.
        ReliefRenderOp::Interpolation {
            expression, raw, ..
        } => {
            if let Some(content) = expression.node() {
                emit_interpolation_value(ctx, content, raw);
            }
        }
        ReliefRenderOp::If { .. } => {
            if let TemplateChildNode::If(if_node) = node {
                generate_if(ctx, if_node);
            }
        }
        op @ ReliefRenderOp::For { .. } => {
            if let TemplateChildNode::For(for_node) = node {
                generate_for_from_relief_op(ctx, op, for_node);
            }
        }
        ReliefRenderOp::HoistRef { index } => {
            // Output reference to hoisted variable
            ctx.push("_hoisted_");
            ctx.push(&(index + 1).to_compact_string());
        }
        _ => {
            ctx.push("null /* unsupported node */");
        }
    }
}
