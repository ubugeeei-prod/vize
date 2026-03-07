//! Slot generation functions.
//!
//! Generates slot objects for component children.

use crate::ast::*;
use crate::transforms::v_slot::{collect_slots, get_slot_name, has_v_slot};

use super::context::CodegenContext;
use super::expression::generate_expression;
use super::helpers::{escape_js_string, is_valid_js_identifier};
use super::node::generate_node;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Get slot props expression as raw source (not transformed)
fn get_slot_props(dir: &DirectiveNode<'_>) -> Option<vize_carton::String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => s.loc.source.clone(),
        ExpressionNode::Compound(c) => c.loc.source.clone(),
    })
}

/// Add _ctx. prefix to default value identifiers in destructuring patterns.
/// e.g., "{ item = defaultItem }" -> "{ item = _ctx.defaultItem }"
/// Only processes identifiers after `=` (default values), not the param names.
fn prefix_slot_defaults(source: &str) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 20);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'=' {
            // Skip == and =>
            if i + 1 < len && (bytes[i + 1] == b'=' || bytes[i + 1] == b'>') {
                result.push(bytes[i] as char);
                result.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            result.push('=');
            i += 1;
            // Skip whitespace after =
            while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
                result.push(bytes[i] as char);
                i += 1;
            }
            // Check if next is a simple identifier (not a literal/number/string/object)
            if i < len && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'$') {
                // Collect the identifier
                let start = i;
                while i < len
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
                {
                    i += 1;
                }
                let ident = &source[start..i];
                // Don't prefix keywords/literals
                if !matches!(
                    ident,
                    "true" | "false" | "null" | "undefined" | "NaN" | "Infinity"
                ) {
                    result.push_str("_ctx.");
                }
                result.push_str(ident);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Extract parameter names from slot props expression
/// e.g., "{ item }" -> ["item"], "{ item, index }" -> ["item", "index"]
/// e.g., "slotProps" -> ["slotProps"]
fn extract_slot_params(props_str: &str) -> Vec<String> {
    let mut params = Vec::new();
    super::v_for::extract_destructure_params(props_str.trim(), &mut params);
    params
}

/// Check if component has slot children that need to be generated as slots object
pub fn has_slot_children(el: &ElementNode<'_>) -> bool {
    if el.children.is_empty() {
        return false;
    }

    // Teleport and KeepAlive pass children as arrays, not slot objects
    if matches!(
        el.tag.as_str(),
        "Teleport" | "teleport" | "KeepAlive" | "keep-alive"
    ) {
        return false;
    }

    // Check for v-slot on component root
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name.as_str() == "slot" {
                return true;
            }
        }
    }

    // If children consist only of whitespace text and/or comments, skip slot generation.
    // This matches Vue's official compiler behavior where `<Comp> </Comp>` does not
    // produce a default slot (important for <router-view>, <transition>, etc.).
    let has_meaningful_child = el.children.iter().any(|child| match child {
        TemplateChildNode::Text(t) => !t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    });
    if !has_meaningful_child {
        return false;
    }

    // Check for any children (default slot) or template slots
    true
}

/// Check if component has dynamic slots (requires DYNAMIC_SLOTS patch flag)
pub fn has_dynamic_slots_flag(el: &ElementNode<'_>) -> bool {
    let collected_slots = collect_slots(el);
    if collected_slots.iter().any(|s| s.is_dynamic) {
        return true;
    }
    // Also check for v-if/v-for on slot templates (they become IfNode/ForNode children)
    has_conditional_or_loop_slots(el)
}

/// Check if children have conditional (v-if) or looped (v-for) slot templates.
/// Only returns true when the IfNode/ForNode wraps a `<template v-slot>` element.
fn has_conditional_or_loop_slots(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(|child| match child {
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|c| {
                if let TemplateChildNode::Element(el) = c {
                    el.tag.as_str() == "template" && has_v_slot(el)
                } else {
                    false
                }
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|c| {
            if let TemplateChildNode::Element(el) = c {
                el.tag.as_str() == "template" && has_v_slot(el)
            } else {
                false
            }
        }),
        _ => false,
    })
}

