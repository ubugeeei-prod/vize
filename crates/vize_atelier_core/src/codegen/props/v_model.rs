//! Dynamic `v-model` on a component, emitted as computed props.

use crate::{DirectiveNode, ExpressionNode};

use super::super::{
    context::CodegenContext,
    expression::generate_expression,
    helpers::{escape_js_string, is_valid_js_identifier},
};

/// Generate dynamic v-model on component as props.
///
/// Emits `[(prop)]: value`, `["onUpdate:" + (prop)]: handler`, and, when modifiers
/// were given, `[(prop) + "Modifiers"]: {…}`.
///
/// The argument goes through the shared expression emitter rather than being
/// hard-prefixed with `_ctx.`: the element transform already ran
/// `process_expression` over it, so a template argument arrives spelled
/// `_ctx.prop`, while an opt-in Babel JSX argument (#3391) stays the raw closure
/// identifier `@vue/babel-plugin-jsx` emits.
pub(super) fn generate_vmodel_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    let Some(arg) = &dir.arg else {
        return;
    };
    if matches!(arg, ExpressionNode::Simple(simple) if simple.is_static) {
        return;
    }

    let value_exp = dir
        .exp
        .as_ref()
        .map(|e| match e {
            ExpressionNode::Simple(s) => vize_s0::String::new(s.content),
            ExpressionNode::Compound(c) => vize_s0::String::new(c.loc.span.slice(&ctx.source)),
        })
        .unwrap_or_else(|| vize_s0::String::new("undefined"));

    // Parentheses preserve comma expressions and derived-key precedence.
    ctx.push("[(");
    generate_expression(ctx, arg);
    ctx.push(")]: ");
    ctx.push(&value_exp);
    ctx.push(",");
    ctx.newline();

    // ["onUpdate:" + (prop)]: $event => ((value) = $event)
    ctx.push("[\"onUpdate:\" + (");
    generate_expression(ctx, arg);
    ctx.push(")]: $event => ((");
    ctx.push(&value_exp);
    ctx.push(") = $event)");

    if !dir.modifiers.is_empty() {
        ctx.push(",");
        ctx.newline();
        // [(prop) + "Modifiers"]: { modifier: true }
        ctx.push("[(");
        generate_expression(ctx, arg);
        ctx.push(") + \"Modifiers\"]: { ");
        for (i, modifier) in dir.modifiers.iter().enumerate() {
            if i > 0 {
                ctx.push(", ");
            }
            if is_valid_js_identifier(modifier.content) {
                ctx.push(modifier.content);
            } else {
                ctx.push("\"");
                ctx.push(&escape_js_string(modifier.content));
                ctx.push("\"");
            }
            ctx.push(": true");
        }
        ctx.push(" }");
    }
}
