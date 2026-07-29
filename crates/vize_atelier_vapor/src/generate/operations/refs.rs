use crate::ir::{
    ChildRefIRNode, GetTextChildIRNode, InsertNodeIRNode, NextRefIRNode, PrependNodeIRNode,
    SetTemplateRefIRNode,
};
use vize_carton::{String, cstr};

use super::super::context::GenerateContext;

/// Generate SetTemplateRef
pub(super) fn generate_set_template_ref(
    ctx: &mut GenerateContext,
    set_ref: &SetTemplateRefIRNode<'_>,
) {
    let element = cstr!("n{}", set_ref.element);

    let value = if set_ref.value.is_static {
        cstr!("\"{}\"", set_ref.value.content)
    } else {
        ctx.resolve_expression(set_ref.value.content.as_str())
    };

    if set_ref.ref_for {
        ctx.push_line_fmt(format_args!(
            "_setRef({}, {}, undefined, true)",
            element, value
        ));
    } else {
        ctx.push_line_fmt(format_args!("_setRef({}, {})", element, value));
    }
}

/// Generate InsertNode
///
/// The Vapor runtime signature is `insert(block, parent, anchor?)`; multiple
/// blocks are passed as one array block, a single block stays bare.
pub(super) fn generate_insert_node(ctx: &mut GenerateContext, insert: &InsertNodeIRNode) {
    ctx.use_helper("insert");
    let parent = cstr!("n{}", insert.parent);
    let elements = insert
        .elements
        .iter()
        .map(|e| cstr!("n{e}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");
    let block = if insert.elements.len() > 1 {
        cstr!("[{}]", elements)
    } else {
        cstr!("{}", elements)
    };

    if let Some(anchor) = insert.anchor {
        ctx.push_line_fmt(format_args!("_insert({}, {}, n{})", block, parent, anchor));
    } else {
        ctx.push_line_fmt(format_args!("_insert({}, {})", block, parent));
    }
}

/// Generate PrependNode
///
/// The Vapor runtime signature is `prepend(parent, ...blocks)`.
pub(super) fn generate_prepend_node(ctx: &mut GenerateContext, prepend: &PrependNodeIRNode) {
    ctx.use_helper("prepend");
    let parent = cstr!("n{}", prepend.parent);
    let elements = prepend
        .elements
        .iter()
        .map(|e| cstr!("n{e}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");

    ctx.push_line_fmt(format_args!("_prepend({}, {})", parent, elements));
}

/// Generate GetTextChild
pub(super) fn generate_get_text_child(ctx: &mut GenerateContext, get_text: &GetTextChildIRNode) {
    let parent = cstr!("n{}", get_text.parent);
    let child = ctx.next_temp();

    ctx.push_line_fmt(format_args!("const {} = {}.firstChild", child, parent));
}

/// Generate ChildRef (`_child` / `_next` / `_nthChild` helper).
///
/// Mirrors the Vue Vapor runtime's navigation contract (see
/// [`build_navigation`]): index 0 is `_child`, index 1 is one `_next` step,
/// and anything further is an absolute `_nthChild` lookup from the parent.
pub(super) fn generate_child_ref(ctx: &mut GenerateContext, child_ref: &ChildRefIRNode) {
    let expr = if child_ref.offset > 1 {
        ctx.use_helper("nthChild");
        cstr!("_nthChild(n{}, {})", child_ref.parent_id, child_ref.offset)
    } else {
        ctx.use_helper("child");
        build_navigation(
            cstr!("_child(n{})", child_ref.parent_id),
            child_ref.offset,
            child_ref.offset,
            ctx,
        )
    };
    ctx.push_line_fmt(format_args!("const n{} = {}", child_ref.child_id, expr));
}

/// Generate NextRef (`_next` helper).
///
/// The node is always the sibling immediately after `prev_id` — jumps of two
/// or more siblings arrive as a [`ChildRefIRNode`] absolute lookup instead
/// (#3330) — so this is a single `_next` step carrying the absolute index as
/// the hydration hint.
pub(super) fn generate_next_ref(ctx: &mut GenerateContext, next_ref: &NextRefIRNode) {
    let expr = build_navigation(cstr!("n{}", next_ref.prev_id), 1, next_ref.offset, ctx);
    ctx.push_line_fmt(format_args!("const n{} = {}", next_ref.child_id, expr));
}

/// Build a navigation expression for a jump of at most one sibling.
///
/// The runtime's `next(node, i)` advances **exactly one** sibling outside
/// hydration — `i` is an absolute logical index used only while hydrating, not
/// a step count. Emitting `_next(node, 3)` for a three-sibling jump therefore
/// landed one sibling over and a chained `_child()` dereferenced `null`
/// (#3330). Multi-step jumps go through `_nthChild` instead; this helper only
/// covers `steps <= 1`, and passes `index` so hydration resolves the same node.
fn build_navigation(base: String, steps: usize, index: usize, ctx: &mut GenerateContext) -> String {
    if steps == 0 {
        base
    } else {
        ctx.use_helper("next");
        cstr!("_next({}, {})", base, index)
    }
}
