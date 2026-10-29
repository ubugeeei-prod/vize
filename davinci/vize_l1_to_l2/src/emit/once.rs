//! `vue.once` realization.

use vize_l0::id::NodeId;
use vize_l2::op::{BindingOp, ComponentOp, ElementOp, IfOp, Op};

use super::buf::Buf;
use super::hoist::is_hoistable;
use super::vnode;
use super::{EmitCx, EmitError};

pub(super) fn has(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(is_once)
}

pub(super) fn is_once(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::VueOnce(_))
}

/// Whether `v-once` sits on the chain's own `v-if` carrier, an element or an
/// unwrapped `<template>`. The whole chain is cached then, so a later change
/// of the condition is ignored as well. Content inside a `<template v-if>` is
/// not the chain's carrier: its condition stays live.
pub(super) fn chain_is_once(cx: &EmitCx<'_>, if_op: &IfOp<'_>, id: Option<NodeId>) -> bool {
    let wrapper = id.and_then(|id| cx.wrappers.get(id));
    if wrapper.is_some_and(|keys| keys.from_template.first() == Some(&true)) {
        return wrapper.is_some_and(|keys| keys.once);
    }
    match if_op
        .branches
        .first()
        .map(|branch| branch.region.ops.as_slice())
    {
        Some([Op::Element(element)]) => has(&element.bindings),
        Some([Op::Component(component)]) => has(&component.bindings),
        _ => false,
    }
}

pub(super) fn emit_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: Option<&str>,
    for_item: bool,
) -> Result<(), EmitError> {
    emit_cached(cx, |cx| {
        // The same vnode as without `v-once`, minus handler caching.
        let cache_handlers = core::mem::replace(&mut cx.cache_handlers, false);
        cx.once_depth += 1;
        let result = super::directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            vnode::emit_call(
                cx,
                element,
                /* block */ false,
                key,
                /* hoist */ (false, None, super::props_static::PropHoistPosition::Nested),
                for_item,
                /* once */ true,
            )
        });
        cx.once_depth -= 1;
        cx.cache_handlers = cache_handlers;
        result
    })
}

pub(super) fn emit_component(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    _key: Option<&str>,
    _for_item: bool,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_cached(cx, |cx| {
        let cache_handlers = core::mem::replace(&mut cx.cache_handlers, false);
        let result = super::component::emit_once(cx, component, id);
        cx.cache_handlers = cache_handlers;
        result
    })
}

pub(super) fn emit_cached(
    cx: &mut EmitCx<'_>,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let cache_slot = cx.once_cache_index;
    cx.once_cache_index += 1;
    cx.buf.use_set_block_tracking();
    cx.buf.push("_cache[");
    cx.push_cache_index(cache_slot);
    cx.buf.push("] || (");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push(Buf::set_block_tracking_alias());
    cx.buf.push("(-1, true),");
    cx.buf.newline();
    cx.buf.push("(_cache[");
    cx.push_cache_index(cache_slot);
    cx.buf.push("] = ");
    emit(cx)?;
    cx.buf.push(").cacheIndex = ");
    cx.push_cache_index(cache_slot);
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push(Buf::set_block_tracking_alias());
    cx.buf.push("(1),");
    cx.buf.newline();
    cx.buf.push("_cache[");
    cx.push_cache_index(cache_slot);
    cx.buf.push("]");
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push(")");
    Ok(())
}

pub(super) fn emit_hoisted_child(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<bool, EmitError> {
    if cx.once_depth == 0 || !cx.hoist_static || !is_hoistable(element, cx.is_ts) {
        return Ok(false);
    }
    let alias = super::hoist::hoist_static_element(cx, element);
    cx.buf.push(alias.as_str());
    Ok(true)
}
