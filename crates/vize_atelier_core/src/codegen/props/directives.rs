//! Directive-to-prop generation (v-bind, v-on, v-model, v-html, v-text).

use crate::ast::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext,
    expression::{generate_event_handler, generate_expression, generate_simple_expression},
    helpers::{
        camelize, capitalize_first, escape_js_string, is_constant_simple_expression,
        is_valid_js_identifier,
    },
};
use vize_carton::String;
use vize_carton::ToCompactString;

/// Check if an expression is a static literal (no runtime identifiers).
/// Returns true for: object literals, array literals, string literals, numbers
/// that don't reference any runtime variables (no `_ctx.` after processing).
fn is_static_expression(exp: &ExpressionNode<'_>, ctx: &CodegenContext) -> bool {
    match exp {
        ExpressionNode::Simple(simple) => {
            is_constant_simple_expression(simple, ctx.options.binding_metadata.as_ref())
        }
        ExpressionNode::Compound(_) => false,
    }
}

/// Check if a directive will produce valid output
pub fn is_supported_directive(dir: &DirectiveNode<'_>) -> bool {
    // v-model with dynamic arg on components needs special props handling
    // Static v-model is handled via withDirectives for native elements or transformed for components
    if dir.name == "model" {
        return dir.arg.as_ref().is_some_and(|arg| match arg {
            ExpressionNode::Simple(exp) => !exp.is_static,
            ExpressionNode::Compound(_) => true,
        });
    }
    matches!(dir.name.as_str(), "bind" | "on" | "html" | "text")
}

/// Generate directive as prop with optional static class/style merging
pub fn generate_directive_prop_with_static(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_class: Option<&str>,
    static_style: Option<&str>,
) {
    match dir.name.as_str() {
        "bind" => {
            generate_vbind_prop(ctx, dir, static_class, static_style);
        }
        "on" => {
            generate_von_prop(ctx, dir);
        }
        "model" => {
            generate_vmodel_prop(ctx, dir);
        }
        "html" => {
            // v-html="rawHtml" -> innerHTML: _ctx.rawHtml
            ctx.push("innerHTML: ");
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            } else {
                ctx.push("undefined");
            }
        }
        "text" => {
            // v-text="message" -> textContent: _toDisplayString(_ctx.message)
            ctx.use_helper(RuntimeHelper::ToDisplayString);
            ctx.push("textContent: ");
            ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
            ctx.push("(");
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            } else {
                ctx.push("undefined");
            }
            ctx.push(")");
        }
        _ => {
            // Other directives are skipped by is_supported_directive()
            // This case should not be reached in normal operation
        }
    }
}

/// Generate v-bind directive as a prop
fn generate_vbind_prop(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_class: Option<&str>,
    static_style: Option<&str>,
) {
    let mut is_class = false;
    let mut is_style = false;

    // Check for modifiers
    let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
    let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");
    let has_attr = dir.modifiers.iter().any(|m| m.content == "attr");

    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if !exp.is_static {
            // Dynamic attribute name: [_ctx.expr || ""]: value
            ctx.push("[");
            // If the expression doesn't already have a prefix, add _ctx.
            let content = exp.content.as_str();
            if content.contains('.')
                || content.starts_with('_')
                || content.starts_with('$')
                || content.contains('`')
                || content.contains('(')
            {
                // Template literal or already prefixed expression
                // For template literals, wrap with parens and prefix inner identifiers
                if content.starts_with('`') {
                    ctx.push("(");
                    // Prefix identifiers inside template literals with _ctx.
                    let prefixed = super::super::expression::generate_simple_expression_with_prefix(
                        ctx, content,
                    );
                    ctx.push(&prefixed);
                    ctx.push(")");
                } else {
                    generate_simple_expression(ctx, exp);
                }
            } else {
                ctx.push("_ctx.");
                ctx.push(content);
            }
            ctx.push(" || \"\"]: ");
        } else {
            let key = &exp.content;
            is_class = key == "class";
            is_style = key == "style";

            // Transform key based on modifiers
            let transformed_key: vize_carton::String = if has_camel {
                // Convert kebab-case to camelCase
                camelize(key)
            } else if has_prop {
                // Add . prefix for DOM property binding
                let mut name = String::with_capacity(1 + key.len());
                name.push('.');
                name.push_str(key);
                name
            } else if has_attr {
                // Add ^ prefix for attribute binding
                let mut name = String::with_capacity(1 + key.len());
                name.push('^');
                name.push_str(key);
                name
            } else {
                key.to_compact_string()
            };

            let needs_quotes = !is_valid_js_identifier(&transformed_key);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&transformed_key);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
        }
    }
    if let Some(exp) = &dir.exp {
        // Check if expression is a static literal (no runtime references)
        let is_static_literal = is_static_expression(exp, ctx);

        if is_class {
            if !ctx.skip_normalize {
                ctx.use_helper(RuntimeHelper::NormalizeClass);
                ctx.push("_normalizeClass(");
            }
            // Merge static class if present (needed even inside mergeProps)
            if let Some(static_val) = static_class {
                ctx.push("[\"");
                ctx.push(&escape_js_string(static_val));
                ctx.push("\", ");
                generate_expression(ctx, exp);
                ctx.push("]");
            } else {
                generate_expression(ctx, exp);
            }
            if !ctx.skip_normalize {
                ctx.push(")");
            }
        } else if is_style {
            // Skip normalizeStyle for static literal expressions (e.g., { color: 'red' })
            let needs_normalize = !ctx.skip_normalize && !is_static_literal;
            if needs_normalize {
                ctx.use_helper(RuntimeHelper::NormalizeStyle);
                ctx.push("_normalizeStyle(");
            }
            // Merge static style if present (needed even inside mergeProps)
            if let Some(static_val) = static_style {
                ctx.push("[{");
                // Parse static style and convert to object
                for (i, part) in static_val
                    .split(';')
                    .filter(|s| !s.trim().is_empty())
                    .enumerate()
                {
                    if i > 0 {
                        ctx.push(",");
                    }
                    let parts: Vec<&str> = part.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim();
                        ctx.push("\"");
                        ctx.push(key);
                        ctx.push("\":\"");
                        ctx.push(value);
                        ctx.push("\"");
                    }
                }
                ctx.push("}, ");
                generate_expression(ctx, exp);
                ctx.push("]");
            } else {
                generate_expression(ctx, exp);
            }
            if needs_normalize {
                ctx.push(")");
            }
        } else {
            generate_expression(ctx, exp);
        }
    } else {
        ctx.push("undefined");
    }
}

