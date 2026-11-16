//! Host-language loops (`ForOp::host`, TSRX `@for`): the `renderList`
//! callback takes the authored left-side pattern and the `index` binding, and
//! the head's `key` keys each item.
//!
//! Refused (`HostForLoop`): a loop without a head `key` (the TSRX Vue
//! profile keys those items by value, which a VDOM loop does not), `@empty`,
//! `for await`, an assignment or annotated left side, a key on both the head
//! and the item, and items the head key cannot reach (slot outlets, `v-memo`
//! items, `<template>` wrappers).

use oxc_span::GetSpan;
use vize_l0::String;
use vize_l0::id::NodeId;
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
    if host.key.is_none() || host.r#await || host.empty.is_some() {
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
    if declarator.init.is_some() || declarator.type_annotation.is_some() {
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
    // Arrays pass the index second, where a directive loop binds its key.
    let prefix_mark = cx.enter_host_for_scope([Some(value), index, None]);
    let result = emit_for_scoped(cx, for_op, id, fragment_key, source, value, index, None);
    cx.leave_scope(prefix_mark);
    result
}

/// The head's `key`, which the item must not repeat.
pub(super) fn item_key(
    cx: &EmitCx<'_>,
    host: &HostFor<'_>,
    own: Option<String>,
    span: vize_l0::Span,
) -> Result<Option<String>, EmitError> {
    let (None, Some(ExprRef::Js(js))) = (own, &host.key) else {
        return Err(EmitError::unsupported_at(Reason::HostForLoop, span));
    };
    if cx.prefixing() {
        return cx.prefixed_js(js, Site::Expression).map(Some);
    }
    Ok(Some(String::from(js_expr_source(js).as_str())))
}
