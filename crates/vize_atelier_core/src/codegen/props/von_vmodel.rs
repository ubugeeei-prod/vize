//! `v-on` and dynamic `v-model` prop emission.
//!
//! Emits the object-property form of a `v-on` handler and of a dynamic-argument
//! `v-model` on a component. Split out of `directives` to keep that file focused
//! on `v-bind` and directive dispatch.

use crate::relief_projection::ReliefRenderOp;
use crate::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext, expression::generate_simple_expression,
    helpers::is_valid_js_identifier,
};

/// Generate v-on directive as a prop
pub(super) fn generate_von_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    let ReliefRenderOp::Directive { arg, modifiers, .. } = ReliefRenderOp::from_directive(dir)
    else {
        unreachable!("v-on emission requires ReliefRenderOp::Directive");
    };
    let is_dynamic_event = if let Some(ExpressionNode::Simple(exp)) = arg.and_then(|arg| arg.node())
    {
        !exp.is_static
    } else {
        false
    };

    if let Some(ExpressionNode::Simple(exp)) = arg.and_then(|arg| arg.node()) {
        if is_dynamic_event {
            // Dynamic event name: [_toHandlerKey(_ctx.event)]:
            ctx.use_helper(RuntimeHelper::ToHandlerKey);
            ctx.push("[");
            ctx.push(ctx.helper(RuntimeHelper::ToHandlerKey));
            ctx.push("(");
            let content = exp.content.as_str();
            if let Some(local) = content
                .strip_prefix("_ctx.")
                .filter(|local| ctx.is_slot_param(local))
            {
                ctx.push(local);
            } else if content.contains('.') || content.starts_with('_') || content.starts_with('$')
            {
                generate_simple_expression(ctx, exp);
            } else if ctx.is_slot_param(content) {
                ctx.push(content);
            } else {
                ctx.push("_ctx.");
                ctx.push(content);
            }
            ctx.push(")]: ");
        } else {
            // Mirror Vue's event-name casing rule (transforms/vOn.ts), including
            // mouse-button event renaming, `vue:` vnode hooks, and the `on:`
            // case-preserving form for custom-element events on plain elements.
            // The `on:` case-preserving form only applies to user-authored v-on
            // directives (those carry a `raw_name`). Compiler-synthesized handlers
            // like v-model's `update:modelValue` always camelize.
            let on_plain_element = ctx.props_is_plain_element && dir.raw_name.is_some();
            let event_name = super::events::von_event_key_for(
                exp.content.as_str(),
                on_plain_element,
                modifiers.names(),
            );

            let needs_quotes = !is_valid_js_identifier(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            // Anchor the generated event-handler key back to the v-on argument
            // in source, recording the original event name so it lands in the
            // v3 `names` array. No-op without `source_map`.
            ctx.record_mapping_named(&exp.loc.start, &exp.content);
            ctx.push(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
        }
    }

    super::events::generate_von_handler_value(ctx, dir);
}

/// Generate dynamic v-model on component as props
pub(super) fn generate_vmodel_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    let ReliefRenderOp::Directive {
        arg,
        exp,
        modifiers,
        ..
    } = ReliefRenderOp::from_directive(dir)
    else {
        unreachable!("v-model emission requires ReliefRenderOp::Directive");
    };
    // Handle dynamic v-model on component
    // Generate: [_ctx.prop]: _ctx.value, ["onUpdate:" + _ctx.prop]: handler
    if let Some(ExpressionNode::Simple(arg_exp)) = arg.and_then(|arg| arg.node())
        && !arg_exp.is_static
    {
        let prop_name = &arg_exp.content;
        let value_exp = exp
            .and_then(|exp| exp.node())
            .map(|e| match e {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(c) => c.loc.source.as_str(),
            })
            .unwrap_or("undefined");

        // [_ctx.prop]: _ctx.value
        ctx.push("[_ctx.");
        ctx.push(prop_name);
        ctx.push("]: ");
        ctx.push(value_exp);
        ctx.push(",");
        ctx.newline();

        // ["onUpdate:" + _ctx.prop]: $event => ((_ctx.value) = $event)
        ctx.push("[\"onUpdate:\" + _ctx.");
        ctx.push(prop_name);
        ctx.push("]: $event => ((");
        ctx.push(value_exp);
        ctx.push(") = $event)");

        // Add modifiers if present
        if !modifiers.is_empty() {
            ctx.push(",");
            ctx.newline();
            // [_ctx.prop + "Modifiers"]: { modifier: true }
            ctx.push("[_ctx.");
            ctx.push(prop_name);
            ctx.push(" + \"Modifiers\"]: { ");
            for (i, modifier) in modifiers.names().enumerate() {
                if i > 0 {
                    ctx.push(", ");
                }
                ctx.push(modifier);
                ctx.push(": true");
            }
            ctx.push(" }");
        }
    }
}
