//! Native element child-list emission.

use vize_s2::op::{Op, Region};

use super::children::{emit_comment_vnode, emit_create_text_vnode, emit_text_like};
use super::{EmitCx, EmitError};

pub(super) fn emit_children(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    force_array: bool,
    hoist_static_children: bool,
    cache_static_children: bool,
) -> Result<(), EmitError> {
    cx.with_static_vnode_hoist_exact(hoist_static_children, |cx| {
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
        && super::hoist::cacheable_elements_array(ops, cx.is_ts)
    {
        return super::hoist::emit_cached_elements_array(cx, ops);
    }
    cx.buf.push("[");
    cx.buf.indent();
    let mut rest: &[Op<'_>] = ops;
    let mut first = true;
    while let Some((op, tail)) = rest.split_first() {
        if is_text_like(op) {
            let (run, after) = split_text_run(rest);
            if !first {
                cx.buf.push(",");
            }
            cx.buf.newline();
            first = false;
            emit_create_text_vnode(cx, run)?;
            rest = after;
            continue;
        }
        if !first {
            cx.buf.push(",");
        }
        cx.buf.newline();
        first = false;
        match op {
            Op::Comment(comment) => {
                let _id = cx.walk.mint();
                emit_comment_vnode(cx, comment);
            }
            op => super::vnode::emit_array_child(
                cx,
                op,
                hoist_static_children,
                cache_static_children,
            )?,
        }
        rest = tail;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

pub(super) fn is_text_like(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(_) | Op::Interpolation(_))
}

/// `ops` split after its leading run of text and interpolation ops.
pub(super) fn split_text_run<'o, 'a>(ops: &'o [Op<'a>]) -> (&'o [Op<'a>], &'o [Op<'a>]) {
    let len = ops.iter().take_while(|op| is_text_like(op)).count();
    ops.split_at_checked(len).unwrap_or((ops, &[]))
}
