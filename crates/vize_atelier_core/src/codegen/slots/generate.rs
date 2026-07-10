//! Slots object generation for component children.

use crate::steps::v_slot::{collect_slots, get_slot_name, has_v_slot};
use crate::{ElementNode, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode};

use super::super::context::CodegenContext;
use super::super::helpers::{escape_js_string, is_valid_js_identifier};
use super::children::{generate_slot_child_node, generate_slot_children};
use super::create_slots::generate_create_slots;
use super::detect::{has_conditional_or_loop_slots, has_forwarded_slot_outlet};
use super::params::{extract_slot_params, get_slot_props, prefix_slot_defaults};

/// Generate slots object for component
pub fn generate_slots(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    // Note: WithCtx helper is registered at each _withCtx() output site,
    // not here, to avoid importing it when slots don't actually use it.

    // Check for v-slot on component root (shorthand for default slot)
    let root_slot = el.props.iter().find_map(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name.as_str() == "slot"
        {
            return Some(dir.as_ref());
        }
        None
    });

    let collected_slots = collect_slots(el);
    let has_forwarded_slots = has_forwarded_slot_outlet(el);
    let forwarded_slots_are_dynamic = has_forwarded_slots && ctx.has_slot_params();
    let has_dynamic_slots =
        ctx.in_v_for || collected_slots.iter().any(|s| s.is_dynamic) || forwarded_slots_are_dynamic;
    let has_conditional_slots = has_conditional_or_loop_slots(el);

    // If there are conditional (v-if) or looped (v-for) slots, use createSlots
    if has_conditional_slots && root_slot.is_none() {
        generate_create_slots(ctx, el);
        return;
    }

    ctx.push("{");
    ctx.indent();

    if let Some(slot_dir) = root_slot {
        // v-slot on component root - all children go to default slot
        ctx.newline();
        ctx.push("default: ");
        ctx.use_helper(RuntimeHelper::WithCtx);
        ctx.push(ctx.helper(RuntimeHelper::WithCtx));
        ctx.push("(");
        // Slot props (scoped slot params) - use raw source with default value prefix
        let params = if let Some(props_str) = get_slot_props(slot_dir) {
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
                && template_el.tag.as_str() == "template"
                && has_v_slot(template_el)
            {
                // This is a named slot template
                if let Some(slot_dir) = template_el.props.iter().find_map(|p| {
                    if let PropNode::Directive(dir) = p
                        && dir.name.as_str() == "slot"
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

                    let slot_name = get_slot_name(slot_dir);
                    let is_dynamic = slot_dir
                        .arg
                        .as_ref()
                        .map(|arg| match arg {
                            ExpressionNode::Simple(exp) => !exp.is_static,
                            ExpressionNode::Compound(_) => true,
                        })
                        .unwrap_or(false);

                    if is_dynamic {
                        let trimmed_name = slot_name.trim();
                        if trimmed_name.starts_with('`') && trimmed_name.ends_with('`') {
                            // Template literal slot name: `item.name` → ["item.name"]
                            let inner = &trimmed_name[1..trimmed_name.len() - 1];
                            ctx.push("[\"");
                            ctx.push(&escape_js_string(inner));
                            ctx.push("\"]");
                        } else {
                            // Dynamic slot name: [_ctx.slotName]
                            ctx.push("[");
                            ctx.push("_ctx.");
                            ctx.push(&slot_name);
                            ctx.push("]");
                        }
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
                    let params = if let Some(props_str) = get_slot_props(slot_dir) {
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
                    !(template_el.tag.as_str() == "template" && has_v_slot(template_el))
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

    // Add slot stability flag
    ctx.push(",");
    ctx.newline();
    if has_forwarded_slots && !forwarded_slots_are_dynamic {
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
