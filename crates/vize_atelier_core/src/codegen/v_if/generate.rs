//! Props and helper utilities for v-if branch code generation.
//!
//! Contains prop object generation, static class/style extraction,
//! dynamic binding checks, event key deduplication, and spread detection.

use crate::ast::{DirectiveNode, ElementNode, ExpressionNode, IfBranchNode, PropNode};

use super::super::{
    context::CodegenContext,
    helpers::{camelize, capitalize_first, escape_js_string, is_valid_js_identifier},
    props::{generate_directive_prop_with_static, is_supported_directive},
};
use super::generate_if_branch_key;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Check if prop should be skipped for v-if branch element.
pub(super) fn should_skip_prop_for_if(
    p: &PropNode<'_>,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
) -> bool {
    match p {
        PropNode::Attribute(attr) => {
            // Skip static class if there's a dynamic :class (will be merged)
            if attr.name == "class" && has_dynamic_class {
                return true;
            }
            // Skip static style if there's a dynamic :style (will be merged)
            if attr.name == "style" && has_dynamic_style {
                return true;
            }
            false
        }
        PropNode::Directive(dir) => {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if arg.content == "key" {
                        return true;
                    }
                }
            }
            // Skip v-if/v-else-if/v-else directives
            if matches!(dir.name.as_str(), "if" | "else-if" | "else") {
                return true;
            }
            false
        }
    }
}

/// Extract static class and style values from element props.
pub(super) fn extract_static_class_style<'a>(
    el: &'a ElementNode<'_>,
) -> (Option<&'a str>, Option<&'a str>) {
    let mut static_class = None;
    let mut static_style = None;
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if attr.name == "class" {
                if let Some(val) = &attr.value {
                    static_class = Some(val.content.as_str());
                }
            } else if attr.name == "style" {
                if let Some(val) = &attr.value {
                    static_style = Some(val.content.as_str());
                }
            }
        }
    }
    (static_class, static_style)
}

/// Check if element has dynamic `:class` binding.
pub(super) fn has_dynamic_class(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    return arg.content == "class";
                }
            }
        }
        false
    })
}

/// Check if element has dynamic `:style` binding.
pub(super) fn has_dynamic_style(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    return arg.content == "style";
                }
            }
        }
        false
    })
}

/// Generate a single prop for v-if branch element.
pub(super) fn generate_single_prop_for_if(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_class: Option<&str>,
    static_style: Option<&str>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let ref_binding_type = if attr.name == "ref" && ctx.options.inline {
                attr.value.as_ref().and_then(|v| {
                    ctx.options
                        .binding_metadata
                        .as_ref()
                        .and_then(|m| m.bindings.get(v.content.as_str()).copied())
                })
            } else {
                None
            };
            let needs_ref_key = matches!(
                ref_binding_type,
                Some(
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupRef
                        | crate::options::BindingType::SetupMaybeRef
                )
            );

            if needs_ref_key {
                let ref_name = &attr.value.as_ref().unwrap().content;
                ctx.push("ref_key: \"");
                ctx.push(ref_name);
                ctx.push("\", ref: ");
                ctx.push(ref_name);
                return;
            }

            let needs_quotes = !is_valid_js_identifier(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                if ref_binding_type.is_some() {
                    ctx.push(&value.content);
                } else {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content.as_str()));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            generate_directive_prop_with_static(ctx, dir, static_class, static_style);
        }
    }
}

