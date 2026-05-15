//! Directive closing generation for elements.
//!
//! Generates the closing portions of `withDirectives()` calls for
//! v-model, v-show, and custom directives on elements.

use crate::{
    ast::{ElementNode, ExpressionNode, PropNode, RuntimeHelper},
    transforms::v_model::{get_vmodel_helper, parse_model_modifiers},
};

use super::super::{context::CodegenContext, expression::generate_expression};
use super::helpers::{get_custom_directives, get_vmodel_directive, has_vshow_directive};
use crate::codegen::helpers::to_valid_asset_identifier;

/// Generate v-model directive closing
pub fn generate_vmodel_closing(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    if let Some(dir) = get_vmodel_directive(el) {
        let helper = get_vmodel_helper(el);
        ctx.use_helper(helper);

        ctx.push(", [");
        ctx.newline();

        // Check for modifiers
        let modifiers: Vec<_> = dir.modifiers.iter().map(|m| m.content.as_str()).collect();
        let parsed_mods = parse_model_modifiers(&dir.modifiers);
        let has_modifiers = parsed_mods.lazy || parsed_mods.number || parsed_mods.trim;

        if has_modifiers {
            // Count active modifiers
            let active_modifiers: Vec<_> = modifiers
                .iter()
                .filter(|m| matches!(*m, &"lazy" | &"number" | &"trim"))
                .collect();
            let is_single_modifier = active_modifiers.len() == 1;

            // Multi-line format with modifiers
            ctx.push("  [");
            ctx.newline();
            ctx.push("    ");
            ctx.push(ctx.helper(helper));
            ctx.push(",");
            ctx.newline();
            ctx.push("    ");
            // Value expression
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            }
            ctx.push(",");
            ctx.newline();
            ctx.push("    void 0,");
            ctx.newline();

            if is_single_modifier {
                // Single modifier: inline format { lazy: true }
                ctx.push("    { ");
                ctx.push(active_modifiers[0]);
                ctx.push(": true }");
            } else {
                // Multiple modifiers: multiline format
                ctx.push("    {");
                for (i, modifier) in active_modifiers.iter().enumerate() {
                    ctx.newline();
                    ctx.push("      ");
                    ctx.push(modifier);
                    ctx.push(": true");
                    if i < active_modifiers.len() - 1 {
                        ctx.push(",");
                    }
                }
                ctx.newline();
                ctx.push("    }");
            }
            ctx.newline();
            ctx.push("  ]");
        } else {
            // Simple format without modifiers
            ctx.push("  [");
            ctx.push(ctx.helper(helper));
            ctx.push(", ");
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            }
            ctx.push("]");
        }

        ctx.newline();
        ctx.push("])");
    }
}

/// Generate v-show directive closing if present
pub fn generate_vshow_closing(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name.as_str() == "show" {
                if let Some(exp) = &dir.exp {
                    ctx.push(", [");
                    ctx.newline();
                    ctx.push("  [");
                    ctx.push(ctx.helper(RuntimeHelper::VShow));
                    ctx.push(", ");
                    generate_expression(ctx, exp);
                    ctx.push("]");
                    ctx.newline();
                    ctx.push("])");
                }
                return;
            }
        }
    }
}

/// Generate custom directives closing
pub fn generate_custom_directives_closing(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let custom_dirs = get_custom_directives(el);
    if custom_dirs.is_empty() {
        return;
    }

    ctx.push(", [");
    ctx.newline();

    for (i, dir) in custom_dirs.iter().enumerate() {
        if i > 0 {
            ctx.push(",");
            ctx.newline();
        }
        ctx.push("  [");
        ctx.push(&to_valid_asset_identifier("directive", &dir.name));

        // Add value if present
        if let Some(exp) = &dir.exp {
            ctx.push(", ");
            generate_expression(ctx, exp);
        }

        // Add argument if present
        if let Some(arg) = &dir.arg {
            // Need to add value placeholder if not present
            if dir.exp.is_none() {
                ctx.push(", void 0");
            }
            ctx.push(", ");
            match arg {
                ExpressionNode::Simple(simple) => {
                    if simple.is_static {
                        ctx.push("\"");
                        ctx.push(&simple.content);
                        ctx.push("\"");
                    } else {
                        ctx.push(&simple.content);
                    }
                }
                ExpressionNode::Compound(compound) => {
                    ctx.push(&compound.loc.source);
                }
            }
        }

        // Add modifiers if present
        if !dir.modifiers.is_empty() {
            // Need to add placeholders if not present
            if dir.exp.is_none() && dir.arg.is_none() {
                ctx.push(", void 0, void 0");
            } else if dir.arg.is_none() {
                ctx.push(", void 0");
            }
            ctx.push(", { ");
            for (j, modifier) in dir.modifiers.iter().enumerate() {
                if j > 0 {
                    ctx.push(", ");
                }
                ctx.push(&modifier.content);
                ctx.push(": true");
            }
            ctx.push(" }");
        }

        ctx.push("]");
    }

    // Also include v-show in the same withDirectives array if present
    if has_vshow_directive(el) {
        for prop in &el.props {
            if let PropNode::Directive(dir) = prop {
                if dir.name.as_str() == "show" {
                    if let Some(exp) = &dir.exp {
                        ctx.push(",");
                        ctx.newline();
                        ctx.push("  [");
                        ctx.use_helper(RuntimeHelper::VShow);
                        ctx.push(ctx.helper(RuntimeHelper::VShow));
                        ctx.push(", ");
                        generate_expression(ctx, exp);
                        ctx.push("]");
                    }
                    break;
                }
            }
        }
    }

    ctx.newline();
    ctx.push("])");
}
