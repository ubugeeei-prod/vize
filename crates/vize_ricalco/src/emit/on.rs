//! Static-name `ui.on` (`@click` / `v-on:click`) without modifiers.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{ChainElement, Expression};
use vize_carton::{String, camelize, capitalize};
use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{DynamicName, OnOp};

use super::EmitCx;
use super::EmitError;
use super::js::is_valid_js_identifier;

pub(super) fn admit_on(on: &OnOp<'_>, seen: &mut StdVec<String>) -> Result<(), EmitError> {
    if !on.modifiers.is_empty() {
        return Err(EmitError::Unsupported);
    }
    let name = static_on_name(on)?;
    if name.contains(':') {
        return Err(EmitError::Unsupported);
    }
    match on.handler {
        None | Some(ExprRef::Js(_)) => {}
        Some(_) => return Err(EmitError::Unsupported),
    }
    let key = event_key(name);
    if seen.contains(&key) {
        return Err(EmitError::Unsupported);
    }
    seen.push(key);
    Ok(())
}

pub(super) fn static_on_name<'a>(on: &'a OnOp<'a>) -> Result<&'a str, EmitError> {
    match on.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

pub(super) fn event_key(raw: &str) -> String {
    if raw.chars().any(|c| c.is_ascii_uppercase()) {
        let mut key = String::with_capacity(raw.len() + 3);
        key.push_str("on:");
        key.push_str(raw);
        key
    } else {
        let camelized = camelize(raw);
        let mut key = String::with_capacity(camelized.len() + 2);
        key.push_str("on");
        key.push_str(capitalize(camelized.as_str()).as_str());
        key
    }
}

pub(super) fn needs_hydration(raw: &str) -> bool {
    raw != "click"
}

pub(super) fn is_inline_handler_source(source: &str) -> bool {
    source.contains('(')
        || source.contains('+')
        || source.contains('-')
        || source.contains('=')
        || source.contains(' ')
}

pub(super) fn emit_on_pair(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let key = event_key(static_on_name(on)?);
    if !is_valid_js_identifier(key.as_str()) {
        cx.buf.push("\"");
        cx.buf.push(key.as_str());
        cx.buf.push("\"");
    } else {
        cx.buf.push(key.as_str());
    }
    cx.buf.push(": ");
    match on.handler {
        Some(ExprRef::Js(js)) => emit_handler(cx, js),
        None => cx.buf.push("() => {}"),
        Some(_) => return Err(EmitError::Unsupported),
    }
    Ok(())
}

fn emit_handler(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    if is_handler_reference(js.ast) || is_function(js.ast) {
        cx.buf.push(js.source);
        return;
    }
    if js.source.contains(';') {
        cx.buf.push("$event => {");
        cx.buf.push(js.source);
        cx.buf.push("}");
    } else {
        cx.buf.push("$event => (");
        cx.buf.push(js.source);
        cx.buf.push(")");
    }
}

fn is_handler_reference(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::ChainExpression(chain) => matches!(
            chain.expression,
            ChainElement::StaticMemberExpression(_) | ChainElement::ComputedMemberExpression(_)
        ),
        _ => false,
    }
}

fn is_function(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}
