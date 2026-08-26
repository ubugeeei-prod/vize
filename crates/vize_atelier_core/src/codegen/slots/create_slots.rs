//! `createSlots` generation for conditional and looped slot templates.
//!
//! When a component carries `<template v-if #name>` / `<template v-for #name>`
//! children the slot *set* is decided at runtime, so the slots object cannot be
//! an object literal. Vue's `createSlots(base, dynamicEntries)` merges the
//! statically known entries with the conditional/looped ones; this module emits
//! that call. The plain object-literal path lives in `generate.rs`.

use crate::steps::v_slot::{get_slot_name, has_v_slot};
use crate::{ElementNode, ForNode, IfNode, PropNode, RuntimeHelper, TemplateChildNode};
use vize_s0::{String, ToCompactString};

use super::super::context::CodegenContext;
use super::super::expression::generate_expression;
use super::detect::{child_is_slot_template, slot_children_have_meaningful_content, slots_spread};
use super::generate::{generate_slot_child_node, generate_slot_children};
use super::name::generate_slot_entry_name;
use super::params::{extract_slot_params, get_slot_props, prefix_slot_defaults};

/// Generate slots using createSlots for conditional/looped slot templates
pub(super) fn generate_create_slots(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.use_helper(RuntimeHelper::CreateSlots);
    ctx.push(ctx.helper(RuntimeHelper::CreateSlots));
    ctx.push("(");
    generate_create_slots_base(ctx, el);
    ctx.push(", [");
    ctx.indent();

    let mut first = true;
    for child in &el.children {
        match child {
            TemplateChildNode::If(if_node) => {
                // v-if on slot template: generate conditional slot entry
                if !first {
                    ctx.push(",");
                }
                first = false;
                ctx.newline();
                generate_conditional_slot(ctx, if_node);
            }
            TemplateChildNode::For(for_node) => {
                // v-for on slot template: generate looped slot entries
                if !first {
                    ctx.push(",");
                }
                first = false;
                ctx.newline();
                generate_looped_slot(ctx, for_node);
            }
            TemplateChildNode::Element(template_el)
                if template_el.tag == "template" && has_v_slot(template_el) =>
            {
                // Regular named slot (no v-if/v-for)
                if !first {
                    ctx.push(",");
                }
                first = false;
                ctx.newline();
                // Generate as static slot entry
                generate_static_slot_entry(ctx, template_el);
            }
            _ => {}
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("])");
}

fn generate_create_slots_base(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let default_children: Vec<_> = el
        .children
        .iter()
        .filter(|child| !child_is_slot_template(child))
        .collect();
    let has_default_children = slot_children_have_meaningful_content(&default_children);
    // A `v-slots` spread combined with `v-if`/`v-for` slot templates keeps the
    // `createSlots` shape: the forwarded entries go into the base object, which
    // `createSlots` then extends with the conditional/looped ones. `_: 2` stays
    // because the conditional entries need it, so a forwarded entry that is not
    // already `withCtx`-wrapped is bound to the wrong instance here. The JSX
    // lowering cannot produce this combination (its slot bodies carry their own
    // control flow); it is reachable only from a template, and dropping the
    // spread instead would be silent.
    let spread = slots_spread(el);

    if !has_default_children && spread.is_none() {
        ctx.push("{ _: 2 /* DYNAMIC */ }");
        return;
    }

    ctx.push("{");
    ctx.indent();

    if has_default_children {
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
        ctx.push("]),");
    }

    if let Some(exp) = spread {
        ctx.newline();
        ctx.push("...");
        generate_expression(ctx, exp);
        ctx.push(",");
    }

    ctx.newline();
    ctx.push("_: 2 /* DYNAMIC */");

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

/// Generate a conditional slot entry (v-if on slot template)
fn generate_conditional_slot(ctx: &mut CodegenContext, if_node: &IfNode<'_>) {
    // For each branch: condition ? { name, fn, key } : undefined
    for (i, branch) in if_node.branches.iter().enumerate() {
        if i > 0 {
            ctx.newline();
            ctx.push(": ");
        }

        // Generate condition
        if let Some(condition) = &branch.condition {
            ctx.push("(");
            generate_expression(ctx, condition);
            ctx.push(")");
            ctx.indent();
            ctx.newline();
            ctx.push("? ");
        }

        // Find the slot template in this branch
        let slot_template = branch.children.iter().find_map(|child| {
            if let TemplateChildNode::Element(el) = child
                && el.tag == "template"
                && has_v_slot(el)
            {
                return Some(el.as_ref());
            }
            None
        });

        if let Some(template_el) = slot_template {
            generate_slot_object_entry(ctx, template_el, Some(i));
        } else {
            ctx.push("undefined");
        }

        if branch.condition.is_some() {
            ctx.deindent();
        }
    }
    if if_node
        .branches
        .last()
        .is_none_or(|branch| branch.condition.is_some())
    {
        ctx.newline();
        ctx.push(": undefined");
    }
}

/// Generate a looped slot entry (v-for on slot template)
fn generate_looped_slot(ctx: &mut CodegenContext, for_node: &ForNode<'_>) {
    ctx.use_helper(RuntimeHelper::RenderList);
    ctx.push(ctx.helper(RuntimeHelper::RenderList));
    ctx.push("(");
    generate_expression(ctx, &for_node.source);
    ctx.push(", (");

    // Collect callback parameter names for scope registration
    let mut callback_params: Vec<String> = Vec::new();

    if let Some(value) = &for_node.value_alias {
        generate_expression(ctx, value);
        super::super::v_for::helpers::extract_for_params(value, &mut callback_params);
    }
    if let Some(key) = &for_node.key_alias {
        ctx.push(", ");
        generate_expression(ctx, key);
        super::super::v_for::helpers::extract_for_params(key, &mut callback_params);
    }
    if let Some(index) = &for_node.object_index_alias {
        ctx.push(", ");
        generate_expression(ctx, index);
        super::super::v_for::helpers::extract_for_params(index, &mut callback_params);
    }

    ctx.add_slot_params(&callback_params);

    ctx.push(") => {");
    ctx.indent();
    ctx.newline();
    ctx.push("return ");

    // Find the slot template in the for body
    let slot_template = for_node.children.iter().find_map(|child| {
        if let TemplateChildNode::Element(el) = child
            && el.tag == "template"
            && has_v_slot(el)
        {
            return Some(el.as_ref());
        }
        None
    });

    if let Some(template_el) = slot_template {
        generate_slot_object_entry(ctx, template_el, None);
    }

    ctx.remove_slot_params(&callback_params);

    ctx.deindent();
    ctx.newline();
    ctx.push("})");
}

/// Generate a slot object entry: { name: "slotName", fn: _withCtx(() => [...]), key: "N" }
fn generate_slot_object_entry(
    ctx: &mut CodegenContext,
    template_el: &ElementNode<'_>,
    key_index: Option<usize>,
) {
    let slot_dir = template_el.props.iter().find_map(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name == "slot"
        {
            return Some(dir.as_ref());
        }
        None
    });

    if let Some(dir) = slot_dir {
        let slot_name = get_slot_name(dir, &ctx.source);

        ctx.push("{");
        ctx.indent();
        ctx.newline();

        // name
        ctx.push("name: ");
        generate_slot_entry_name(ctx, dir, &slot_name);
        ctx.push(",");
        ctx.newline();

        // fn
        ctx.push("fn: ");
        ctx.use_helper(RuntimeHelper::WithCtx);
        ctx.push(ctx.helper(RuntimeHelper::WithCtx));
        ctx.push("(");

        // Slot props
        let params = if let Some(props_str) = get_slot_props(dir, &ctx.source) {
            let processed = prefix_slot_defaults(&props_str);
            ctx.push("(");
            ctx.push(&processed);
            ctx.push(")");
            extract_slot_params(&props_str)
        } else {
            ctx.push("()");
            vec![]
        };

        ctx.add_slot_params(&params);

        ctx.push(" => [");
        ctx.indent();
        generate_slot_children(ctx, &template_el.children);
        ctx.deindent();
        ctx.newline();
        ctx.push("])");

        ctx.remove_slot_params(&params);

        // key (for v-if branches)
        if let Some(key) = key_index {
            ctx.push(",");
            ctx.newline();
            ctx.push("key: \"");
            ctx.push(&key.to_compact_string());
            ctx.push("\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    }
}

/// Generate a static slot entry for createSlots context
fn generate_static_slot_entry(ctx: &mut CodegenContext, template_el: &ElementNode<'_>) {
    generate_slot_object_entry(ctx, template_el, None);
}
