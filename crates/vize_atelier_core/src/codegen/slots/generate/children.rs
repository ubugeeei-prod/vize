//! Slot child and expression spelling.

use crate::{ExpressionNode, RuntimeHelper, TemplateChildNode};
use super::super::super::context::CodegenContext;
use super::super::super::node::generate_node;
use super::slot_params::strip_ctx_prefix_for_slot_params;

/// Generate children for a slot
pub(super) fn generate_slot_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    // Check if all children are text/interpolation - if so, concatenate into single _createTextVNode
    let all_text_or_interp = children.iter().all(|child| {
        matches!(
            child,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });

    if all_text_or_interp && !children.is_empty() {
        ctx.newline();
        ctx.use_helper(RuntimeHelper::CreateText);
        ctx.push(ctx.helper(RuntimeHelper::CreateText));
        ctx.push("(");

        let has_interpolation = children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                ctx.push(" + ");
            }
            match child {
                TemplateChildNode::Text(text) => {
                    ctx.push("\"");
                    ctx.push_text(text);
                    ctx.push("\"");
                }
                TemplateChildNode::Interpolation(interp) => {
                    // Vue 1.x raw-HTML `{{{ … }}}` renders unescaped.
                    #[cfg(feature = "legacy")]
                    let raw = interp.raw;
                    #[cfg(not(feature = "legacy"))]
                    let raw = false;
                    if raw {
                        generate_slot_expression(ctx, &interp.content);
                    } else {
                        ctx.use_helper(RuntimeHelper::ToDisplayString);
                        ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
                        ctx.push("(");
                        generate_slot_expression(ctx, &interp.content);
                        ctx.push(")");
                    }
                }
                _ => {}
            }
        }

        if has_interpolation {
            ctx.push(", 1 /* TEXT */)");
        } else {
            ctx.push(")");
        }
    } else {
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.newline();
            generate_slot_child_node(ctx, child);
        }
    }
}

/// Generate a single child node for slot content
pub(super) fn generate_slot_child_node(ctx: &mut CodegenContext, child: &TemplateChildNode<'_>) {
    match child {
        TemplateChildNode::Text(text) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(\"");
            ctx.push_text(text);
            ctx.push("\")");
        }
        TemplateChildNode::Interpolation(interp) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(");
            // Vue 1.x raw-HTML `{{{ … }}}` renders unescaped.
            #[cfg(feature = "legacy")]
            let raw = interp.raw;
            #[cfg(not(feature = "legacy"))]
            let raw = false;
            if raw {
                // Generate expression, stripping _ctx. prefix for slot params
                generate_slot_expression(ctx, &interp.content);
            } else {
                ctx.use_helper(RuntimeHelper::ToDisplayString);
                ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
                ctx.push("(");
                // Generate expression, stripping _ctx. prefix for slot params
                generate_slot_expression(ctx, &interp.content);
                ctx.push(")");
            }
            ctx.push(", 1 /* TEXT */)");
        }
        _ => {
            generate_node(ctx, child);
        }
    }
}

/// Generate expression for slot content, stripping _ctx. prefix for slot parameters
fn generate_slot_expression(ctx: &mut CodegenContext, expr: &ExpressionNode<'_>) {
    match expr {
        ExpressionNode::Simple(exp) => {
            if exp.is_static {
                ctx.push("\"");
                ctx.push(exp.content);
                ctx.push("\"");
            } else {
                // Strip _ctx. prefix for slot parameters
                let content = strip_ctx_prefix_for_slot_params(ctx, exp.content);
                ctx.push_expression(&content, exp.loc.span);
            }
        }
        ExpressionNode::Compound(comp) => {
            for child in comp.children.iter() {
                match child {
                    crate::CompoundExpressionChild::Simple(exp) => {
                        if exp.is_static {
                            ctx.push("\"");
                            ctx.push(exp.content);
                            ctx.push("\"");
                        } else {
                            let content = strip_ctx_prefix_for_slot_params(ctx, exp.content);
                            ctx.push_expression(&content, exp.loc.span);
                        }
                    }
                    crate::CompoundExpressionChild::String(s) => {
                        ctx.push(s);
                    }
                    crate::CompoundExpressionChild::Symbol(helper) => {
                        ctx.push(ctx.helper(*helper));
                    }
                    _ => {}
                }
            }
        }
    }
}
