use vize_s2::op::{BindingOp, ElementOp, Op};

use super::buf::Buf;
use super::props_static::PropHoistPosition;
use super::{EmitCx, EmitError};

pub(super) fn emit_transition_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<bool, EmitError> {
    if !cx.transition_slot_root {
        return Ok(false);
    }
    let Op::Element(element) = op else {
        return Ok(false);
    };
    if !has_dynamic_key(element) || super::once::has(&element.bindings) {
        return Ok(false);
    }
    let id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        super::directive::wrap_element(cx, element, |cx| {
            cx.buf.use_open_block();
            cx.buf.use_create_element_block();
            cx.buf.push("(");
            cx.buf.push(Buf::open_block_alias());
            cx.buf.push("(), ");
            super::vnode::emit_call(
                cx,
                element,
                true,
                None,
                (true, id, PropHoistPosition::Nested),
                false,
                false,
            )?;
            cx.buf.push(")");
            Ok(())
        })
    })?;
    Ok(true)
}

fn has_dynamic_key(element: &ElementOp<'_>) -> bool {
    element.bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(bind) if super::props_bind::is_key_bind_name(bind)
        )
    })
}
