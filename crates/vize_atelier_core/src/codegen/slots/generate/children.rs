//! Slot child and expression spelling.

use super::super::super::context::CodegenContext;
use super::super::super::node::generate_node;
use super::slot_params::strip_ctx_prefix_for_slot_params;
use crate::{ExpressionNode, RuntimeHelper, TemplateChildNode};

/// Generate adjacent text runs without crossing a slot or node boundary.
pub(in crate::codegen::slots) fn generate_slot_children(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
) {
    // Preserve the preexisting wholly-textual legacy raw branch. Raw values
    // deliberately bypass ToDisplayString, so extending `+` to other bodies
    // would change numeric coercion rather than merely joining text nodes.
    #[cfg(feature = "legacy")]
    if children.iter().any(is_raw) && children.iter().all(is_text) {
        ctx.newline();
        generate_slot_text_run(ctx, children);
        return;
    }
    generate_slot_children_where(ctx, children, |_| true);
}

pub(in crate::codegen::slots) fn generate_slot_children_where(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    selected: impl Fn(&TemplateChildNode<'_>) -> bool,
) {
    #[cfg(feature = "legacy")]
    let merge = !children
        .iter()
        .any(|child| selected(child) && is_raw(child));
    #[cfg(not(feature = "legacy"))]
    let merge = true;
    let mut index = 0;
    let mut first = true;
    while let Some(child) = children.get(index) {
        if !selected(child) {
            index += 1;
            continue;
        }
        if !first {
            ctx.push(",");
        }
        first = false;
        ctx.newline();
        if merge && is_text(child) {
            let start = index;
            index += 1;
            while children
                .get(index)
                .is_some_and(|child| selected(child) && is_text(child))
            {
                index += 1;
            }
            generate_slot_text_run(ctx, children.get(start..index).unwrap_or_default());
        } else {
            generate_slot_child_node(ctx, child);
            index += 1;
        }
    }
}

#[cfg(feature = "legacy")]
fn is_raw(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Interpolation(interp) if interp.raw)
}

fn is_text(child: &TemplateChildNode<'_>) -> bool {
    matches!(
        child,
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
    )
}

fn generate_slot_text_run(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    ctx.use_helper(RuntimeHelper::CreateText);
    ctx.push(ctx.helper(RuntimeHelper::CreateText));
    ctx.push("(");
    let mut dynamic = false;
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            ctx.push(" + ");
        }
        match child {
            TemplateChildNode::Text(text) => {
                ctx.push("\"");
                ctx.push_text(text);
                ctx.push("\"");
            }
            TemplateChildNode::Interpolation(interp) => {
                dynamic = true;
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
    ctx.push(if dynamic { ", 1 /* TEXT */)" } else { ")" });
}

/// Generate a single child node for slot content
fn generate_slot_child_node(ctx: &mut CodegenContext, child: &TemplateChildNode<'_>) {
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
