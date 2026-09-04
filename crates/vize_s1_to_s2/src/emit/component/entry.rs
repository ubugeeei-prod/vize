//! The component emit entry points: which vnode shape a component takes
//! (block, plain vnode, memo/once cache) and, with it, the shipped
//! `hoist_static_inner` position its props are decided at. Split out of
//! `component.rs` under the 350-line source budget.

use vize_davinci::id::NodeId;
use vize_s2::op::ComponentOp;

use super::super::EmitCx;
use super::super::EmitError;
use super::super::buf::Buf;
use super::super::builtin;
use super::super::directive;
use super::super::props_static::PropHoistPosition as Position;
use super::checks::has_dynamic_key_binding;
use super::emit_call;

/// The template root's single component: the shipped
/// `hoist_static(children, is_root = true)` position.
pub(in crate::emit) fn emit_root(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_top(cx, component, id, Position::Root)
}

fn emit_top(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    position: Position,
) -> Result<(), EmitError> {
    if super::super::once::has(&component.bindings) {
        return super::super::once::emit_component(cx, component, None, false, id);
    }
    if super::super::memo::has(&component.bindings) && !cx.skip_memo {
        return super::super::memo::emit_cached(cx, &component.bindings, |cx| {
            emit_vnode(cx, component, None, false, id, position)
        });
    }
    emit_block(cx, component, None, false, id, position)
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
    position: Position,
) -> Result<(), EmitError> {
    directive::wrap_component(cx, component, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(
            cx, component, /* block */ true, if_key, for_item, id, position,
        )?;
        cx.buf.push(")");
        Ok(())
    })
}

fn emit_vnode(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
    position: Position,
) -> Result<(), EmitError> {
    directive::wrap_component(cx, component, |cx| {
        cx.buf.use_create_vnode();
        emit_call(cx, component, false, if_key, for_item, id, position)
    })
}

pub(in crate::emit) fn emit_nested(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_at(cx, component, id, Position::Nested)
}

/// A component child of the root fragment. The shipped lane hoists the
/// root children as one list, so every one of them is visited at
/// `is_root = true` — the multi-root sibling of [`emit_root`].
pub(in crate::emit) fn emit_fragment_root(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_at(cx, component, id, Position::Root)
}

fn emit_at(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    position: Position,
) -> Result<(), EmitError> {
    if super::super::once::has(&component.bindings) {
        return super::super::once::emit_component(cx, component, None, false, id);
    }
    if super::super::memo::has(&component.bindings) && !cx.skip_memo {
        return super::super::memo::emit_cached(cx, &component.bindings, |cx| {
            emit_vnode(cx, component, None, false, id, position)
        });
    }
    if builtin::forces_block(component) {
        return emit_top(cx, component, id, position);
    }
    if has_dynamic_key_binding(component) {
        return emit_block(cx, component, None, false, id, position);
    }
    emit_vnode(cx, component, None, false, id, position)
}

pub(in crate::emit) fn emit_if_branch(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    key: &str,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_block(cx, component, Some(key), false, id, Position::Nested)
}

pub(in crate::emit) fn emit_for_item(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    key: Option<&str>,
) -> Result<(), EmitError> {
    if let Some(memo) = super::super::memo::first(&component.bindings)
        && !cx.skip_memo
    {
        return super::super::memo::emit_cached_with_key(cx, memo, key.unwrap_or("0"), |cx| {
            emit_block(cx, component, key, true, id, Position::Nested)
        });
    }
    emit_block(cx, component, key, true, id, Position::Nested)
}
