//! Props and helper utilities for v-if branch code generation.
//!
//! Contains single-prop generation, event key deduplication, and spread detection.

use crate::{DirectiveNode, ExpressionNode, PropNode};

use super::super::{
    context::CodegenContext,
    helpers::{camelize, capitalize_first, escape_js_string, is_valid_js_identifier},
    props::{StaticMerge, generate_directive_prop_with_static},
};
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
            if dir.name == "bind"
                && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                && arg.content == "key"
            {
                return true;
            }
            // Skip v-if/v-else-if/v-else directives
            if matches!(dir.name.as_str(), "if" | "else-if" | "else") {
                return true;
            }
            false
        }
    }
}

/// Generate a single prop for v-if branch element.
pub(super) fn generate_single_prop_for_if(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let ref_value = if attr.name == "ref" && ctx.options.inline {
                attr.value.as_ref()
            } else {
                None
            };
            let ref_binding_type = ref_value.and_then(|v| {
                ctx.options
                    .binding_metadata
                    .as_ref()
                    .and_then(|m| m.bindings.get(v.content.as_str()).copied())
            });
            let should_ref_runtime_binding = matches!(
                ref_binding_type,
                Some(
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupRef
                        | crate::options::BindingType::SetupMaybeRef
                )
            );
            let needs_ref_for = attr.name == "ref" && ctx.in_v_for;

            if let (true, Some(ref_value)) = (should_ref_runtime_binding, ref_value) {
                let ref_name = &ref_value.content;
                if needs_ref_for {
                    ctx.push("ref_for: true, ");
                }
                ctx.push("ref_key: \"");
                ctx.push(ref_name);
                ctx.push("\", ref: ");
                ctx.push(ref_name);
                return;
            }

            if needs_ref_for {
                ctx.push("ref_for: true, ");
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
                if should_ref_runtime_binding {
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
            generate_directive_prop_with_static(ctx, dir, static_merge);
        }
    }
}

/// Check if prop is a v-bind object spread (`v-bind="obj"`).
pub(super) fn is_vbind_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "bind" && dir.arg.is_none();
    }
    false
}

/// Check if prop is a v-on object spread (`v-on="obj"`).
pub(super) fn is_von_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "on" && dir.arg.is_none();
    }
    false
}

/// Compute static event prop key for dedupe (e.g., `onClick`, `onUpdate:modelValue`).
pub(super) fn get_static_event_key(dir: &DirectiveNode<'_>) -> Option<String> {
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
            "left" | "right" if !is_keyboard_event => {
                system_modifiers.push(mod_name);
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
