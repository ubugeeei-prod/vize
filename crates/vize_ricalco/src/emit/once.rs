//! Native-element `vue.once` realization.

use vize_s0::ToCompactString;
use vize_s2::op::{BindingOp, ElementOp};

use super::buf::Buf;
use super::directive;
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::vnode;
use super::{EmitCx, EmitError};

pub(super) fn has(bindings: &[BindingOp<'_>]) -> bool {
    bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::VueOnce(_)))
}

pub(super) fn emit_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: Option<&str>,
    for_item: bool,
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
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_create_element_vnode();
        cx.once_depth += 1;
        let result = vnode::emit_call(
            cx, element, /* block */ false, key, /* hoist */ false, for_item,
            /* once */ true,
        );
        cx.once_depth -= 1;
        result
    })?;
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
    emit_hoisted_element(cx, element)?;
    Ok(true)
}
