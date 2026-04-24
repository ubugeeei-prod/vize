use crate::ir::{
    ChildRefIRNode, GetTextChildIRNode, InsertNodeIRNode, NextRefIRNode, PrependNodeIRNode,
    SetTemplateRefIRNode,
};
use vize_carton::{cstr, String};

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
pub(super) fn generate_insert_node(ctx: &mut GenerateContext, insert: &InsertNodeIRNode) {
    let parent = cstr!("n{}", insert.parent);
    let elements = insert
        .elements
        .iter()
        .map(|e| cstr!("n{e}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");

    if let Some(anchor) = insert.anchor {
        ctx.push_line_fmt(format_args!(
            "_insert({}, [{}], n{})",
            parent, elements, anchor
        ));
    } else {
        ctx.push_line_fmt(format_args!("_insert({}, [{}])", parent, elements));
    }
}

/// Generate PrependNode
pub(super) fn generate_prepend_node(ctx: &mut GenerateContext, prepend: &PrependNodeIRNode) {
    let parent = cstr!("n{}", prepend.parent);
    let elements = prepend
        .elements
        .iter()
        .map(|e| cstr!("n{e}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");

    ctx.push_line_fmt(format_args!("_prepend({}, [{}])", parent, elements));
}

/// Generate GetTextChild
pub(super) fn generate_get_text_child(ctx: &mut GenerateContext, get_text: &GetTextChildIRNode) {
    let parent = cstr!("n{}", get_text.parent);
    let child = ctx.next_temp();

    ctx.push_line_fmt(format_args!("const {} = {}.firstChild", child, parent));
}

/// Generate ChildRef (_child helper)
pub(super) fn generate_child_ref(ctx: &mut GenerateContext, child_ref: &ChildRefIRNode) {
    ctx.use_helper("child");
    if child_ref.offset == 0 {
        ctx.push_line_fmt(format_args!(
            "const n{} = _child(n{})",
            child_ref.child_id, child_ref.parent_id
        ));
    } else {
        ctx.use_helper("next");
        let expr = build_next_chain(cstr!("_child(n{})", child_ref.parent_id), child_ref.offset);
        ctx.push_line_fmt(format_args!("const n{} = {}", child_ref.child_id, expr));
    }
}

/// Generate NextRef (_next helper)
pub(super) fn generate_next_ref(ctx: &mut GenerateContext, next_ref: &NextRefIRNode) {
    ctx.use_helper("next");
    let expr = build_next_chain(cstr!("n{}", next_ref.prev_id), next_ref.offset);
    ctx.push_line_fmt(format_args!("const n{} = {}", next_ref.child_id, expr));
}

fn build_next_chain(base: String, offset: usize) -> String {
    let mut expr = base;
    for _ in 0..offset {
        expr = cstr!("_next({})", expr);
    }
    expr
}
