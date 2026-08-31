//! Native element v-for item wrappers.

use vize_davinci::id::NodeId;
use vize_s2::op::ElementOp;

use super::buf::Buf;
use super::props_static::PropHoistPosition;
use super::{EmitCx, EmitError, directive, vnode};

pub(super) fn emit_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    register_props_hoist(cx, element, id)?;
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        if stable {
            return directive::wrap_element(cx, element, |cx| {
                cx.buf.use_create_element_vnode();
                cx.with_static_vnode_hoist(true, |cx| {
                    vnode::emit_call(
                        cx,
                        element,
                        false,
                        key,
                        (false, None, PropHoistPosition::Nested),
                        true,
                        false,
                    )
                })
            });
        }
        emit_block(cx, element, key)
    })
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: Option<&str>,
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        cx.with_static_vnode_hoist(true, |cx| {
            vnode::emit_call(
                cx,
                element,
                true,
                key,
                (false, None, PropHoistPosition::Nested),
                true,
                false,
            )
        })?;
        cx.buf.push(")");
        Ok(())
    })
}

fn register_props_hoist(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if !super::props_static::should_hoist(cx, id, PropHoistPosition::ForItem) {
        return Ok(());
    }
    if let Some(props) =
        super::props_static::root_hoist_props(&element.attributes, &element.bindings)?
    {
        let _ = cx.buf.push_hoist(props);
    }
    Ok(())
}
