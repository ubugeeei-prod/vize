//! Object key spelling for component `ui.model` product props.

use oxc_ast::ast as js;
use vize_s0::{Span, String, camelize};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::ModelOp;

use super::EmitCx;
use super::js::push_ident_key;

#[derive(Clone, Copy)]
pub(super) enum ModelName<'a> {
    Static(&'a str),
    Dynamic(&'a JsExpr<'a>),
}

pub(super) enum ModelUpdateKey<'a> {
    Static(String),
    Dynamic(&'a JsExpr<'a>),
}

pub(super) enum ModelModifiersKey<'a> {
    Static(String),
    Dynamic(&'a JsExpr<'a>),
}

pub(super) fn emit_value(cx: &mut EmitCx<'_>, name: ModelName<'_>, source: &str) {
    emit_model_name(cx, name);
    cx.buf.push(": ");
    cx.buf.push(source);
}

pub(super) fn emit_update(
    cx: &mut EmitCx<'_>,
    key: &ModelUpdateKey<'_>,
    model: &ModelOp<'_>,
    source: &str,
) {
    emit_update_key(cx, key);
    cx.buf.push(": ");
    emit_assignment(cx, model, source);
}

pub(super) fn emit_modifiers(
    cx: &mut EmitCx<'_>,
    name: &ModelModifiersKey<'_>,
    modifiers: &[&str],
) {
    emit_modifiers_key(cx, name);
    cx.buf.push(": { ");
    for (i, modifier) in modifiers.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        cx.buf.push(modifier);
        cx.buf.push(": true");
    }
    cx.buf.push(" }");
}

pub(super) fn update_key_for(name: ModelName<'_>) -> ModelUpdateKey<'_> {
    match name {
        ModelName::Static(prop) => ModelUpdateKey::Static(static_update_key(prop)),
        ModelName::Dynamic(js) => ModelUpdateKey::Dynamic(js),
    }
}

pub(super) fn modifiers_key(name: ModelName<'_>) -> ModelModifiersKey<'_> {
    match name {
        ModelName::Static(prop) => ModelModifiersKey::Static(static_modifiers_key(prop)),
        ModelName::Dynamic(js) => ModelModifiersKey::Dynamic(js),
    }
}

pub(super) fn static_update_key(prop: &str) -> String {
    let mut key = String::from("onUpdate:");
    key.push_str(camelize(prop).as_str());
    key
}

fn static_modifiers_key(prop: &str) -> String {
    if prop == "modelValue" {
        return String::from("modelModifiers");
    }
    let mut key = String::from(prop);
    key.push_str("Modifiers");
    key
}

fn emit_model_name(cx: &mut EmitCx<'_>, name: ModelName<'_>) {
    match name {
        ModelName::Static(name) => push_ident_key(cx, name),
        ModelName::Dynamic(js) => {
            cx.buf.push("[");
            super::js::push_js_expr(cx, js);
            cx.buf.push("]");
        }
    }
}

fn emit_update_key(cx: &mut EmitCx<'_>, key: &ModelUpdateKey<'_>) {
    match key {
        ModelUpdateKey::Static(key) => push_ident_key(cx, key.as_str()),
        ModelUpdateKey::Dynamic(js) => {
            cx.buf.push("[\"onUpdate:\" + ");
            super::js::push_js_expr(cx, js);
            cx.buf.push("]");
        }
    }
}

fn emit_modifiers_key(cx: &mut EmitCx<'_>, key: &ModelModifiersKey<'_>) {
    match key {
        ModelModifiersKey::Static(key) => push_ident_key(cx, key.as_str()),
        ModelModifiersKey::Dynamic(js) => {
            cx.buf.push("[");
            super::js::push_js_expr(cx, js);
            cx.buf.push(" + \"Modifiers\"]");
        }
    }
}

pub(super) fn emit_assignment(cx: &mut EmitCx<'_>, model: &ModelOp<'_>, source: &str) {
    if requires_legacy_nested_assignment(&model.contract.read) {
        cx.buf.push("$event => ($event => ($event => ((");
        if let Some((leading, trailing)) =
            authored_model_padding(cx.source, model.span, source, model.contract.read.span())
        {
            cx.buf.push(leading);
            cx.buf.push(source);
            cx.buf.push(trailing);
        } else {
            cx.buf.push(source);
        }
        cx.buf.push(") = $event)))");
        return;
    }
    cx.buf.push("$event => ((");
    if let Some((leading, trailing)) =
        authored_model_padding(cx.source, model.span, source, model.contract.read.span())
    {
        cx.buf.push(leading);
        cx.buf.push(source);
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source);
    }
    cx.buf.push(") = $event)");
}

fn authored_model_padding<'a>(
    source: &'a str,
    model_span: Span,
    value: &str,
    value_span: Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(model_span.start).ok()?;
    let attr_end = usize::try_from(model_span.end).ok()?;
    let value_start = usize::try_from(value_span.start).ok()?;
    let value_end = usize::try_from(value_span.end).ok()?;
    if attr_start > value_start
        || value_start > value_end
        || value_end > attr_end
        || attr_end > source.len()
        || source.get(value_start..value_end)? != value
    {
        return None;
    }
    let before = source.get(attr_start..value_start)?;
    let quote_pos = before
        .as_bytes()
        .iter()
        .rposition(|byte| matches!(*byte, b'\'' | b'"'))?;
    let quote = before.as_bytes()[quote_pos];
    let leading = before.get(quote_pos + 1..)?;
    let after = source.get(value_end..attr_end)?;
    let trailing_end = after
        .as_bytes()
        .iter()
        .position(|byte| *byte == quote)
        .unwrap_or(after.len());
    let trailing = after.get(..trailing_end)?;
    if leading.is_empty() && trailing.is_empty() {
        return None;
    }
    (leading.bytes().all(|byte| byte.is_ascii_whitespace())
        && trailing.bytes().all(|byte| byte.is_ascii_whitespace()))
    .then_some((leading, trailing))
}

fn requires_legacy_nested_assignment(read: &ExprRef<'_>) -> bool {
    let ExprRef::Js(js) = read else {
        return false;
    };
    if matches!(
        js.ast,
        js::Expression::BinaryExpression(_)
            | js::Expression::ConditionalExpression(_)
            | js::Expression::LogicalExpression(_)
    ) {
        return true;
    }
    super::on_typed::uses_ts_only_syntax(js.ast)
}
