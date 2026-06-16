//! `v-bind` (`:`) directive prop emission.
//!
//! Holds `generate_vbind_prop` — the largest directive emitter, covering
//! class/style merging, dynamic keys, and binding-type runtime resolution.
//! Split out of `directives` to keep that file focused on directive dispatch.

use crate::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext,
    expression::{generate_expression, generate_simple_expression},
    helpers::{camelize, escape_js_string, is_valid_js_identifier},
};
use super::directives::{StaticBindKeyCasing, StaticMerge, is_static_expression};
use vize_carton::String;
use vize_carton::ToCompactString;

/// Generate v-bind directive as a prop
pub(super) fn generate_vbind_prop(
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
            // Anchor the generated prop key back to the v-bind argument in
            // source, recording the original (untransformed) symbol so it lands
            // in the v3 `names` array. No-op without `source_map`.
            ctx.record_mapping_named(&exp.loc.start, &exp.content);
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
