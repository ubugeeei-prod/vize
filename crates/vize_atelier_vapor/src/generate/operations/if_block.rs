use crate::ir::{IfIRNode, NegativeBranch};
use vize_atelier_core::codegen::spanned::SpannedText;
use vize_carton::{FxHashMap, ToCompactString};

use super::{
    super::{context::GenerateContext, generate_block},
    insertion::{block_requires_parent_insertion_state, emit_insertion_state},
};

/// Generate If
pub(super) fn generate_if(
    ctx: &mut GenerateContext,
    if_node: &IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    generate_if_inner(ctx, if_node, element_template_map);
}

/// Generate If (inner - for top-level if nodes)
fn generate_if_inner(
    ctx: &mut GenerateContext,
    if_node: &IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createIf");
    emit_insertion_state(ctx, if_node.parent, if_node.anchor);

    let mut head =
        SpannedText::plain(&["const n", &if_node.id.to_compact_string(), " = "].concat());
    head.push_spanned(&ctx.if_head(if_node));
    ctx.push_line_spanned(&head);

    let was_fragment = ctx.is_fragment;
    ctx.is_fragment = true;
    ctx.indent();
    if block_requires_parent_insertion_state(&if_node.positive) {
        emit_insertion_state(ctx, if_node.parent, if_node.anchor);
    }
    ctx.push_component_scope();
    generate_block(ctx, &if_node.positive, element_template_map);
    ctx.pop_component_scope();
    ctx.deindent();

    if let Some(ref negative) = if_node.negative {
        match negative {
            NegativeBranch::Block(block) => {
                let else_head = ctx.else_head(if_node);
                ctx.push_line_spanned(&else_head);
                ctx.indent();
                if block_requires_parent_insertion_state(block) {
                    emit_insertion_state(ctx, if_node.parent, if_node.anchor);
                }
                ctx.push_component_scope();
                generate_block(ctx, block, element_template_map);
                ctx.pop_component_scope();
                ctx.deindent();
                ctx.push_line("})");
            }
            NegativeBranch::If(nested_if) => {
                if nested_if.parent.is_none() && nested_if.anchor.is_none() {
                    ctx.push("}, () => ");
                    generate_nested_if(ctx, nested_if, element_template_map);
                    ctx.push(")");
                    ctx.push("\n");
                } else {
                    ctx.push_line("}, () => {");
                    ctx.indent();
                    emit_insertion_state(ctx, nested_if.parent, nested_if.anchor);
                    ctx.push_indent();
                    ctx.push("return ");
                    generate_nested_if(ctx, nested_if, element_template_map);
                    ctx.push("\n");
                    ctx.deindent();
                    ctx.push_line("})");
                }
            }
        }
    } else {
        ctx.push_line("})");
    }
    ctx.is_fragment = was_fragment;
}

/// Generate nested if (for v-else-if chains - starts inline without leading indent)
fn generate_nested_if(
    ctx: &mut GenerateContext,
    if_node: &IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createIf");

    // Start inline - no leading indent or newline
    let head = ctx.if_head(if_node);
    ctx.push_spanned(&head);
    ctx.push("\n");

    ctx.indent();
    if block_requires_parent_insertion_state(&if_node.positive) {
        emit_insertion_state(ctx, if_node.parent, if_node.anchor);
    }
    ctx.push_component_scope();
    generate_block(ctx, &if_node.positive, element_template_map);
    ctx.pop_component_scope();
    ctx.deindent();

    if let Some(ref negative) = if_node.negative {
        match negative {
            NegativeBranch::Block(block) => {
                let else_head = ctx.else_head(if_node);
                ctx.push_line_spanned(&else_head);
                ctx.indent();
                if block_requires_parent_insertion_state(block) {
                    emit_insertion_state(ctx, if_node.parent, if_node.anchor);
                }
                ctx.push_component_scope();
                generate_block(ctx, block, element_template_map);
                ctx.pop_component_scope();
                ctx.deindent();
                ctx.push_indent();
                ctx.push("})");
            }
            NegativeBranch::If(nested_if) => {
                if nested_if.parent.is_none() && nested_if.anchor.is_none() {
                    ctx.push("}, () => ");
                    generate_nested_if(ctx, nested_if, element_template_map);
                    ctx.push(")");
                } else {
                    ctx.push_line("}, () => {");
                    ctx.indent();
                    emit_insertion_state(ctx, nested_if.parent, nested_if.anchor);
                    ctx.push_indent();
                    ctx.push("return ");
                    generate_nested_if(ctx, nested_if, element_template_map);
                    ctx.push("\n");
                    ctx.deindent();
                    ctx.push_indent();
                    ctx.push("})");
                }
            }
        }
    } else {
        ctx.push_indent();
        ctx.push("})");
    }
}