/// Generate props object for v-if branch (with key and other props).
#[allow(clippy::too_many_arguments)]
pub(super) fn generate_if_branch_props_object(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
    static_class: Option<&str>,
    static_style: Option<&str>,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
) {
    // Check if there are other props besides key (skip excluded ones)
    let has_other_props = el.props.iter().any(|p| {
        // Skip unsupported directives (v-slot, v-tooltip, custom directives, etc.)
        if let PropNode::Directive(dir) = p {
            if !is_supported_directive(dir) {
                return false;
            }
        }
        !should_skip_prop_for_if(p, has_dynamic_class, has_dynamic_style)
            && !is_vbind_spread_prop(p)
            && !is_von_spread_prop(p)
    });
    // For component elements, skip_scope_id suppresses the attribute.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };
    let has_scope = scope_id.is_some();

    if !has_other_props && !has_scope {
        // Key-only: use inline format { key: N }
        ctx.push("{ key: ");
        generate_if_branch_key(ctx, branch, branch_index);
        ctx.push(" }");
        return;
    }

    // Multiline format for key + other props
    ctx.push("{");
    ctx.indent();
    ctx.newline();
    ctx.push("key: ");
    generate_if_branch_key(ctx, branch, branch_index);

    let mut seen_events: FxHashSet<String> = FxHashSet::default();

    for prop in el.props.iter() {
        // Skip unsupported directives (v-slot, v-tooltip, custom directives, etc.)
        if let PropNode::Directive(dir) = prop {
            if !is_supported_directive(dir) {
                continue;
            }
        }
        if should_skip_prop_for_if(prop, has_dynamic_class, has_dynamic_style) {
            continue;
        }
        if is_vbind_spread_prop(prop) {
            continue;
        }
        if is_von_spread_prop(prop) {
            continue;
        }
        if let PropNode::Directive(dir) = prop {
            if dir.name == "on" {
                if let Some(key) = get_static_event_key(dir) {
                    if !seen_events.insert(key) {
                        continue;
                    }
                }
            }
        }
        ctx.push(",");
        ctx.newline();
        generate_single_prop_for_if(ctx, prop, static_class, static_style);
    }

    // Add scope_id for scoped CSS
    if let Some(ref scope_id) = scope_id {
        ctx.push(",");
        ctx.newline();
        ctx.push("\"");
        ctx.push(scope_id);
        ctx.push("\": \"\"");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

/// Check if element has v-bind object spread.
pub(super) fn has_vbind_spread(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| is_vbind_spread_prop(p))
}

/// Check if prop is a v-bind object spread (`v-bind="obj"`).
pub(super) fn is_vbind_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "bind" && dir.arg.is_none();
    }
    false
}

/// Check if element has v-on object spread (`v-on="obj"`).
pub(super) fn has_von_spread(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| is_von_spread_prop(p))
}

/// Check if prop is a v-on object spread (`v-on="obj"`).
pub(super) fn is_von_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "on" && dir.arg.is_none();
    }
    false
}

/// Compute static event prop key for dedupe (e.g., `onClick`, `onUpdate:modelValue`).
fn get_static_event_key(dir: &DirectiveNode<'_>) -> Option<String> {
    let arg = dir.arg.as_ref()?;
    let ExpressionNode::Simple(exp) = arg else {
        return None;
    };
    if !exp.is_static {
        return None;
    }

    let mut event_name = exp.content.as_str();
    let is_keyboard_event = matches!(event_name, "keydown" | "keyup" | "keypress");

    let mut event_option_modifiers: Vec<&str> = Vec::new();
    let mut system_modifiers: Vec<&str> = Vec::new();

    for modifier in dir.modifiers.iter() {
        let mod_name = modifier.content.as_str();
        match mod_name {
            "capture" | "once" | "passive" => {
                event_option_modifiers.push(mod_name);
            }
            "left" | "right" => {
                if !is_keyboard_event {
                    system_modifiers.push(mod_name);
                }
            }
            "middle" => {
                system_modifiers.push(mod_name);
            }
            _ => {}
        }
    }

    let has_right_modifier = system_modifiers.contains(&"right");
    let has_middle_modifier = system_modifiers.contains(&"middle");

    if event_name == "click" && has_right_modifier {
        event_name = "contextmenu";
    } else if event_name == "click" && has_middle_modifier {
        event_name = "mouseup";
    }

    let mut key = if event_name.contains(':') {
        let parts: Vec<&str> = event_name.splitn(2, ':').collect();
        if parts.len() == 2 {
            let first_part = camelize(parts[0]);
            let mut name = String::from("on");
            if let Some(first) = first_part.chars().next() {
                name.push_str(&first.to_uppercase().to_compact_string());
                name.push_str(&first_part[first.len_utf8()..]);
            }
            name.push(':');
            name.push_str(parts[1]);
            name
        } else {
            String::from(event_name)
        }
    } else {
        let camelized = camelize(event_name);
        let mut name = String::from("on");
        if let Some(first) = camelized.chars().next() {
            name.push_str(&first.to_uppercase().to_compact_string());
            name.push_str(&camelized[first.len_utf8()..]);
        }
        name
    };

    for opt_mod in &event_option_modifiers {
        key.push_str(&capitalize_first(opt_mod));
    }

    Some(key)
}
