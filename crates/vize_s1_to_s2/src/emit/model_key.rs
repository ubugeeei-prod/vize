//! Object key spelling for component `ui.model` product props.

use oxc_ast::ast as js;
use vize_s0::{Span, String, camelize};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::ModelOp;

use super::js::push_ident_key;
use super::prefix::Site;
use super::{EmitCx, EmitError};

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

pub(super) fn emit_value(
    cx: &mut EmitCx<'_>,
    name: ModelName<'_>,
    source: &str,
) -> Result<(), EmitError> {
    emit_model_name(cx, name)?;
    cx.buf.push(": ");
    cx.buf.push(source);
    Ok(())
}

pub(super) fn emit_update(
    cx: &mut EmitCx<'_>,
    key: &ModelUpdateKey<'_>,
    model: &ModelOp<'_>,
    source: &str,
) -> Result<(), EmitError> {
    emit_update_key(cx, key)?;
    cx.buf.push(": ");
    emit_cached_assignment(cx, model, source)
}

/// The synthesized `onUpdate:` assignment is an inline handler, so
/// `cache_handlers` hoists it into the same `_cache` array the authored
/// ones use. Shared with the merged-handler array, where a `v-model` and
/// an authored listener on the same key both take a slot.
pub(super) fn emit_cached_assignment(
    cx: &mut EmitCx<'_>,
    model: &ModelOp<'_>,
    source: &str,
) -> Result<(), EmitError> {
    let cached = cx.caches_handlers();
    if cached {
        let slot = cx.once_cache_index;
        cx.once_cache_index += 1;
        cx.buf.push("_cache[");
        cx.push_cache_index(slot);
        cx.buf.push("] || (_cache[");
        cx.push_cache_index(slot);
        cx.buf.push("] = ");
    }
    emit_assignment(cx, model, source)?;
    if cached {
        cx.buf.push(")");
    }
    Ok(())
}

pub(super) fn emit_modifiers(
    cx: &mut EmitCx<'_>,
    name: &ModelModifiersKey<'_>,
    modifiers: &[&str],
) -> Result<(), EmitError> {
    emit_modifiers_key(cx, name)?;
    cx.buf.push(": { ");
    for (i, modifier) in modifiers.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        cx.buf.push(modifier);
        cx.buf.push(": true");
    }
    cx.buf.push(" }");
    Ok(())
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

/// A dynamic model argument, `generate_expression` over the transform's
/// prefixed text.
fn push_argument(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) -> Result<(), EmitError> {
    if cx.prefixing() {
        return cx.push_prefixed_js(js, Site::Expression);
    }
    super::js::push_js_expr(cx, js);
    Ok(())
}

fn emit_model_name(cx: &mut EmitCx<'_>, name: ModelName<'_>) -> Result<(), EmitError> {
    match name {
        ModelName::Static(name) => push_ident_key(cx, name),
        ModelName::Dynamic(js) => {
            cx.buf.push("[");
            push_argument(cx, js)?;
            cx.buf.push("]");
        }
    }
    Ok(())
}

fn emit_update_key(cx: &mut EmitCx<'_>, key: &ModelUpdateKey<'_>) -> Result<(), EmitError> {
    match key {
        ModelUpdateKey::Static(key) => push_ident_key(cx, key.as_str()),
        ModelUpdateKey::Dynamic(js) => {
            cx.buf.push("[\"onUpdate:\" + ");
            push_argument(cx, js)?;
            cx.buf.push("]");
        }
    }
    Ok(())
}

fn emit_modifiers_key(cx: &mut EmitCx<'_>, key: &ModelModifiersKey<'_>) -> Result<(), EmitError> {
    match key {
        ModelModifiersKey::Static(key) => push_ident_key(cx, key.as_str()),
        ModelModifiersKey::Dynamic(js) => {
            cx.buf.push("[");
            push_argument(cx, js)?;
            cx.buf.push(" + \"Modifiers\"]");
        }
    }
    Ok(())
}

pub(super) fn emit_assignment(
    cx: &mut EmitCx<'_>,
    model: &ModelOp<'_>,
    source: &str,
) -> Result<(), EmitError> {
    if cx.prefixing() {
        return emit_prefixed_assignment(cx, model, source);
    }
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
        return Ok(());
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
    Ok(())
}

/// The shipped `v-model` write under `prefix_identifiers`: the transform
/// synthesizes `$event => ((<raw value>) = $event)` from the authored
/// (padded) value text and runs it through `process_inline_handler`.
fn emit_prefixed_assignment(
    cx: &mut EmitCx<'_>,
    model: &ModelOp<'_>,
    source: &str,
) -> Result<(), EmitError> {
    let read = model.contract.read;
    let (leading, trailing) =
        authored_model_padding(cx.source, model.span, source, read.span()).unwrap_or(("", ""));
    let mut handler = String::with_capacity(source.len() + leading.len() + trailing.len() + 24);
    handler.push_str("$event => ((");
    handler.push_str(leading);
    handler.push_str(read.source());
    handler.push_str(trailing);
    handler.push_str(") = $event)");
    let text = cx.prefixed_handler_text(handler.as_str())?;
    cx.buf.push(text.as_str());
    Ok(())
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
