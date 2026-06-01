//! Event-related props generation (v-on merging and handler generation).

use crate::ast::{DirectiveNode, ExpressionNode, PropNode, RuntimeHelper};

use super::super::{
    context::CodegenContext,
    expression::generate_event_handler,
    helpers::{camelize, capitalize_first},
};
use vize_carton::String;

/// Compute the prop key for a static v-on event, mirroring Vue's
/// `transforms/vOn.ts` casing rules.
///
/// * `raw` is the static event argument (e.g. `click`, `foo-bar`, `vue:mounted`).
/// * `is_plain_element` distinguishes native elements from components/slots.
/// * `option_modifiers` are the `capture`/`once`/`passive` modifiers appended
///   (capitalized) to the resulting key when `with_option_modifiers` is set.
pub fn von_event_key_for(
    raw: &str,
    is_plain_element: bool,
    modifiers: impl Iterator<Item = impl AsRef<str>>,
) -> String {
    // `@click.right` -> contextmenu, `@click.middle` -> mouseup. Detect first.
    let mut has_right = false;
    let mut has_middle = false;
    let mut option_modifiers: Vec<&'static str> = Vec::new();
    let collected: Vec<String> = modifiers.map(|m| m.as_ref().into()).collect();
    for m in &collected {
        match m.as_str() {
            "right" => has_right = true,
            "middle" => has_middle = true,
            "capture" => option_modifiers.push("capture"),
            "once" => option_modifiers.push("once"),
            "passive" => option_modifiers.push("passive"),
            _ => {}
        }
    }

    let mut raw_name = raw;
    if raw_name == "click" && has_right {
        raw_name = "contextmenu";
    } else if raw_name == "click" && has_middle {
        raw_name = "mouseup";
    }

    // `vue:mounted` -> `vnode-mounted`
    let vnode_owned: String;
    if let Some(rest) = raw_name.strip_prefix("vue:") {
        let mut s = String::with_capacity(rest.len() + 6);
        s.push_str("vnode-");
        s.push_str(rest);
        vnode_owned = s;
        raw_name = &vnode_owned;
    }

    let mut name = if !is_plain_element
        || raw_name.starts_with("vnode")
        || !raw_name.chars().any(|c| c.is_ascii_uppercase())
    {
        let camelized = camelize(raw_name);
        let mut n = String::with_capacity(camelized.len() + 2);
        n.push_str("on");
        n.push_str(&capitalize_first(&camelized));
        n
    } else {
        let mut n = String::with_capacity(raw_name.len() + 3);
        n.push_str("on:");
        n.push_str(raw_name);
        n
    };

    for opt in &option_modifiers {
        name.push_str(&capitalize_first(opt));
    }
    name
}

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
        if let PropNode::Directive(dir) = p
            && let Some(key) = get_von_event_key(dir)
            && key == target_event_key
        {
            if handler_idx > 0 {
                ctx.push(", ");
            }
            generate_von_handler_value(ctx, dir);
            handler_idx += 1;
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
