use crate::{DirectiveNode, ExpressionNode};

use super::super::context::CodegenContext;
use super::super::expression::generate_expression;
use super::super::helpers::escape_js_string;

pub(super) fn generate_slot_entry_name(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    slot_name: &str,
) {
    match dir.arg.as_ref() {
        Some(arg @ ExpressionNode::Simple(exp)) if !exp.is_static => generate_expression(ctx, arg),
        Some(arg @ ExpressionNode::Compound(_)) => generate_expression(ctx, arg),
        _ => {
            ctx.push("\"");
            ctx.push(&escape_js_string(slot_name));
            ctx.push("\"");
        }
    }
}
