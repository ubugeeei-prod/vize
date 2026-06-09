//! Directive-to-prop generation (v-bind, v-on, v-model, v-html, v-text).

use crate::ast::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext,
    expression::{generate_expression, generate_simple_expression},
    helpers::{camelize, escape_js_string, is_constant_simple_expression, is_valid_js_identifier},
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

/// A static class/style attribute that will be merged with a dynamic
/// `:class`/`:style` binding, plus whether the static value appears before
/// the dynamic one in source order (Vue preserves source order in the merged
/// array).
#[derive(Clone, Copy, Default)]
pub struct StaticMerge<'a> {
    pub class: Option<&'a str>,
    pub class_before: bool,
    pub style: Option<&'a str>,
    pub style_before: bool,
}

impl<'a> StaticMerge<'a> {
    /// Build the merge metadata from an element's props in source order.
    pub fn from_props(props: &'a [crate::ast::PropNode<'a>]) -> Self {
        let mut merge = StaticMerge::default();
        let mut class_index = None;
        let mut style_index = None;
        for (index, prop) in props.iter().enumerate() {
            match prop {
                crate::ast::PropNode::Attribute(attr) => {
                    if attr.name == "class" && merge.class.is_none() {
                        merge.class = attr.value.as_ref().map(|v| v.content.as_str());
                        class_index = Some(index);
                    } else if attr.name == "style" && merge.style.is_none() {
                        merge.style = attr.value.as_ref().map(|v| v.content.as_str());
                        style_index = Some(index);
                    }
                }
                crate::ast::PropNode::Directive(dir) => {
                    if dir.name == "bind"
                        && let Some(ExpressionNode::Simple(exp)) = &dir.arg
                        && exp.is_static
                    {
                        if exp.content == "class" && class_index.is_some_and(|i| i < index) {
                            merge.class_before = true;
                        } else if exp.content == "style" && style_index.is_some_and(|i| i < index) {
                            merge.style_before = true;
                        }
                    }
                }
            }
        }
        merge
    }
}

/// Generate directive as prop with optional static class/style merging
pub fn generate_directive_prop_with_static(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    generate_directive_prop_with_static_key_casing(
        ctx,
        dir,
        static_merge,
        StaticBindKeyCasing::Preserve,
    );
}

/// Generate a directive prop for a `<slot>` outlet.
///
/// Vue camelizes static slot prop keys before passing them to renderSlot.
pub fn generate_slot_outlet_directive_prop_with_static(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    generate_directive_prop_with_static_key_casing(
        ctx,
        dir,
        static_merge,
        StaticBindKeyCasing::Camelize,
    );
}

#[derive(Clone, Copy)]
enum StaticBindKeyCasing {
    Preserve,
    Camelize,
}

