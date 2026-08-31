//! Native element child-list emission.

use vize_s2::op::{Op, Region};

use super::children::{emit_create_text_vnode, emit_text_like};
use super::{EmitCx, EmitError};

pub(super) fn emit_children(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    force_array: bool,
    hoist_static_children: bool,
    cache_static_children: bool,
) -> Result<(), EmitError> {
    let hoist_static_children = hoist_static_children || cx.hoist_static_vnodes;
    cx.with_static_vnode_hoist(hoist_static_children, |cx| {
        emit_children_inner(
            cx,
            children,
            force_array,
            hoist_static_children,
            cache_static_children,
        )
    })
}

fn emit_children_inner(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    force_array: bool,
    hoist_static_children: bool,
    cache_static_children: bool,
) -> Result<(), EmitError> {
    let ops = &children.ops;
    if !force_array
        && ops
            .iter()
            .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        return emit_text_like(cx, ops);
    }
    if !force_array
        && !hoist_static_children
        && cache_static_children
        && super::hoist::cacheable_elements_array(ops)
    {
        return super::hoist::emit_cached_elements_array(cx, ops);
    }
    cx.buf.push("[");
    cx.buf.indent();
    let mut i = 0;
    let mut first = true;
    while i < ops.len() {
        if matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
            let start = i;
            while i < ops.len() && matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
                i += 1;
            }
            if !first {
                cx.buf.push(",");
            }
            cx.buf.newline();
            first = false;
            emit_create_text_vnode(cx, &ops[start..i])?;
            continue;
        }
        if !first {
            cx.buf.push(",");
        }
        cx.buf.newline();
        first = false;
        super::vnode::emit_array_child(cx, &ops[i], hoist_static_children, cache_static_children)?;
        i += 1;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}
