//! Slots object generation for component children.

use crate::steps::v_slot::{collect_slots, get_slot_name, has_v_slot, is_dynamic_slot};
use crate::{ElementNode, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode};
use vize_carton::String;

use super::super::context::CodegenContext;
use super::super::expression::generate_expression;
use super::super::helpers::{escape_js_string, is_valid_js_identifier};
use super::super::node::generate_node;
use super::create_slots::generate_create_slots;
use super::detect::{
    has_conditional_or_loop_slots, has_forwarded_slot_outlet, slots_are_only_forwarded,
    slots_spread,
};
use super::params::{extract_slot_params, get_slot_props, prefix_slot_defaults};

/// Generate slots object for component
///
/// # Forwarded slots (`v-slots`)
///
/// A `v-slots` directive carries an object the compiler cannot see inside, so
/// it is emitted as a spread rather than expanded into entries (#3467). Two
/// shapes, both matching `@vue/babel-plugin-jsx`:
///
/// - nothing else contributes slots — the forwarded value *is* the children
///   argument: `createBlock(B, null, slots, 1024 /* DYNAMIC_SLOTS */)`;
/// - otherwise the authored slots come first and `...expr` closes the object,
///   so a forwarded entry overrides an authored one of the same name.
///
/// **No `_` stability flag is emitted alongside a spread**, and that is
/// load-bearing rather than an omission. `initSlots`/`updateSlots` only run
/// `normalizeObjectSlots` — which binds each raw slot to the owning instance
/// and passes already-`withCtx`-wrapped entries through untouched via
/// `rawSlot._n` — when the children object carries no `_`. Under `_: 2
/// /* DYNAMIC */` `updateSlots` does a bare `extend(slots, children)` with no
/// normalization at all, so an entry arriving through the spread unwrapped
/// would render without the right instance context; under `_: 1 /* STABLE */`
/// the child would never re-render when the forwarded slots change. The vnode
/// instead carries `1024 /* DYNAMIC_SLOTS */` (see
/// [`has_dynamic_slots_flag`](super::detect::has_dynamic_slots_flag)) to force
/// that update.
pub fn generate_slots(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    // Note: WithCtx helper is registered at each _withCtx() output site,
    // not here, to avoid importing it when slots don't actually use it.

    // A `v-slots` value with nothing to merge it into is the children argument
    // itself: `createVNode(B, null, slots)`, exactly as babel emits it.
    if slots_are_only_forwarded(el)
        && let Some(exp) = slots_spread(el)
    {
        generate_expression(ctx, exp);
        return;
    }

    // Check for v-slot on component root. Bare `v-slot` is the default slot;
    // named / dynamic root spellings preserve their authored key.
    let root_slot = el.props.iter().find_map(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name == "slot"
        {
            return Some(dir.as_ref());
        }
        None
    });

    let collected_slots = collect_slots(el, &ctx.source);
    let has_forwarded_slots = has_forwarded_slot_outlet(el);
    let forwarded_slots_are_dynamic = has_forwarded_slots && ctx.has_slot_params();
    let has_dynamic_slots = ctx.in_v_for
        || root_slot.is_some_and(is_dynamic_slot)
        || collected_slots.iter().any(|s| s.is_dynamic)
        || forwarded_slots_are_dynamic;
    let has_conditional_slots = has_conditional_or_loop_slots(el);

    // If there are conditional (v-if) or looped (v-for) slots, use createSlots
    if has_conditional_slots && root_slot.is_none() {
        generate_create_slots(ctx, el);
        return;
    }

    ctx.push("{");
    ctx.indent();

    if let Some(slot_dir) = root_slot {
        // v-slot on component root - all children go to the authored slot key.
        ctx.newline();
        let slot_name = get_slot_name(slot_dir, &ctx.source);
        let is_dynamic = is_dynamic_slot(slot_dir);
        emit_slot_property_name(ctx, slot_dir, &slot_name, is_dynamic);
        ctx.push(": ");
        ctx.use_helper(RuntimeHelper::WithCtx);
        ctx.push(ctx.helper(RuntimeHelper::WithCtx));
        ctx.push("(");
        // Slot props (scoped slot params) - use raw source with default value prefix
        let params = if let Some(props_str) = get_slot_props(slot_dir, &ctx.source) {
            let processed = prefix_slot_defaults(&props_str);
            ctx.push("(");
            ctx.push(&processed);
            ctx.push(")");
            extract_slot_params(&props_str)
        } else {
            ctx.push("()");
            vec![]
        };

        // Track slot params for stripping _ctx. prefix
        ctx.add_slot_params(&params);

        ctx.push(" => [");
        ctx.indent();
        generate_slot_children(ctx, &el.children);
        ctx.deindent();
        ctx.newline();
        ctx.push("])");

        // Remove slot params
        ctx.remove_slot_params(&params);
    } else {
        // Check for named slots via template#slotName
        let mut has_generated_default = false;
        let mut first_slot = true;

        for child in &el.children {
            if let TemplateChildNode::Element(template_el) = child
                && template_el.tag == "template"
                && has_v_slot(template_el)
            {
                // This is a named slot template
                if let Some(slot_dir) = template_el.props.iter().find_map(|p| {
                    if let PropNode::Directive(dir) = p
                        && dir.name == "slot"
                    {
                        return Some(dir.as_ref());
                    }
                    None
                }) {
                    if !first_slot {
                        ctx.push(",");
                    }
                    first_slot = false;
                    ctx.newline();

                    let slot_name = get_slot_name(slot_dir, &ctx.source);
                    let is_dynamic = slot_dir
                        .arg
                        .as_ref()
                        .map(|arg| match arg {
                            ExpressionNode::Simple(exp) => !exp.is_static,
                            ExpressionNode::Compound(_) => true,
                        })
                        .unwrap_or(false);

                    if is_dynamic {
                        // Use the transformed argument instead of rebuilding it from the
                        // raw source. The expression generator preserves v-for/slot locals
                        // and resolves script-setup bindings (`ref` -> `.value`) exactly as
                        // it does for every other template expression.
                        ctx.push("[");
                        if let Some(arg) = &slot_dir.arg {
                            generate_expression(ctx, arg);
                        }
                        ctx.push("]");
                    } else if is_valid_js_identifier(&slot_name) {
                        ctx.push(&slot_name);
                    } else {
                        ctx.push("\"");
                        ctx.push(&escape_js_string(&slot_name));
                        ctx.push("\"");
                    }

                    if slot_name.as_str() == "default" {
                        has_generated_default = true;
                    }

                    ctx.push(": ");
                    ctx.use_helper(RuntimeHelper::WithCtx);
                    ctx.push(ctx.helper(RuntimeHelper::WithCtx));
                    ctx.push("(");

                    // Slot props - use raw source with default value prefix
                    let params = if let Some(props_str) = get_slot_props(slot_dir, &ctx.source) {
                        let processed = prefix_slot_defaults(&props_str);
                        ctx.push("(");
                        ctx.push(&processed);
                        ctx.push(")");
                        extract_slot_params(&props_str)
                    } else {
                        ctx.push("()");
                        vec![]
                    };

                    // Track slot params for stripping _ctx. prefix
                    ctx.add_slot_params(&params);

                    ctx.push(" => [");
                    ctx.indent();
                    generate_slot_children(ctx, &template_el.children);
                    ctx.deindent();
                    ctx.newline();
                    ctx.push("])");

                    // Remove slot params
                    ctx.remove_slot_params(&params);
                }
            }
        }

        // Generate default slot for non-template children
        let default_children: Vec<_> = el
            .children
            .iter()
            .filter(|child| {
                if let TemplateChildNode::Element(template_el) = child {
                    !(template_el.tag == "template" && has_v_slot(template_el))
                } else {
                    true
                }
            })
            .collect();

        if !default_children.is_empty() && !has_generated_default {
            if !first_slot {
                ctx.push(",");
            }
            ctx.newline();
            ctx.push("default: ");
            ctx.use_helper(RuntimeHelper::WithCtx);
            ctx.push(ctx.helper(RuntimeHelper::WithCtx));
            ctx.push("(() => [");
            ctx.indent();
            for (i, child) in default_children.iter().enumerate() {
                if i > 0 {
                    ctx.push(",");
                }
                ctx.newline();
                generate_slot_child_node(ctx, child);
            }
            ctx.deindent();
            ctx.newline();
            ctx.push("])");
        }
    }

    ctx.push(",");
    ctx.newline();
    if let Some(exp) = slots_spread(el) {
        // The forwarded object closes the literal so its entries override the
        // authored ones, matching babel's `{default: () => […], ...slots}`. No
        // `_` flag: see this function's docs for why the raw-slots path is the
        // only one that normalizes spread entries correctly.
        ctx.push("...");
        generate_expression(ctx, exp);
    } else if has_forwarded_slots && !forwarded_slots_are_dynamic {
        ctx.push("_: 3 /* FORWARDED */");
    } else if has_dynamic_slots {
        ctx.push("_: 2 /* DYNAMIC */");
    } else {
        ctx.push("_: 1 /* STABLE */");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

fn emit_slot_property_name(
    ctx: &mut CodegenContext,
    slot_dir: &crate::DirectiveNode<'_>,
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
                    ctx.push(&super::super::helpers::escape_js_string(text.content));
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
            ctx.push(&super::super::helpers::escape_js_string(text.content));
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
                ctx.push(&content);
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
                            ctx.push(&content);
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

/// Strip _ctx. prefix from identifiers that are slot parameters
fn strip_ctx_prefix_for_slot_params(ctx: &CodegenContext, content: &str) -> String {
    let mut result = String::new(content);
    for param in &ctx.slot_params {
        // Replace _ctx.paramName with paramName
        let mut prefixed = String::with_capacity(5 + param.len());
        prefixed.push_str("_ctx.");
        prefixed.push_str(param);
        let replaced = result.replace(prefixed.as_str(), param.as_str());
        result = String::from(replaced);
    }
    result
}
