use crate::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

use super::super::context::CodegenContext;
use super::super::expression::generate_expression;
use super::super::helpers::{escape_js_string, is_valid_js_identifier};

pub(super) fn component_root_slot<'a, 'b>(
    el: &'b ElementNode<'a>,
) -> Option<&'b DirectiveNode<'a>> {
    el.props.iter().find_map(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name == "slot"
        {
            return Some(dir.as_ref());
        }
        None
    })
}

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

pub(super) fn emit_slot_property_name(
    ctx: &mut CodegenContext,
    slot_dir: &DirectiveNode<'_>,
    slot_name: &str,
    is_dynamic: bool,
) {
    if is_dynamic {
        ctx.push("[");
        if let Some(arg) = &slot_dir.arg {
            generate_expression(ctx, arg);
        }
        ctx.push("]");
    } else if is_valid_js_identifier(slot_name) {
        ctx.push(slot_name);
    } else {
        ctx.push("\"");
        ctx.push(&escape_js_string(slot_name));
        ctx.push("\"");
    }
}
