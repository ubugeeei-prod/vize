//! `vue.once` realization.

use vize_davinci::id::NodeId;
use vize_s0::ToCompactString;
use vize_s2::op::{BindingOp, ComponentOp, ElementOp, Op};

use super::buf::Buf;
use super::builtin;
use super::directive;
use super::hoist::is_hoistable;
use super::js::asset_ident;
use super::vnode;
use super::{EmitCx, EmitError};

pub(super) fn has(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(is_once)
}

pub(super) fn is_once(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::VueOnce(_))
}

pub(super) fn emit_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: Option<&str>,
    for_item: bool,
) -> Result<(), EmitError> {
    emit_cached(cx, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            cx.once_depth += 1;
            let result = vnode::emit_call(
                cx,
                element,
                /* block */ false,
                key,
                /* hoist */ (false, None),
                for_item,
                /* once */ true,
            );
            cx.once_depth -= 1;
            result
        })
    })
}

pub(super) fn emit_component(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    _key: Option<&str>,
    _for_item: bool,
    _id: Option<NodeId>,
) -> Result<(), EmitError> {
    skip_or_hoist_component_children(cx, component);
    emit_cached(cx, |cx| {
        directive::wrap_component(cx, component, |cx| {
            cx.buf.use_create_vnode();
            cx.buf.push(Buf::create_vnode_alias());
            cx.buf.push("(");
            emit_component_target(cx, component)?;
            cx.buf.push(")");
            Ok(())
        })
    })
}

fn emit_component_target(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
) -> Result<(), EmitError> {
    if builtin::emit_dynamic_tag(cx, component)? {
    } else if let Some(helper) = builtin::helper(component.name) {
        cx.buf.use_helper(helper);
        cx.buf.push(helper.alias());
    } else {
        cx.buf
            .push(asset_ident("component", component.name).as_str());
    }
    Ok(())
}

fn skip_or_hoist_component_children(cx: &mut EmitCx<'_>, component: &ComponentOp<'_>) {
    for op in component.children.ops.iter() {
        match op {
            Op::Element(element) if super::hoist::is_hoistable(element) => {
                let _id = cx.walk.mint();
                cx.walk.skip(element.bindings.len());
                let _alias = super::hoist::hoist_static_element(cx, element);
            }
            _ => super::create_slots_walk::skip_ops(cx, core::slice::from_ref(op)),
        }
    }
}

fn emit_cached(
    cx: &mut EmitCx<'_>,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let cache_index = cx.once_cache_index;
    cx.once_cache_index += 1;
    let cache_index = cache_index.to_compact_string();
    cx.buf.use_set_block_tracking();
    cx.buf.push("_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] || (");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push(Buf::set_block_tracking_alias());
    cx.buf.push("(-1, true),");
    cx.buf.newline();
    cx.buf.push("(_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] = ");
    emit(cx)?;
    cx.buf.push(").cacheIndex = ");
    cx.buf.push(cache_index.as_str());
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push(Buf::set_block_tracking_alias());
    cx.buf.push("(1),");
    cx.buf.newline();
    cx.buf.push("_cache[");
    cx.buf.push(cache_index.as_str());
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
    if cx.once_depth == 0 || !is_hoistable(element) {
        return Ok(false);
    }
    let alias = super::hoist::hoist_static_element(cx, element);
    cx.buf.push(alias.as_str());
    Ok(true)
}
