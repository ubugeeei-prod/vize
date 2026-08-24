//! Native HTML `ui.if` (`v-if` / `v-else-if` / `v-else`) emission.

use vize_carton::{String, ToCompactString};
use vize_davinci::id::NodeId;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{IfBranch, IfOp, Op};

use super::buf::Buf;
use super::js::escape_js_string;
use super::EmitCx;
use super::EmitError;
use crate::pass::{BranchKeyKind, IfFacts};

pub(super) fn emit_if(
    cx: &mut EmitCx<'_>,
    if_op: &IfOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if if_op.branches.is_empty() {
        return Err(EmitError::Unsupported);
    }
    cx.buf.use_open_block();
    cx.buf.use_create_comment();
    let facts = id.and_then(|id| cx.facts.if_facts.get(id));
    for (i, branch) in if_op.branches.iter().enumerate() {
        let allocated = next_if_key(cx);
        if let Some(condition) = &branch.condition {
            if i == 0 {
                cx.buf.push("(");
                emit_condition(cx, condition)?;
                cx.buf.push(")");
                cx.buf.indent();
                cx.buf.newline();
                cx.buf.push("? ");
            } else {
                cx.buf.newline();
                cx.buf.push(": (");
                emit_condition(cx, condition)?;
                cx.buf.push(")");
                cx.buf.indent();
                cx.buf.newline();
                cx.buf.push("? ");
            }
        } else {
            cx.buf.newline();
            cx.buf.push(": ");
        }
        let key = branch_key_js(facts, i, allocated)?;
        let saved = cx.if_branch_key;
        cx.if_branch_key = 0;
        emit_branch(cx, branch, key.as_str())?;
        cx.if_branch_key = saved;
        if branch.condition.is_some() && i > 0 {
            cx.buf.deindent();
        }
    }
    if if_op
        .branches
        .iter()
        .all(|branch| branch.condition.is_some())
    {
        cx.buf.newline();
        cx.buf.push(": ");
        cx.buf.push(Buf::create_comment_alias());
        cx.buf.push("(\"v-if\", true)");
    }
    cx.buf.deindent();
    Ok(())
}

fn next_if_key(cx: &mut EmitCx<'_>) -> u32 {
    let key = cx.if_branch_key;
    cx.if_branch_key = cx.if_branch_key.saturating_add(1);
    key
}

fn emit_condition(cx: &mut EmitCx<'_>, condition: &ExprRef<'_>) -> Result<(), EmitError> {
    match condition {
        ExprRef::Js(js) => {
            cx.buf.push(js.source);
            Ok(())
        }
        _ => Err(EmitError::Unsupported),
    }
}

fn branch_key_js(
    facts: Option<&IfFacts>,
    index: usize,
    allocated: u32,
) -> Result<String, EmitError> {
    match facts
        .and_then(|facts| facts.branches.get(index))
        .and_then(|key| key.as_ref())
        .map(|key| &key.kind)
    {
        None | Some(BranchKeyKind::Static(None)) => Ok(allocated.to_compact_string()),
        Some(BranchKeyKind::Static(Some(value))) => {
            let mut out = String::from("\"");
            out.push_str(escape_js_string(value.as_str()).as_str());
            out.push('"');
            Ok(out)
        }
        Some(BranchKeyKind::Dynamic { .. }) => Err(EmitError::Unsupported),
    }
}

fn emit_branch(cx: &mut EmitCx<'_>, branch: &IfBranch<'_>, key: &str) -> Result<(), EmitError> {
    match branch.region.ops.as_slice() {
        [Op::Element(element)] => {
            let _id = cx.walk.mint();
            cx.walk.skip(element.bindings.len());
            super::emit_if_branch_call(cx, element, key)
        }
        [Op::Component(component)] => {
            let _id = cx.walk.mint();
            cx.walk.skip(component.bindings.len());
            super::component::emit_if_branch(cx, component, key, _id)
        }
        [Op::Slot(slot)] => {
            let _id = cx.walk.mint();
            cx.walk.skip(slot.bindings.len());
            super::outlet::emit_outlet(cx, slot, Some(key), true)
        }
        _ => Err(EmitError::Unsupported),
    }
}
