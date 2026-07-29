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

/// Generate ChildRef (_child helper)
pub(super) fn generate_child_ref(ctx: &mut GenerateContext, child_ref: &ChildRefIRNode) {
    // `offset` is the absolute rendered index within the parent. Record it so a
    // `NextRef` hopping off this node can compute its own index (#3330).
    ctx.record_node_position(child_ref.child_id, child_ref.parent_id, child_ref.offset);
    ctx.use_helper("child");
    if child_ref.offset == 0 {
        ctx.push_line_fmt(format_args!(
            "const n{} = _child(n{})",
            child_ref.child_id, child_ref.parent_id
        ));
    } else if child_ref.offset == 1 {
        let expr = build_next_chain(cstr!("_child(n{})", child_ref.parent_id), 1, 1, ctx);
        ctx.push_line_fmt(format_args!("const n{} = {}", child_ref.child_id, expr));
    } else {
        // Outside hydration the runtime `next()` advances a single sibling, so
        // absolute child positions beyond one step must go through `nthChild`
        // (mirrors vue's compiler-vapor).
        ctx.use_helper("nthChild");
        ctx.push_line_fmt(format_args!(
            "const n{} = _nthChild(n{}, {})",
            child_ref.child_id, child_ref.parent_id, child_ref.offset
        ));
    }
}

/// Generate NextRef (`_next` / `_nthChild` helper).
///
/// A jump of two or more siblings becomes an absolute `_nthChild` lookup for
/// the same reason `generate_child_ref` uses one: chaining bare `_next(node)`
/// calls works outside hydration but returns `null` while hydrating, because
/// each call reaches `locateChildByLogicalIndex(parent, undefined)` and no
/// index ever equals `undefined`. A single-step jump stays a `_next`, carrying
/// this node's **absolute** index so hydration resolves the same node — a
/// literal `1` is only correct when the target happens to be the parent's
/// second child (#3330).
///
/// The node only names its predecessor, so the parent and this node's absolute
/// index come from the position the predecessor recorded: every sibling chain
/// is anchored by a `ChildRef`, which knows both.
pub(super) fn generate_next_ref(ctx: &mut GenerateContext, next_ref: &NextRefIRNode) {
    let position = ctx
        .node_position(next_ref.prev_id)
        .map(|(parent_id, prev_index)| (parent_id, prev_index + next_ref.offset));

    let expr = match position {
        Some((parent_id, index)) if next_ref.offset > 1 => {
            ctx.use_helper("nthChild");
            cstr!("_nthChild(n{}, {})", parent_id, index)
        }
        Some((_, index)) => {
            build_next_chain(cstr!("n{}", next_ref.prev_id), next_ref.offset, index, ctx)
        }
        // Without an anchored predecessor there is no parent handle to look the
        // node up on, so the hop stays relative and the index falls back to the
        // offset. Sibling chains always start at a `ChildRef`, so this is only a
        // safety net.
        None => build_next_chain(
            cstr!("n{}", next_ref.prev_id),
            next_ref.offset,
            next_ref.offset,
            ctx,
        ),
    };

    if let Some((parent_id, index)) = position {
        ctx.record_node_position(next_ref.child_id, parent_id, index);
    }
    ctx.push_line_fmt(format_args!("const n{} = {}", next_ref.child_id, expr));
}

/// Build a navigation expression for a jump of at most one sibling, passing
/// `index` as the hydration hint. Multi-step jumps never reach here.
fn build_next_chain(
    base: String,
    offset: usize,
    index: usize,
    ctx: &mut GenerateContext,
) -> String {
    if offset == 0 {
        base
    } else {
        ctx.use_helper("next");
        cstr!("_next({}, {})", base, index)
    }
}
