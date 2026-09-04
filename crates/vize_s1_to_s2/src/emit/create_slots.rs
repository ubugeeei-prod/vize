//! `createSlots` for `v-if` / `v-for` slot templates (`{ _: 2 }` base
//! plus `{ name, fn }` entries — ternaries, `_renderList`, static named).
//! A `v-slots` spread lands in the base object (`...expr`) before `_: 2`.

mod branch_keys;
mod entry;

use alloc::vec::Vec as StdVec;

use vize_s2::op::{Op, Region};

use self::branch_keys::{
    default_branch_key_count, is_template_for_slot_outlet_entry, peek_id, with_branch_key,
};
use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::create_slots_walk::{
    advance_after_op, first_slot_template, is_slot_for, is_slot_if, slot_template_content,
};
use super::js::RawJs;
use super::slots::{SlotPiece, capture, capture_child, is_whitespace_text};

pub(super) fn needs_create_slots(cx: &EmitCx<'_>, children: &Region<'_>) -> bool {
    let mut walk = cx.walk.clone();
    children.ops.iter().any(|op| {
        let id = walk.mint();
        let needs = match op {
            Op::If(if_op) => is_slot_if(cx, id, if_op),
            Op::For(for_op) => is_slot_for(cx, id, for_op),
            _ => false,
        };
        advance_after_op(&mut walk, op);
        needs
    })
}

pub(super) fn emit_create_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    spread: Option<&RawJs<'_>>,
) -> Result<(), EmitError> {
    cx.buf.use_create_slots();
    cx.buf.use_with_ctx();
    cx.buf.indent();
    let (defaults, entries) = collect(cx, children)?;
    cx.buf.deindent();
    cx.buf.push(Buf::create_slots_alias());
    cx.buf.push("(");
    emit_base(cx, &defaults, spread);
    cx.buf.push(", [");
    cx.buf.indent();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.newline();
        cx.push_captured(entry);
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("])");
    Ok(())
}

fn collect(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
) -> Result<(StdVec<SlotPiece>, StdVec<SlotPiece>), EmitError> {
    let mut defaults = StdVec::new();
    let mut entries = StdVec::new();
    let first_branch_key = cx.if_branch_key;
    let mut default_branch_key = first_branch_key;
    let mut entry_branch_key =
        first_branch_key.saturating_add(default_branch_key_count(cx, children));
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    for op in children.ops.iter() {
        if skip_ws && is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        match op {
            Op::If(if_op) => {
                let is_slot = is_slot_if(cx, peek_id(cx), if_op);
                let walk_before = cx.walk.clone();
                let walk_after_default = if !is_slot {
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                    Some(cx.walk.clone())
                } else {
                    None
                };
                cx.walk = walk_before;
                entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                    capture(cx, |cx| entry::emit_if_entry(cx, if_op))
                })?);
                if let Some(walk_after) = walk_after_default {
                    cx.walk = walk_after;
                }
            }
            Op::For(for_op) => {
                let id = peek_id(cx);
                if is_slot_for(cx, id, for_op) {
                    let Some((idx, element, content)) = first_slot_template(&for_op.region) else {
                        return Err(EmitError::unsupported_at(
                            super::UnsupportedReason::CreateSlotsMissingSlotTemplate,
                            for_op.span,
                        ));
                    };
                    entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                        capture(cx, |cx| {
                            entry::emit_for_entry(cx, for_op, idx, element, content)
                        })
                    })?);
                } else if is_template_for_slot_outlet_entry(cx, id, for_op) {
                    let walk_before = cx.walk.clone();
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                    let walk_after_default = cx.walk.clone();
                    cx.walk = walk_before;
                    entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                        capture(cx, |cx| entry::emit_empty_for_slot_outlet_entry(cx, for_op))
                    })?);
                    cx.walk = walk_after_default;
                } else {
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                }
            }
            Op::Element(element) => {
                if let Some(content) = slot_template_content(element) {
                    entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                        capture(cx, |cx| entry::emit_slot_object(cx, element, content, None))
                    })?);
                } else {
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                }
            }
            _ => {
                with_branch_key(cx, &mut default_branch_key, |cx| {
                    collect_default(cx, &mut defaults, op)
                })?;
            }
        }
    }
    cx.if_branch_key = entry_branch_key;
    Ok((defaults, entries))
}

fn collect_default(
    cx: &mut EmitCx<'_>,
    defaults: &mut StdVec<SlotPiece>,
    op: &Op<'_>,
) -> Result<(), EmitError> {
    cx.buf.indent();
    defaults.push(capture_child(cx, op)?);
    cx.buf.deindent();
    Ok(())
}

fn emit_base(cx: &mut EmitCx<'_>, defaults: &[SlotPiece], spread: Option<&RawJs<'_>>) {
    if defaults.is_empty() && spread.is_none() {
        cx.buf.push("{ _: 2 /* DYNAMIC */ }");
        return;
    }
    cx.buf.push("{");
    cx.buf.indent();
    if !defaults.is_empty() {
        cx.buf.newline();
        cx.buf.push("default: ");
        cx.buf.push(Buf::with_ctx_alias());
        cx.buf.push("(() => [");
        cx.buf.indent();
        for (i, piece) in defaults.iter().enumerate() {
            if i > 0 {
                cx.buf.push(",");
            }
            cx.buf.newline();
            cx.push_captured(piece);
        }
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("]),");
    }
    if let Some(spread) = spread {
        cx.buf.newline();
        cx.buf.push("...");
        cx.buf.push(spread.as_str());
        cx.buf.push(",");
    }
    cx.buf.newline();
    cx.buf.push("_: 2 /* DYNAMIC */");
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
}
