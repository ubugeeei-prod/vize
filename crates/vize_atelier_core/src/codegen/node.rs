//! Node generation functions.

use crate::TemplateChildNode;
use crate::rendu::RenduOp;

use super::children::{generate_comment, generate_interpolation, generate_text};
use super::context::CodegenContext;
use super::element::generate_element;
use super::v_for::generate_for;
use super::v_if::generate_if;
use vize_carton::ToCompactString;

/// Generate node code.
///
/// Dispatch flows through the Rendu render-IR classification
/// ([`RenduOp::from_template_child`]) rather than matching Relief variants
/// directly: this is the first seam where DOM codegen consumes Rendu. Output is
/// byte-for-byte unchanged — the concrete Relief node is still read for the
/// details Rendu does not yet carry (props, source positions, hoist bodies).
pub fn generate_node(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    match RenduOp::from_template_child(node) {
        RenduOp::Element { .. } => {
            if let TemplateChildNode::Element(el) = node {
                generate_element(ctx, el);
            }
        }
        RenduOp::Text { .. } => {
            if let TemplateChildNode::Text(text) = node {
                generate_text(ctx, text);
            }
        }
        RenduOp::Comment { .. } => {
            if let TemplateChildNode::Comment(comment) = node {
                generate_comment(ctx, comment);
            }
        }
        RenduOp::Interpolation { .. } => {
            if let TemplateChildNode::Interpolation(interp) = node {
                generate_interpolation(ctx, interp);
            }
        }
        RenduOp::If { .. } => {
            if let TemplateChildNode::If(if_node) = node {
                generate_if(ctx, if_node);
            }
        }
        RenduOp::For { .. } => {
            if let TemplateChildNode::For(for_node) = node {
                generate_for(ctx, for_node);
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
