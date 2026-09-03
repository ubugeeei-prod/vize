use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{BindingOp, ElementOp, Op, Region, TextOp};

use crate::emit::children::{emit_slot_text_child, emit_slot_text_run};
use crate::emit::hoist::{emit_hoisted_element, is_static_element_tree};
use crate::emit::vnode::emit_array_child;
use crate::emit::{EmitCx, EmitError};

pub(in crate::emit) fn emit_template_pieces(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    bucket: &mut StdVec<String>,
) -> Result<(), EmitError> {
    if children.ops.iter().all(is_whitespace_text) {
        for op in children.ops.iter() {
            let _id = cx.walk.mint();
            let _ = op;
        }
        return Ok(());
    }
    if !children.ops.is_empty()
        && children
            .ops
            .iter()
            .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        bucket.push(capture(cx, |cx| {
            emit_slot_text_run(cx, children.ops.as_slice())
        })?);
        return Ok(());
    }
    for op in children.ops.iter() {
        bucket.push(capture_child(cx, op)?);
    }
    Ok(())
}

pub(in crate::emit) fn capture_child(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
) -> Result<String, EmitError> {
    capture(cx, |cx| emit_slot_child(cx, op))
}

pub(in crate::emit) fn capture(
    cx: &mut EmitCx<'_>,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<String, EmitError> {
    let start = cx.buf.code.len();
    write(cx)?;
    let piece = String::from(&cx.buf.code.as_str()[start..]);
    cx.buf.code.truncate(start);
    Ok(piece)
}

fn emit_slot_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    if crate::emit::slot_root::emit_transition_child(cx, op)? {
        return Ok(());
    }
    match op {
        Op::Text(_) | Op::Interpolation(_) => emit_slot_text_child(cx, op),
        Op::Element(element) if is_static_element_tree(element, cx.is_ts) => {
            emit_hoisted_element(cx, element)
        }
        Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
            emit_array_child(cx, op, false, false)
        }
    }
}

pub(in crate::emit) fn is_slot_template(element: &ElementOp<'_>) -> bool {
    element.tag == "template"
        && element
            .bindings
            .iter()
            .any(|binding| matches!(binding, BindingOp::SlotContent(_)))
}

pub(in crate::emit) fn is_whitespace_text(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if is_whitespace(text))
}

fn is_whitespace(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}
