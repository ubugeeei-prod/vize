//! Directive-to-prop generation (v-bind, v-on, v-model, v-html, v-text).

use crate::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext,
    expression::{generate_expression, generate_simple_expression},
    helpers::{camelize, escape_js_string, is_constant_simple_expression, is_valid_js_identifier},
};
use super::StaticMerge;
use super::v_model::generate_vmodel_prop;
use vize_carton::{String, ToCompactString};

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
    matches!(dir.name, "bind" | "on" | "html" | "text")
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
    match dir.name {
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
    if dir.arg.is_none() && !ctx.merge_props {
        ctx.push("...");
        if let Some(exp) = &dir.exp {
            generate_expression(ctx, exp);
        } else {
            ctx.push("undefined");
        }
        return;
    }

    let static_class = static_merge.class;
    let static_style = static_merge.style;
    let mut is_class = false;
    let mut is_style = false;

    let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
    let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");
    let has_attr = dir.modifiers.iter().any(|m| m.content == "attr");

    if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
        if !exp.is_static {
            // Dynamic keys compose in Vue order: camelize, then `.prop`, then `.attr`.
            let emit_key_expr = |ctx: &mut CodegenContext| {
                // If the expression doesn't already have a prefix, add _ctx.
                let content = exp.content;
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
                    // For template literals, wrap with parens and prefix inner
                    // identifiers (retained-AST aware, P1-7).
                    if content.starts_with('`') {
                        ctx.push("(");
                        let prefixed = super::super::expression::prefix_context::
                            prefix_identifiers_with_context_node(exp, ctx);
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
            if has_attr {
                ctx.push("`^${");
            }
            if has_prop {
                ctx.push("`.${");
            }
            if has_camel {
                ctx.use_helper(RuntimeHelper::Camelize);
                ctx.push("_camelize(");
            }
            emit_key_expr(ctx);
            ctx.push(" || \"\"");
            if has_camel {
                ctx.push(")");
            }
            if has_prop {
                ctx.push("}`");
            }
            if has_attr {
                ctx.push("}`");
            }
            ctx.push("]: ");
        } else {
            let key = &exp.content;
            is_class = *key == "class";
            is_style = *key == "style";

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

            if transformed_key == "ref" && ctx.in_v_for {
                ctx.push("ref_for: true, ");
            }

            let needs_quotes = !is_valid_js_identifier(&transformed_key);
            if needs_quotes {
                ctx.push("\"");
            }
            // Anchor the generated prop key back to the v-bind argument in
            // source, recording the original (untransformed) symbol so it lands
            // in the v3 `names` array. No-op without `source_map`.
            ctx.record_mapping_named(exp.loc.span.start, exp.content);
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
                    // Mirror Vue's `parseStringStyle` (regex `/;(?![^(]*\))/`): a `;`
                    // inside parentheses (e.g. `url(a;b)`) is not a declaration
                    // separator, and only the first `:` separates key from value.
                    let mut emitted = 0;
                    for declaration in split_style_declarations(static_val) {
                        let declaration = declaration.trim();
                        if declaration.is_empty() {
                            continue;
                        }
                        // Skip orphan parts with no `:`; only push the separating
                        // comma once a valid key/value pair is confirmed.
                        let Some((key, value)) = declaration.split_once(':') else {
                            continue;
                        };
                        if emitted > 0 {
                            ctx.push(",");
                        }
                        emitted += 1;
                        ctx.push("\"");
                        ctx.push(&escape_js_string(key.trim()));
                        ctx.push("\":\"");
                        ctx.push(&escape_js_string(value.trim()));
                        ctx.push("\"");
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

/// Split a static `style` attribute value into declarations on each `;` that is
/// not inside parentheses, mirroring Vue's `parseStringStyle` regex
/// `/;(?![^(]*\))/`. A `;` inside `url(a;b)` therefore stays within one
/// declaration instead of being treated as a separator.
fn split_style_declarations(value: &str) -> Vec<&str> {
    let mut declarations = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            ';' if depth == 0 => {
                declarations.push(&value[start..i]);
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }
    declarations.push(&value[start..]);
    declarations
}

/// Generate v-on directive as a prop
fn generate_von_prop(ctx: &mut CodegenContext, dir: &DirectiveNode<'_>) {
    if dir.arg.is_none() && !ctx.merge_props {
        ctx.use_helper(RuntimeHelper::ToHandlers);
        ctx.push("...");
        ctx.push(ctx.helper(RuntimeHelper::ToHandlers));
        ctx.push("(");
        if let Some(exp) = &dir.exp {
            generate_expression(ctx, exp);
        } else {
            ctx.push("undefined");
        }
        ctx.push(", true)");
        return;
    }

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
            let content = exp.content;
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
                exp.content,
                on_plain_element,
                dir.modifiers.iter().map(|m| m.content),
            );

            let needs_quotes = !is_valid_js_identifier(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            // Anchor the generated event-handler key back to the v-on argument
            // in source, recording the original event name so it lands in the
            // v3 `names` array. No-op without `source_map`.
            ctx.record_mapping_named(exp.loc.span.start, exp.content);
            ctx.push(&event_name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
        }
    }

    super::events::generate_von_handler_value(ctx, dir);
}