/// Generate slots object for component
pub fn generate_slots(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    // Note: WithCtx helper is registered at each _withCtx() output site,
    // not here, to avoid importing it when slots don't actually use it.

    // Check for v-slot on component root (shorthand for default slot)
    let root_slot = el.props.iter().find_map(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name.as_str() == "slot" {
                return Some(dir.as_ref());
            }
        }
        None
    });

    let collected_slots = collect_slots(el);
    let has_dynamic_slots = ctx.in_v_for || collected_slots.iter().any(|s| s.is_dynamic);
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
            if let TemplateChildNode::Element(template_el) = child {
                if template_el.tag.as_str() == "template" && has_v_slot(template_el) {
                    // This is a named slot template
                    if let Some(slot_dir) = template_el.props.iter().find_map(|p| {
                        if let PropNode::Directive(dir) = p {
                            if dir.name.as_str() == "slot" {
                                return Some(dir.as_ref());
                            }
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
    if has_dynamic_slots {
        ctx.push("_: 2 /* DYNAMIC */");
    } else {
        ctx.push("_: 1 /* STABLE */");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

/// Generate slots using createSlots for conditional/looped slot templates
fn generate_create_slots(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.use_helper(RuntimeHelper::CreateSlots);
    ctx.push(ctx.helper(RuntimeHelper::CreateSlots));
    ctx.push("({ _: 2 /* DYNAMIC */ }, [");
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
            TemplateChildNode::Element(template_el) => {
                if template_el.tag.as_str() == "template" && has_v_slot(template_el) {
                    // Regular named slot (no v-if/v-for)
                    if !first {
                        ctx.push(",");
                    }
                    first = false;
                    ctx.newline();
                    // Generate as static slot entry
                    generate_static_slot_entry(ctx, template_el);
                }
            }
            _ => {}
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("])");
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
            if let TemplateChildNode::Element(el) = child {
                if el.tag.as_str() == "template" && has_v_slot(el) {
                    return Some(el.as_ref());
                }
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
    // Final else: undefined
    ctx.newline();
    ctx.push(": undefined");
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
        super::v_for::helpers::extract_for_params(value, &mut callback_params);
    }
    if let Some(key) = &for_node.key_alias {
        ctx.push(", ");
        generate_expression(ctx, key);
    }
    if let Some(index) = &for_node.object_index_alias {
        ctx.push(", ");
        generate_expression(ctx, index);
    }

    ctx.add_slot_params(&callback_params);

    ctx.push(") => {");
    ctx.indent();
    ctx.newline();
    ctx.push("return ");

    // Find the slot template in the for body
    let slot_template = for_node.children.iter().find_map(|child| {
        if let TemplateChildNode::Element(el) = child {
            if el.tag.as_str() == "template" && has_v_slot(el) {
                return Some(el.as_ref());
            }
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
        if let PropNode::Directive(dir) = p {
            if dir.name.as_str() == "slot" {
                return Some(dir.as_ref());
            }
        }
        None
    });

    if let Some(dir) = slot_dir {
        let slot_name = get_slot_name(dir);

        ctx.push("{");
        ctx.indent();
        ctx.newline();

        // name
        ctx.push("name: \"");
        ctx.push(&escape_js_string(&slot_name));
        ctx.push("\",");
        ctx.newline();

        // fn
        ctx.push("fn: ");
        ctx.use_helper(RuntimeHelper::WithCtx);
        ctx.push(ctx.helper(RuntimeHelper::WithCtx));
        ctx.push("(");

        // Slot props
        let params = if let Some(props_str) = get_slot_props(dir) {
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

/// Generate children for a slot
fn generate_slot_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
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
                    ctx.push(&super::helpers::escape_js_string(&text.content));
                    ctx.push("\"");
                }
                TemplateChildNode::Interpolation(interp) => {
                    ctx.use_helper(RuntimeHelper::ToDisplayString);
                    ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
                    ctx.push("(");
                    generate_slot_expression(ctx, &interp.content);
                    ctx.push(")");
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
fn generate_slot_child_node(ctx: &mut CodegenContext, child: &TemplateChildNode<'_>) {
    match child {
        TemplateChildNode::Text(text) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(\"");
            ctx.push(&super::helpers::escape_js_string(&text.content));
            ctx.push("\")");
        }
        TemplateChildNode::Interpolation(interp) => {
            ctx.use_helper(RuntimeHelper::CreateText);
            ctx.use_helper(RuntimeHelper::ToDisplayString);
            ctx.push(ctx.helper(RuntimeHelper::CreateText));
            ctx.push("(");
            ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
            ctx.push("(");
            // Generate expression, stripping _ctx. prefix for slot params
            generate_slot_expression(ctx, &interp.content);
            ctx.push("), 1 /* TEXT */)");
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
                ctx.push(&exp.content);
                ctx.push("\"");
            } else {
                // Strip _ctx. prefix for slot parameters
                let content = strip_ctx_prefix_for_slot_params(ctx, &exp.content);
                ctx.push(&content);
            }
        }
        ExpressionNode::Compound(comp) => {
            for child in comp.children.iter() {
                match child {
                    crate::ast::CompoundExpressionChild::Simple(exp) => {
                        if exp.is_static {
                            ctx.push("\"");
                            ctx.push(&exp.content);
                            ctx.push("\"");
                        } else {
                            let content = strip_ctx_prefix_for_slot_params(ctx, &exp.content);
                            ctx.push(&content);
                        }
                    }
                    crate::ast::CompoundExpressionChild::String(s) => {
                        ctx.push(s);
                    }
                    crate::ast::CompoundExpressionChild::Symbol(helper) => {
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

#[cfg(test)]
mod tests {
    use super::{is_valid_js_identifier, prefix_slot_defaults};

    #[test]
    fn test_is_valid_js_identifier_valid() {
        assert!(is_valid_js_identifier("foo"));
        assert!(is_valid_js_identifier("_bar"));
        assert!(is_valid_js_identifier("$baz"));
        assert!(is_valid_js_identifier("foo123"));
        assert!(is_valid_js_identifier("camelCase"));
        assert!(is_valid_js_identifier("PascalCase"));
    }

    #[test]
    fn test_is_valid_js_identifier_invalid() {
        assert!(!is_valid_js_identifier("123foo")); // starts with number
        assert!(!is_valid_js_identifier("")); // empty
        assert!(!is_valid_js_identifier("foo-bar")); // contains hyphen
        assert!(!is_valid_js_identifier("foo.bar")); // contains dot
        assert!(!is_valid_js_identifier("foo bar")); // contains space
        assert!(!is_valid_js_identifier("item-header")); // hyphenated slot name
    }

    #[test]
    fn test_hyphenated_slot_names_need_quotes() {
        assert!(!is_valid_js_identifier("item-header"));
        assert!(!is_valid_js_identifier("card-body"));
        assert!(!is_valid_js_identifier("main-content"));
        assert!(!is_valid_js_identifier("list-item"));
    }

    #[test]
    fn test_regular_slot_names_are_valid_identifiers() {
        assert!(is_valid_js_identifier("default"));
        assert!(is_valid_js_identifier("header"));
        assert!(is_valid_js_identifier("footer"));
        assert!(is_valid_js_identifier("content"));
    }

    #[test]
    fn test_prefix_slot_defaults() {
        // Default values should get _ctx. prefix
        assert_eq!(
            prefix_slot_defaults("{ item = defaultItem }"),
            "{ item = _ctx.defaultItem }"
        );
        assert_eq!(prefix_slot_defaults("{ count = 0 }"), "{ count = 0 }");
        assert_eq!(
            prefix_slot_defaults("{ name = 'test' }"),
            "{ name = 'test' }"
        );
        // Literals should not be prefixed
        assert_eq!(prefix_slot_defaults("{ x = true }"), "{ x = true }");
        assert_eq!(prefix_slot_defaults("{ x = false }"), "{ x = false }");
        assert_eq!(prefix_slot_defaults("{ x = null }"), "{ x = null }");
        assert_eq!(
            prefix_slot_defaults("{ x = undefined }"),
            "{ x = undefined }"
        );
    }
}
