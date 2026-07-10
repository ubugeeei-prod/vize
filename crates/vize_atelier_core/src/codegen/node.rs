//! Node generation functions.

use crate::TemplateChildNode;
use crate::rendu::RenduOp;

use super::children::{generate_comment, generate_text};
use super::context::CodegenContext;
use super::element::generate_element;
use super::interpolation::emit_interpolation_value;
use super::v_for::generate_for_from_rendu;
use super::v_if::generate_if;
use vize_carton::ToCompactString;

/// Generate node code.
///
/// Classifies the node through the Rendu render-IR
/// ([`RenduOp::from_template_child`]) and dispatches via [`dispatch_rendu_op`].
/// Output is byte-for-byte unchanged.
pub fn generate_node(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    dispatch_rendu_op(ctx, RenduOp::from_template_child(node), node);
}

/// Emit one already-classified Rendu op.
///
/// Split from [`generate_node`] so a caller that already holds the op — a child
/// iterator walking the Rendu stream — can dispatch without re-running
/// [`RenduOp::from_template_child`]. This is the seam for driving structural
/// emission from the Rendu op stream (#1756): the concrete Relief `node` is
/// still read for the details Rendu does not yet carry (element props, hoist
/// bodies, control-flow subtrees), so output is byte-for-byte identical to
/// matching the op inline.
pub(crate) fn dispatch_rendu_op<'a>(
    ctx: &mut CodegenContext,
    op: RenduOp<'a>,
    node: &'a TemplateChildNode<'a>,
) {
    match op {
        RenduOp::Element { .. } => {
            if let TemplateChildNode::Element(el) = node {
                generate_element(ctx, el);
            }
        }
        // Text and comments are now emitted entirely from the Rendu op — the
        // Relief node is no longer read for these.
        RenduOp::Text { content, span } => {
            generate_text(ctx, content, &span.start);
        }
        RenduOp::Comment {
            content,
            is_directive,
            span,
        } => {
            // `@vize:` directive comments are stripped from build output.
            if !is_directive {
                generate_comment(ctx, content, &span.start);
            }
        }
        // Interpolations emit from the Rendu op's expression node and raw flag.
        RenduOp::Interpolation {
            expression, raw, ..
        } => {
            if let Some(content) = expression.node() {
                emit_interpolation_value(ctx, content, raw);
            }
        }
        RenduOp::If { .. } => {
            if let TemplateChildNode::If(if_node) = node {
                generate_if(ctx, if_node);
            }
        }
        op @ RenduOp::For { .. } => {
            if let TemplateChildNode::For(for_node) = node {
                generate_for_from_rendu(ctx, op, for_node);
            }
        }
        RenduOp::HoistRef { index } => {
            // Output reference to hoisted variable
            ctx.push("_hoisted_");
            ctx.push(&(index + 1).to_compact_string());
        }
        _ => {
            ctx.push("null /* unsupported node */");
        }
    }
}
