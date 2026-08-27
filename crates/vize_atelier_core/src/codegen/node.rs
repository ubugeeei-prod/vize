//! Node generation functions.

use crate::TemplateChildNode;

use super::children::{generate_comment, generate_interpolation, generate_text};
use super::context::CodegenContext;
use super::element::generate_element;
use super::expression::generate_compound_expression;
use super::v_for::generate_for;
use super::v_if::generate_if;
use vize_s0::{ToCompactString, ensure_sufficient_stack};

/// Generate node code
///
/// Every codegen path that emits a child funnels back through here, so this is
/// the one point whose call depth tracks template nesting depth. The guard moves
/// the descent onto a fresh stack before the thread stack runs out, because a
/// stack overflow aborts the process instead of producing a diagnostic
/// (`vize_s0::recursion`).
pub fn generate_node(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    crate::walk_probe::record_visit(crate::walk_probe::WalkStage::Codegen);
    ensure_sufficient_stack(|| match node {
        TemplateChildNode::Element(el) => generate_element(ctx, el),
        TemplateChildNode::Text(text) => generate_text(ctx, text),
        TemplateChildNode::Comment(comment) => generate_comment(ctx, comment),
        TemplateChildNode::Interpolation(interp) => generate_interpolation(ctx, interp),
        TemplateChildNode::CompoundExpression(compound) => {
            generate_compound_expression(ctx, compound)
        }
        TemplateChildNode::If(if_node) => generate_if(ctx, if_node),
        TemplateChildNode::For(for_node) => generate_for(ctx, for_node),
        TemplateChildNode::Hoisted(index) => {
            // Output reference to hoisted variable
            ctx.push("_hoisted_");
            ctx.push(&(index + 1).to_compact_string());
        }
        _ => {
            ctx.push("null /* unsupported node */");
        }
    });
}