fn generate_directive_prop_with_static_key_casing(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
    static_key_casing: StaticBindKeyCasing,
) {
    match dir.name.as_str() {
        "bind" => {
            generate_vbind_prop(ctx, dir, static_merge, static_key_casing);
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
    static_merge: StaticMerge<'_>,
    static_key_casing: StaticBindKeyCasing,
) {
    let static_class = static_merge.class;
    let static_style = static_merge.style;
    let mut is_class = false;
    let mut is_style = false;

    // Check for modifiers
    let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
    let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");
    let has_attr = dir.modifiers.iter().any(|m| m.content == "attr");

    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if !exp.is_static {
            // Dynamic attribute name. Modifiers transform the computed key:
            //   (none)  -> [<expr> || ""]
            //   .camel  -> [_camelize(<expr> || "")]
            //   .prop   -> [`.${<expr> || ""}`]
            //   .attr   -> [`^${<expr> || ""}`]
            let emit_key_expr = |ctx: &mut CodegenContext| {
                // If the expression doesn't already have a prefix, add _ctx.
                let content = exp.content.as_str();
                if let Some(local) = content
                    .strip_prefix("_ctx.")
                    .filter(|local| ctx.is_slot_param(local))
                {
                    ctx.push(local);
                } else if content.contains('.')
                    || content.starts_with('_')
                    || content.starts_with('$')
                    || content.contains('`')
                    || content.contains('(')
                {
                    // Template literal or already prefixed expression
                    // For template literals, wrap with parens and prefix inner identifiers
                    if content.starts_with('`') {
                        ctx.push("(");
                        let prefixed =
                            super::super::expression::generate_simple_expression_with_prefix(
                                ctx, content,
                            );
                        ctx.push(&prefixed);
                        ctx.push(")");
                    } else {
                        generate_simple_expression(ctx, exp);
                    }
                } else {
                    if ctx.is_slot_param(content) {
                        ctx.push(content);
                    } else {
                        ctx.push("_ctx.");
                        ctx.push(content);
                    }
                }
            };

            ctx.push("[");
            if has_camel {
                ctx.use_helper(RuntimeHelper::Camelize);
                ctx.push("_camelize(");
                emit_key_expr(ctx);
                ctx.push(" || \"\")");
            } else if has_prop {
                ctx.push("`.${");
                emit_key_expr(ctx);
                ctx.push(" || \"\"}`");
            } else if has_attr {
                ctx.push("`^${");
                emit_key_expr(ctx);
                ctx.push(" || \"\"}`");
            } else {
                emit_key_expr(ctx);
                ctx.push(" || \"\"");
            }
            ctx.push("]: ");
        } else {
            let key = &exp.content;
            is_class = key == "class";
            is_style = key == "style";

            // Transform key based on modifiers
            let base_key: vize_carton::String =
                if has_camel || matches!(static_key_casing, StaticBindKeyCasing::Camelize) {
                    camelize(key)
                } else {
                    key.to_compact_string()
                };

            let transformed_key: vize_carton::String = if has_prop {
                // Add . prefix for DOM property binding
                let mut name = String::with_capacity(1 + base_key.len());
                name.push('.');
                name.push_str(&base_key);
                name
            } else if has_attr {
                // Add ^ prefix for attribute binding
                let mut name = String::with_capacity(1 + base_key.len());
                name.push('^');
                name.push_str(&base_key);
                name
            } else {
                base_key
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
        if is_class {
            if !ctx.skip_normalize {
                ctx.use_helper(RuntimeHelper::NormalizeClass);
                ctx.push("_normalizeClass(");
            }
            // Merge static class if present (needed even inside mergeProps).
            // The array order follows source order: `class` before `:class`
            // yields `["static", dynamic]`, otherwise `[dynamic, "static"]`.
            if let Some(static_val) = static_class {
                ctx.push("[");
                if static_merge.class_before {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(static_val));
                    ctx.push("\", ");
                    generate_expression(ctx, exp);
                } else {
                    generate_expression(ctx, exp);
                    ctx.push(", \"");
                    ctx.push(&escape_js_string(static_val));
                    ctx.push("\"");
                }
                ctx.push("]");
            } else {
                generate_expression(ctx, exp);
            }
            if !ctx.skip_normalize {
                ctx.push(")");
            }
        } else if is_style {
            // Skip normalizeStyle for static literal expressions (e.g., { color: 'red' }).
            // `is_static_expression` runs a full oxc parse, so the `&&` short-circuit
            // keeps it off the hot path for every non-:style v-bind (the common case)
            // and even for :style when normalization is already skipped.
            let needs_normalize = !ctx.skip_normalize && !is_static_expression(exp, ctx);
            if needs_normalize {
                ctx.use_helper(RuntimeHelper::NormalizeStyle);
                ctx.push("_normalizeStyle(");
            }
            // Merge static style if present (needed even inside mergeProps).
            // The array order follows source order, like class merging above.
            if let Some(static_val) = static_style {
                let emit_static_style = |ctx: &mut CodegenContext| {
                    ctx.push("{");
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
                    ctx.push("}");
                };
                ctx.push("[");
                if static_merge.style_before {
                    emit_static_style(ctx);
                    ctx.push(", ");
                    generate_expression(ctx, exp);
                } else {
                    generate_expression(ctx, exp);
                    ctx.push(", ");
                    emit_static_style(ctx);
                }
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
    let is_dynamic_event = if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        !exp.is_static
    } else {
        false
    };

    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if is_dynamic_event {
            // Dynamic event name: [_toHandlerKey(_ctx.event)]:
            ctx.use_helper(RuntimeHelper::ToHandlerKey);
            ctx.push("[");
            ctx.push(ctx.helper(RuntimeHelper::ToHandlerKey));
            ctx.push("(");
            let content = exp.content.as_str();
            if let Some(local) = content
                .strip_prefix("_ctx.")
                .filter(|local| ctx.is_slot_param(local))
            {
                ctx.push(local);
            } else if content.contains('.') || content.starts_with('_') || content.starts_with('$')
            {
                generate_simple_expression(ctx, exp);
            } else if ctx.is_slot_param(content) {
                ctx.push(content);
            } else {
                ctx.push("_ctx.");
                ctx.push(content);
            }
            ctx.push(")]: ");
        } else {
            // Mirror Vue's event-name casing rule (transforms/vOn.ts), including
            // mouse-button event renaming, `vue:` vnode hooks, and the `on:`
            // case-preserving form for custom-element events on plain elements.
            // The `on:` case-preserving form only applies to user-authored v-on
            // directives (those carry a `raw_name`). Compiler-synthesized handlers
            // like v-model's `update:modelValue` always camelize.
            let on_plain_element = ctx.props_is_plain_element && dir.raw_name.is_some();
            let event_name = super::events::von_event_key_for(
                exp.content.as_str(),
                on_plain_element,
                dir.modifiers.iter().map(|m| m.content.as_str()),
            );

            let needs_quotes = !is_valid_js_identifier(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
        }
    }

    super::events::generate_von_handler_value(ctx, dir);
}

/// Generate dynamic v-model on component as props
fn generate_vmodel_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    // Handle dynamic v-model on component
    // Generate: [_ctx.prop]: _ctx.value, ["onUpdate:" + _ctx.prop]: handler
    if let Some(ExpressionNode::Simple(arg_exp)) = &dir.arg
        && !arg_exp.is_static
    {
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
