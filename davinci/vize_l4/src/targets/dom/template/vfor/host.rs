//! Host-language loops (`ForOp::host`, TSRX `@for`): the `renderList`
//! callback takes the authored left-side pattern and the `index` binding.
//! Every item is keyed, as in the TSRX Vue profile's `VaporFor`: by the head's
//! `key`, else by the item's own `key`, else by the `index` binding, else by
//! the item value itself. A body of several nodes renders a keyed fragment
//! per item. A type annotation on the left side is erased with the rest.
//!
//! `@empty` renders when the collection has no items, as the reference tests
//! it: `(items).length !== 0 ? <loop> : <empty>`, a keyed branch pair like
//! `v-if`/`v-else`, whose empty branch is one element or component.
//!
//! Refused (`HostForLoop`): `for await`, an assignment left side, a key on
//! both the head and the item, a destructured item without a key or an index
//! (its value has no name), items a loop key cannot reach (slot outlets,
//! `v-memo` items, `<template>` wrappers), and an `@empty` region of other
//! shapes or on a loop that is itself a branch.

use oxc_span::GetSpan;
use vize_l0::id::NodeId;
use vize_l0::{String, ToCompactString};
use vize_l2::expr::ExprRef;
use vize_l2::op::{ForOp, HostFor, Op};

use super::super::js::js_expr_source;
use super::super::prefix::Site;
use super::super::{EmitCx, EmitError, UnsupportedReason as Reason};
use super::{emit_for_scoped, js_source};

pub(super) fn emit_for(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    host: &HostFor<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
) -> Result<(), EmitError> {
    let refuse = || EmitError::unsupported_at(Reason::HostForLoop, for_op.span);
    if host.r#await || (host.empty.is_some() && fragment_key.is_some()) {
        return Err(refuse());
    }
    let wrapped = id.is_some_and(|id| cx.for_wrappers.get(id).is_some());
    if wrapped
        || matches!(for_op.region.ops.as_slice(), [Op::Slot(_)])
        || super::plain_item_memo(&for_op.region.ops).is_some()
    {
        return Err(refuse());
    }
    let oxc_ast::ast::ForStatementLeft::VariableDeclaration(declaration) = host.left else {
        return Err(refuse());
    };
    let [declarator] = declaration.declarations.as_slice() else {
        return Err(refuse());
    };
    if declarator.init.is_some() {
        return Err(refuse());
    }
    let pattern = declarator.id.span();
    let value = cx
        .source
        .get(pattern.start as usize..pattern.end as usize)
        .ok_or_else(refuse)?;
    let index = host.index.map(|index| index.name.as_str());
    let source_raw = js_source(&for_op.binding.source)?;
    let source_prefixed;
    let source = if cx.prefixing() {
        source_prefixed = cx.prefixed_expr(&for_op.binding.source, Site::Expression)?;
        source_prefixed.as_str()
    } else {
        source_raw.as_str()
    };
    let Some(empty) = &host.empty else {
        return emit_loop(cx, for_op, id, fragment_key, source, value, index);
    };
    let empty_root = match empty.ops.as_slice() {
        [root @ (Op::Element(_) | Op::Component(_))] => root,
        _ => return Err(refuse()),
    };
    cx.use_open_block();
    let loop_key = next_branch_key(cx);
    let empty_key = next_branch_key(cx);
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(").length !== 0");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push("? ");
    emit_loop(cx, for_op, id, Some(&loop_key), source, value, index)?;
    cx.buf.newline();
    cx.buf.push(": ");
    // The walk visits the empty region after the body.
    let empty_id = cx.walk.mint();
    match empty_root {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            super::super::emit_if_branch_call(cx, element, &empty_key, empty_id)?;
        }
        Op::Component(component) => {
            cx.walk.skip(component.bindings.len());
            super::super::component::emit_if_branch(cx, component, &empty_key, empty_id)?;
        }
        _ => return Err(refuse()),
    }
    cx.buf.deindent();
    Ok(())
}

fn emit_loop(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
    source: &str,
    value: &str,
    index: Option<&str>,
) -> Result<(), EmitError> {
    // Arrays pass the index second, where a directive loop binds its key.
    let prefix_mark = cx.enter_host_for_scope([Some(value), index, None]);
    let result = emit_for_scoped(cx, for_op, id, fragment_key, source, value, index, None);
    cx.leave_scope(prefix_mark);
    result
}

/// The next key of the enclosing `v-if` chain counter.
fn next_branch_key(cx: &mut EmitCx<'_>) -> String {
    let key = cx.if_branch_key;
    cx.if_branch_key = cx.if_branch_key.saturating_add(1);
    key.to_compact_string()
}

/// The item's key: the head's `key` (which the item must not repeat), the
/// item's own, the `index` binding, or the item value's binding.
pub(super) fn item_key(
    cx: &EmitCx<'_>,
    host: &HostFor<'_>,
    own: Option<String>,
    span: vize_l0::Span,
) -> Result<Option<String>, EmitError> {
    let refuse = || EmitError::unsupported_at(Reason::HostForLoop, span);
    match (own, &host.key) {
        (None, Some(ExprRef::Js(js))) => {
            if cx.prefixing() {
                return cx.prefixed_js(js, Site::Expression).map(Some);
            }
            Ok(Some(String::from(js_expr_source(js).as_str())))
        }
        (Some(own), None) => Ok(Some(own)),
        (None, None) => {
            if let Some(index) = host.index {
                return Ok(Some(String::from(index.name.as_str())));
            }
            let value = value_binding(host).ok_or_else(refuse)?;
            Ok(Some(String::from(value)))
        }
        _ => Err(refuse()),
    }
}

/// The left side's name when it binds the whole item.
fn value_binding<'a>(host: &HostFor<'a>) -> Option<&'a str> {
    let oxc_ast::ast::ForStatementLeft::VariableDeclaration(declaration) = host.left else {
        return None;
    };
    match declaration.declarations.as_slice() {
        [declarator] => match &declarator.id {
            oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
            _ => None,
        },
        _ => None,
    }
}