/// Generate v-on directive as a prop
fn generate_von_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    // Get event name first to determine context for modifiers
    let (event_name, is_dynamic_event) = if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        (exp.content.as_str(), !exp.is_static)
    } else {
        ("", false)
    };

    // Check if this is a keyboard event (for context-dependent modifiers)
    let is_keyboard_event = matches!(event_name, "keydown" | "keyup" | "keypress");

    // Collect modifiers into categories
    let mut event_option_modifiers: Vec<&str> = Vec::new();
    let mut system_modifiers: Vec<&str> = Vec::new();
    let mut key_modifiers: Vec<&str> = Vec::new();

    for modifier in dir.modifiers.iter() {
        let mod_name = modifier.content.as_str();
        match mod_name {
            // Event option modifiers - appended to event name
            "capture" | "once" | "passive" => {
                event_option_modifiers.push(mod_name);
            }
            // "native" modifier is a no-op in Vue 3 (removed)
            "native" => {}
            // Context-dependent: left/right are arrow keys on keyboard events,
            // mouse buttons on click events
            "left" | "right" => {
                if is_keyboard_event {
                    key_modifiers.push(mod_name);
                } else {
                    system_modifiers.push(mod_name);
                }
            }
            // System modifiers - wrapped with withModifiers
            "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "middle"
            | "exact" => {
                system_modifiers.push(mod_name);
            }
            // Key modifiers - wrapped with withKeys
            "enter" | "tab" | "delete" | "esc" | "space" | "up" | "down" => {
                key_modifiers.push(mod_name);
            }
            _ => {
                // Unknown modifiers (including numeric keycodes) are treated as key modifiers
                key_modifiers.push(mod_name);
            }
        }
    }

    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if is_dynamic_event {
            // Dynamic event name: [_toHandlerKey(_ctx.event)]:
            ctx.use_helper(RuntimeHelper::ToHandlerKey);
            ctx.push("[");
            ctx.push(ctx.helper(RuntimeHelper::ToHandlerKey));
            ctx.push("(");
            let content = exp.content.as_str();
            if content.contains('.') || content.starts_with('_') || content.starts_with('$') {
                generate_simple_expression(ctx, exp);
            } else {
                ctx.push("_ctx.");
                ctx.push(content);
            }
            ctx.push(")]: ");
        } else {
            let mut event_name = exp.content.as_str();

            // Special mouse button modifiers that change the event name
            // @click.right -> onContextmenu, @click.middle -> onMouseup
            let has_right_modifier = system_modifiers.contains(&"right");
            let has_middle_modifier = system_modifiers.contains(&"middle");

            if event_name == "click" && has_right_modifier {
                event_name = "contextmenu";
            } else if event_name == "click" && has_middle_modifier {
                event_name = "mouseup";
            }

            // Handle special event names like "update:modelValue"
            if event_name.contains(':') {
                // Event name with colon needs quotes (e.g., "onUpdate:modelValue")
                let parts: Vec<&str> = event_name.splitn(2, ':').collect();
                if parts.len() == 2 {
                    ctx.push("\"on");
                    // Capitalize the first part (e.g., "update" -> "Update")
                    // Also convert kebab-case to camelCase
                    let first_part_camelized = camelize(parts[0]);
                    if let Some(first) = first_part_camelized.chars().next() {
                        ctx.push(&first.to_uppercase().to_compact_string());
                        ctx.push(&first_part_camelized[first.len_utf8()..]);
                    }
                    ctx.push(":");
                    ctx.push(parts[1]);
                    // Append event option modifiers
                    for opt_mod in &event_option_modifiers {
                        ctx.push(&capitalize_first(opt_mod));
                    }
                    ctx.push("\": ");
                }
            } else {
                // Simple event names don't need quotes (onUpdate, onClick)
                // Convert kebab-case to camelCase first (e.g., "select-koma" -> "selectKoma")
                let camelized = camelize(event_name);
                ctx.push("on");
                // Capitalize first letter of camelized name
                if let Some(first) = camelized.chars().next() {
                    ctx.push(&first.to_uppercase().to_compact_string());
                    ctx.push(&camelized[first.len_utf8()..]);
                }
                // Append event option modifiers (Capture, Once, Passive)
                for opt_mod in &event_option_modifiers {
                    ctx.push(&capitalize_first(opt_mod));
                }
                ctx.push(": ");
            }
        }
    }

    // Generate handler with optional withModifiers/withKeys wrappers
    // Order: _withKeys(_withModifiers(handler, [system_mods]), [key_mods])
    let has_system_mods = !system_modifiers.is_empty();
    let has_key_mods = !key_modifiers.is_empty();

    // Check if this handler needs caching
    // When cache_handlers is true, handlers are cached UNLESS the handler is a
    // setup-const binding (stable reference, no need for caching)
    // Pattern: _cache[n] || (_cache[n] = handler)
    // Simple identifiers get safety wrapper: (...args) => (_ctx.handler && _ctx.handler(...args))
    // Inline expressions get: $event => (expression)
    let is_const_handler = dir.exp.as_ref().is_some_and(|exp| {
        if let ExpressionNode::Simple(simple) = exp {
            if !simple.is_static {
                let content = simple.content.trim();
                // Check if content is a simple identifier that's a setup-const binding
                if crate::transforms::is_simple_identifier(content) {
                    if let Some(ref metadata) = ctx.options.binding_metadata {
                        return matches!(
                            metadata.bindings.get(content),
                            Some(crate::options::BindingType::SetupConst)
                        );
                    }
                }
            }
        }
        false
    });
    let needs_cache = ctx.options.cache_handlers && dir.exp.is_some() && !is_const_handler;

    if needs_cache {
        let cache_index = ctx.next_cache_index();
        ctx.push("_cache[");
        ctx.push(&cache_index.to_compact_string());
        ctx.push("] || (_cache[");
        ctx.push(&cache_index.to_compact_string());
        ctx.push("] = ");
    }

    if has_key_mods {
        ctx.use_helper(RuntimeHelper::WithKeys);
        ctx.push("_withKeys(");
    }

    if has_system_mods {
        ctx.use_helper(RuntimeHelper::WithModifiers);
        ctx.push("_withModifiers(");
    }

    // Generate the actual handler
    if let Some(exp) = &dir.exp {
        generate_event_handler(ctx, exp, needs_cache);
    } else {
        ctx.push("() => {}");
    }

    // Close withModifiers wrapper
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

    // Close withKeys wrapper
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

    // Close cache wrapper
    if needs_cache {
        ctx.push(")");
    }
}

/// Generate dynamic v-model on component as props
fn generate_vmodel_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    // Handle dynamic v-model on component
    // Generate: [_ctx.prop]: _ctx.value, ["onUpdate:" + _ctx.prop]: handler
    if let Some(ExpressionNode::Simple(arg_exp)) = &dir.arg {
        if !arg_exp.is_static {
            let prop_name = &arg_exp.content;
            let value_exp = dir
                .exp
                .as_ref()
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
            if !dir.modifiers.is_empty() {
                ctx.push(",");
                ctx.newline();
                // [_ctx.prop + "Modifiers"]: { modifier: true }
                ctx.push("[_ctx.");
                ctx.push(prop_name);
                ctx.push(" + \"Modifiers\"]: { ");
                for (i, modifier) in dir.modifiers.iter().enumerate() {
                    if i > 0 {
                        ctx.push(", ");
                    }
                    ctx.push(&modifier.content);
                    ctx.push(": true");
                }
                ctx.push(" }");
            }
        }
    }
}
