use crate::ir::{IfIRNode, NegativeBranch};
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

    let condition = if if_node.condition.is_static {
        ["\"", if_node.condition.content.as_str(), "\""].concat()
    } else {
        let resolved = ctx.resolve_expression(&if_node.condition.content);
        ["(", &resolved, ")"].concat()
    };

    ctx.push_line(
        &[
            "const n",
            &if_node.id.to_compact_string(),
            " = _createIf(() => ",
            &condition,
            ", () => {",
        ]
        .concat(),
    );

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
                ctx.push_line("}, () => {");
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

    let condition = if if_node.condition.is_static {
        ["\"", if_node.condition.content.as_str(), "\""].concat()
    } else {
        let resolved = ctx.resolve_expression(&if_node.condition.content);
        ["(", &resolved, ")"].concat()
    };

    // Start inline - no leading indent or newline
    ctx.push(&["_createIf(() => ", &condition, ", () => {\n"].concat());

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
                ctx.push_line("}, () => {");
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
    } else {
        ctx.push_indent();
        ctx.push("})");
    }
}
