//! Slots object generation for component children.

use crate::steps::v_slot::{collect_slots, get_slot_name, has_v_slot, is_dynamic_slot};
use crate::{ElementNode, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode};

use super::super::context::CodegenContext;

mod children;
mod slot_params;

pub(super) use children::{generate_slot_children, generate_slot_children_where};

use super::super::expression::generate_expression;
use super::super::helpers::{escape_js_string, is_valid_js_identifier};
use super::create_slots::generate_create_slots;
use super::detect::{
    has_conditional_or_loop_slots, has_forwarded_slot_outlet, slots_are_only_forwarded,
    slots_spread,
};
use super::name::{component_root_slot, emit_slot_property_name};
use super::params::slot_parameters;

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
    let root_slot = component_root_slot(el);

    let collected_slots = collect_slots(el, &ctx.source);
    let has_forwarded_slots = has_forwarded_slot_outlet(el);
    let forwarded_slots_are_dynamic = has_forwarded_slots && ctx.has_slot_params();
    let has_dynamic_slots = (ctx.in_v_for || ctx.in_match_scope)
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
        ctx.push_slot_function(el.loc.span.start);
        ctx.push("(");
        // Slot props (scoped slot params) - use raw source with default value prefix
        let params = if let Some((processed, params)) = slot_parameters(slot_dir, ctx) {
            ctx.push("(");
            ctx.push(&processed);
            ctx.push(")");
            params
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
                        ctx.push_slot_name(&slot_name, slot_dir);
                    } else {
                        ctx.push("\"");
                        ctx.push_slot_name(&escape_js_string(&slot_name), slot_dir);
                        ctx.push("\"");
                    }

                    if slot_name.as_str() == "default" {
                        has_generated_default = true;
                    }

                    ctx.push(": ");
                    ctx.push_slot_function(template_el.loc.span.start);
                    ctx.push("(");

                    // Slot props - use raw source with default value prefix
                    let params = if let Some((processed, params)) = slot_parameters(slot_dir, ctx) {
                        ctx.push("(");
                        ctx.push(&processed);
                        ctx.push(")");
                        params
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

        let has_slot_template = super::detect::has_authored_slot_template(el);
        let default_children: Vec<_> = el
            .children
            .iter()
            .filter(|child| super::detect::child_is_implicit_default(child, has_slot_template))
            .collect();
        let has_default_content = !has_slot_template
            || super::detect::slot_children_have_meaningful_content(&default_children);
        if !default_children.is_empty() && !has_generated_default && has_default_content {
            if !first_slot {
                ctx.push(",");
            }
            ctx.newline();
            ctx.push("default: ");
            ctx.use_helper(RuntimeHelper::WithCtx);
            ctx.push(ctx.helper(RuntimeHelper::WithCtx));
            ctx.push("(() => [");
            ctx.indent();
            generate_slot_children_where(ctx, &el.children, |child| {
                super::detect::child_is_implicit_default(child, has_slot_template)
            });
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
