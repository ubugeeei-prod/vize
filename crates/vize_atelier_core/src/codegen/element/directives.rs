//! Directive closing generation for elements.
//!
//! Generates the closing portions of `withDirectives()` calls for
//! v-model, v-show, and custom directives on elements.

use crate::{
    ElementNode, ExpressionNode, PropNode, RuntimeHelper,
    steps::v_model::{get_vmodel_helper, parse_model_modifiers},
};

use super::super::{context::CodegenContext, expression::generate_expression};
use super::helpers::{
    get_custom_directives, get_vmodel_directive, has_vmodel_directive, has_vshow_directive,
};
use crate::codegen::helpers::to_valid_asset_identifier;

fn generate_vmodel_entry(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    dir: &crate::DirectiveNode<'_>,
) {
    let helper = get_vmodel_helper(el);
    ctx.use_helper(helper);

    let modifiers: Vec<_> = dir.modifiers.iter().map(|m| m.content.as_str()).collect();
    let parsed_mods = parse_model_modifiers(&dir.modifiers);
    let has_modifiers = parsed_mods.lazy || parsed_mods.number || parsed_mods.trim;

    if has_modifiers {
        let active_modifiers: Vec<_> = modifiers
            .iter()
            .filter(|m| matches!(*m, &"lazy" | &"number" | &"trim"))
            .collect();
        let is_single_modifier = active_modifiers.len() == 1;

        ctx.push("  [");
        ctx.newline();
        ctx.push("    ");
        ctx.push(ctx.helper(helper));
        ctx.push(",");
        ctx.newline();
        ctx.push("    ");
        if let Some(exp) = &dir.exp {
            generate_expression(ctx, exp);
        }
        ctx.push(",");
        ctx.newline();
        ctx.push("    void 0,");
        ctx.newline();

        if is_single_modifier {
            ctx.push("    { ");
            ctx.push(active_modifiers[0]);
            ctx.push(": true }");
        } else {
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
        ctx.push("  [");
        ctx.push(ctx.helper(helper));
        ctx.push(", ");
        if let Some(exp) = &dir.exp {
            generate_expression(ctx, exp);
        }
        ctx.push("]");
    }
}

fn generate_vshow_entry(ctx: &mut CodegenContext, dir: &crate::DirectiveNode<'_>) -> bool {
    let Some(exp) = &dir.exp else {
        return false;
    };

    ctx.use_helper(RuntimeHelper::VShow);
    ctx.push("  [");
    ctx.push(ctx.helper(RuntimeHelper::VShow));
    ctx.push(", ");
    generate_expression(ctx, exp);
    ctx.push("]");
    true
}

fn generate_custom_directive_entry(ctx: &mut CodegenContext, dir: &crate::DirectiveNode<'_>) {
    ctx.push("  [");
    ctx.push(&to_valid_asset_identifier("directive", &dir.name));

    if let Some(exp) = &dir.exp {
        ctx.push(", ");
        generate_expression(ctx, exp);
    }

    if let Some(arg) = &dir.arg {
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

    if !dir.modifiers.is_empty() {
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

/// Generate v-model directive closing
pub fn generate_vmodel_closing(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let Some(dir) = get_vmodel_directive(el) else {
        return;
    };

    ctx.push(", [");
    ctx.newline();
    generate_vmodel_entry(ctx, el, dir);

    for prop in &el.props {
        if let PropNode::Directive(show_dir) = prop
            && show_dir.name.as_str() == "show"
            && show_dir.exp.is_some()
        {
            ctx.push(",");
            ctx.newline();
            generate_vshow_entry(ctx, show_dir);
            break;
        }
    }

    ctx.newline();
    ctx.push("])");
}

/// Generate v-show directive closing if present
pub fn generate_vshow_closing(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name.as_str() == "show"
            && dir.exp.is_some()
        {
            ctx.push(", [");
            ctx.newline();
            generate_vshow_entry(ctx, dir);
            ctx.newline();
            ctx.push("])");
            return;
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

    let has_native_vmodel = has_vmodel_directive(el);
    let mut emitted = false;

    if has_native_vmodel && let Some(dir) = get_vmodel_directive(el) {
        generate_vmodel_entry(ctx, el, dir);
        emitted = true;
    }

    for dir in custom_dirs {
        if emitted {
            ctx.push(",");
            ctx.newline();
        }
        generate_custom_directive_entry(ctx, dir);
        emitted = true;
    }

    if has_vshow_directive(el) {
        for prop in &el.props {
            if let PropNode::Directive(dir) = prop
                && dir.name.as_str() == "show"
                && dir.exp.is_some()
            {
                if emitted {
                    ctx.push(",");
                    ctx.newline();
                }
                generate_vshow_entry(ctx, dir);
                break;
            }
        }
    }

    ctx.newline();
    ctx.push("])");
}
