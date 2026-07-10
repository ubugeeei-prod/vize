//! Directive closing generation for elements.
//!
//! Generates the closing portions of `withDirectives()` calls for
//! v-model, v-show, and custom directives on elements.

use crate::{
    ElementNode, ExpressionNode, PropNode, RuntimeHelper, rendu::RenduOp,
    steps::v_model::get_vmodel_helper,
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
    let RenduOp::Directive { exp, modifiers, .. } = RenduOp::from_directive(dir) else {
        unreachable!("v-model emission requires RenduOp::Directive");
    };
    let helper = get_vmodel_helper(el);
    ctx.use_helper(helper);

    let modifiers: Vec<_> = modifiers.names().collect();
    let has_modifiers = modifiers
        .iter()
        .any(|modifier| matches!(*modifier, "lazy" | "number" | "trim"));

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
        if let Some(exp) = exp.and_then(|exp| exp.node()) {
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
        if let Some(exp) = exp.and_then(|exp| exp.node()) {
            generate_expression(ctx, exp);
        }
        ctx.push("]");
    }
}

fn generate_vshow_entry(ctx: &mut CodegenContext, dir: &crate::DirectiveNode<'_>) -> bool {
    let RenduOp::Directive { exp, .. } = RenduOp::from_directive(dir) else {
        unreachable!("v-show emission requires RenduOp::Directive");
    };
    let Some(exp) = exp.and_then(|exp| exp.node()) else {
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
    let RenduOp::Directive {
        name,
        arg,
        exp,
        modifiers,
        ..
    } = RenduOp::from_directive(dir)
    else {
        unreachable!("custom directive emission requires RenduOp::Directive");
    };
    ctx.push("  [");
    ctx.push(&to_valid_asset_identifier("directive", name));

    if let Some(exp) = exp.and_then(|exp| exp.node()) {
        ctx.push(", ");
        generate_expression(ctx, exp);
    }

    if let Some(arg) = arg {
        if exp.is_none() {
            ctx.push(", void 0");
        }
        ctx.push(", ");
        match arg.node() {
            Some(ExpressionNode::Simple(simple)) => {
                if simple.is_static {
                    ctx.push("\"");
                    ctx.push(&simple.content);
                    ctx.push("\"");
                } else {
                    ctx.push(&simple.content);
                }
            }
            Some(ExpressionNode::Compound(compound)) => {
                ctx.push(&compound.loc.source);
            }
            None => ctx.push(arg.text()),
        }
    }

    if !modifiers.is_empty() {
        if exp.is_none() && arg.is_none() {
            ctx.push(", void 0, void 0");
        } else if arg.is_none() {
            ctx.push(", void 0");
        }
        ctx.push(", { ");
        for (j, modifier) in modifiers.names().enumerate() {
            if j > 0 {
                ctx.push(", ");
            }
            ctx.push(modifier);
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
        if matches!(
            RenduOp::from_prop(prop),
            RenduOp::Directive {
                name: "show",
                exp: Some(_),
                ..
            }
        ) && let PropNode::Directive(show_dir) = prop
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
        if matches!(
            RenduOp::from_prop(prop),
            RenduOp::Directive {
                name: "show",
                exp: Some(_),
                ..
            }
        ) && let PropNode::Directive(dir) = prop
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
            if matches!(
                RenduOp::from_prop(prop),
                RenduOp::Directive {
                    name: "show",
                    exp: Some(_),
                    ..
                }
            ) && let PropNode::Directive(dir) = prop
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
