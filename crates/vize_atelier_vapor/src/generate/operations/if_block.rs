use crate::ir::{IfIRNode, NegativeBranch};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, ToCompactString};

use super::{
    super::{context::GenerateContext, generate_block},
    block_shape::{IfFlags, if_flags},
    insertion::emit_insertion_state,
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
        EmitDocument::plain(&["const n", &if_node.id.to_compact_string(), " = "].concat());
    head.push_spanned(&ctx.if_head(if_node));
    ctx.push_line_spanned(&head);

    let slot_root = ctx.transition_slot
        && if_node.parent.is_none()
        && if_node.negative.is_none()
        && matches!(if_node.positive.returns.as_slice(), [id] if element_template_map.contains_key(id));
    let force_multi_root = ctx.loop_row_ifs.contains(&if_node.id);
    let flags = if_flags(
        ctx,
        if_node,
        element_template_map,
        force_multi_root,
        slot_root,
    );
    let previous_transition = ctx.transition_slot;
    ctx.transition_slot = false;
    let was_fragment = ctx.is_fragment;
    ctx.is_fragment = true;
    ctx.indent();
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
                ctx.push_component_scope();
                generate_block(ctx, block, element_template_map);
                ctx.pop_component_scope();
                ctx.deindent();
                ctx.push_indent();
                push_close(ctx, "})", flags);
                ctx.push("\n");
            }
            NegativeBranch::If(nested_if) => {
                // Nested `v-else-if` chains are branch roots of the outer `createIf`
                // fragment: they never carry their own insertion state.
                ctx.push_indent();
                ctx.push("}, () => ");
                generate_nested_if(ctx, nested_if, element_template_map);
                push_close(ctx, ")", flags);
                ctx.push("\n");
            }
        }
    } else {
        ctx.push_indent();
        push_close(ctx, "}, null)", flags);
        ctx.push("\n");
    }
    ctx.transition_slot = previous_transition;
    ctx.is_fragment = was_fragment;
}

/// Generate nested if (for v-else-if chains - starts inline without leading indent)
fn generate_nested_if(
    ctx: &mut GenerateContext,
    if_node: &IfIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createIf");
    let flags = if_flags(ctx, if_node, element_template_map, false, false);

    // Start inline - no leading indent or newline
    let head = ctx.if_head(if_node);
    ctx.push_spanned(&head);
    ctx.push("\n");

    ctx.indent();
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
                ctx.push_component_scope();
                generate_block(ctx, block, element_template_map);
                ctx.pop_component_scope();
                ctx.deindent();
                ctx.push_indent();
                push_close(ctx, "})", flags);
            }
            NegativeBranch::If(nested_if) => {
                // Nested `v-else-if` chains are branch roots of the outer `createIf`
                // fragment: they never carry their own insertion state.
                ctx.push_indent();
                ctx.push("}, () => ");
                generate_nested_if(ctx, nested_if, element_template_map);
                push_close(ctx, ")", flags);
            }
        }
    } else {
        ctx.push_indent();
        push_close(ctx, "}, null)", flags);
    }
}

/// Ends a `createIf` call with `close`, writing any flags before its final
/// `)`. A call without flags and without a false branch closes as `})`.
fn push_close(ctx: &mut GenerateContext<'_>, close: &str, flags: Option<IfFlags>) {
    match flags {
        Some(flags) => {
            ctx.push(close.strip_suffix(')').unwrap_or(close));
            ctx.push(", ");
            flags.write(ctx);
            ctx.push(")");
        }
        None if close == "}, null)" => ctx.push("})"),
        None => ctx.push(close),
    }
}
