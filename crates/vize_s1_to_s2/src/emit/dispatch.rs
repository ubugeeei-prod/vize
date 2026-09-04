//! The structural-op forwarders `vif` / `vfor` / `tpl` call by name.
//!
//! They keep the shape those modules expect while the real emitters live
//! beside their own passes; split out of `emit.rs` under the 350-line
//! source budget.

use vize_davinci::id::NodeId;
use vize_s2::op::{ElementOp, ForOp, IfOp};

use super::{EmitCx, EmitError, vfor, vfor_item, vif, vnode};

pub(super) fn emit_if_op(
    cx: &mut EmitCx<'_>,
    if_op: &IfOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    vif::emit_if(cx, if_op, id)
}

pub(super) fn emit_if_branch_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    vnode::emit_if_branch_element(cx, element, key)
}

pub(super) fn emit_for_op(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
) -> Result<(), EmitError> {
    vfor::emit_for(cx, for_op, id, fragment_key)
}

pub(super) fn emit_for_item_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    vfor_item::emit_element(cx, element, id, stable, key)
}
