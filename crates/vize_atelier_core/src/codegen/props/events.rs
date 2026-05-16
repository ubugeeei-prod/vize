//! Event-related props generation (v-on merging and handler generation).

use crate::ast::{DirectiveNode, ExpressionNode, PropNode, RuntimeHelper};

use super::super::{
    context::CodegenContext, expression::generate_event_handler, helpers::camelize,
};
use vize_carton::String;

/// Get the event key for a v-on directive (e.g., "onClick", "onKeyupEnter")
pub(super) fn get_von_event_key(dir: &DirectiveNode<'_>) -> Option<String> {
    if dir.name != "on" {
        return None;
    }
    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if exp.is_static {
            let camelized = camelize(exp.content.as_str());
            let mut key = String::from("on");
            if let Some(first) = camelized.chars().next() {
                key.push(first.to_uppercase().next().unwrap_or(first));
                key.push_str(&camelized[first.len_utf8()..]);
            }
            Some(key)
        } else {
            None // Dynamic events can't be merged
        }
    } else {
        None
    }
}

/// Generate merged event handlers for the same event name as array syntax
/// e.g., onClick: [_ctx.a, _withModifiers(_ctx.b, ["ctrl"])]
pub(super) fn generate_merged_event_handlers(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    target_event_key: &str,
    _static_class: Option<&str>,
    _static_style: Option<&str>,
) {
    // Output the event key name (e.g., "onClick" or "\"onUpdate:modelValue\"")
    // Event names containing ':' need quotes for valid JavaScript
    if target_event_key.contains(':') {
        ctx.push("\"");
        ctx.push(target_event_key);
        ctx.push("\"");
    } else {
        ctx.push(target_event_key);
    }
    ctx.push(": [");

    // Output each handler as an element in the array
    let mut handler_idx = 0;
    for p in props {
        if let PropNode::Directive(dir) = p {
            if let Some(key) = get_von_event_key(dir) {
                if key == target_event_key {
                    if handler_idx > 0 {
                        ctx.push(", ");
                    }
                    generate_von_handler_value(ctx, dir);
                    handler_idx += 1;
                }
            }
        }
    }

    ctx.push("]");
}

/// Generate just the handler value part of a v-on directive (without the key name)
fn generate_von_handler_value(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    // Classify modifiers (same logic as in generate_directive_prop_with_static)
    let event_name = if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        exp.content.as_str()
    } else {
        ""
    };
    let is_keyboard_event = matches!(event_name, "keydown" | "keyup" | "keypress");

    let mut system_modifiers: Vec<&str> = Vec::new();
    let mut key_modifiers: Vec<&str> = Vec::new();

    for modifier in dir.modifiers.iter() {
        let mod_name = modifier.content.as_str();
        match mod_name {
            "capture" | "once" | "passive" | "native" => {}
            "left" | "right" => {
                if is_keyboard_event {
                    key_modifiers.push(mod_name);
                } else {
                    system_modifiers.push(mod_name);
                }
            }
            "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "middle"
            | "exact" => {
                system_modifiers.push(mod_name);
            }
            "enter" | "tab" | "delete" | "esc" | "space" | "up" | "down" => {
                key_modifiers.push(mod_name);
            }
            _ => {
                key_modifiers.push(mod_name);
            }
        }
    }

    let has_system_mods = !system_modifiers.is_empty();
    let has_key_mods = !key_modifiers.is_empty();

    if has_key_mods {
        ctx.use_helper(RuntimeHelper::WithKeys);
        ctx.push("_withKeys(");
    }

    if has_system_mods {
        ctx.use_helper(RuntimeHelper::WithModifiers);
        ctx.push("_withModifiers(");
    }

    if let Some(exp) = &dir.exp {
        generate_event_handler(ctx, exp, false);
    } else {
        ctx.push("() => {}");
    }

    if has_system_mods {
        ctx.push(", [");
        for (i, mod_name) in system_modifiers.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.push("\"");
            ctx.push(mod_name);
            ctx.push("\"");
        }
        ctx.push("])");
    }

    if has_key_mods {
        ctx.push(", [");
        for (i, mod_name) in key_modifiers.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.push("\"");
            ctx.push(mod_name);
            ctx.push("\"");
        }
        ctx.push("])");
    }
}
