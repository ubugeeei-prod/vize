use oxc_ast::ast::{ChainElement, Expression};
use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::OnOp;

use super::super::{EmitCx, EmitError, UnsupportedReason as Reason, buf::Buf};
use super::Classified;

pub(in crate::emit) fn emit_wrapped_handler(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    classified: &Classified<'_>,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    if !classified.keys.is_empty() {
        cx.buf.use_with_keys();
        cx.buf.push(Buf::with_keys_alias());
        cx.buf.push("(");
    }
    if !classified.event.is_empty() {
        cx.buf.use_with_modifiers();
        cx.buf.push(Buf::with_modifiers_alias());
        cx.buf.push("(");
    }
    match on.handler {
        Some(ExprRef::Js(js)) => emit_handler(cx, on, js, is_plain_element),
        Some(ExprRef::Opaque(opaque)) if opaque.reason == OpaqueReason::MultiStatement => {
            super::super::on_body::emit(cx, opaque.source);
        }
        None => cx.buf.push("() => {}"),
        Some(expr) => {
            return Err(EmitError::unsupported_at(
                Reason::OnHandlerNotJs,
                expr.span(),
            ));
        }
    }
    if !classified.event.is_empty() {
        cx.buf.push(", ");
        emit_mod_array(cx, &classified.event);
        cx.buf.push(")");
    }
    if !classified.keys.is_empty() {
        cx.buf.push(", ");
        emit_mod_array(cx, &classified.keys);
        cx.buf.push(")");
    }
    Ok(())
}

fn emit_mod_array(cx: &mut EmitCx<'_>, mods: &[&str]) {
    cx.buf.push("[");
    for (i, modifier) in mods.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.push("\"");
        cx.buf.push(modifier);
        cx.buf.push("\"");
    }
    cx.buf.push("]");
}

fn emit_handler(cx: &mut EmitCx<'_>, on: &OnOp<'_>, js: &JsExpr<'_>, is_plain_element: bool) {
    if is_handler_reference(js.ast) || super::super::on_body::preserves_raw_function_handler(js) {
        emit_raw_handler(cx, on, js);
        return;
    }
    if is_raw_function_handler(js.ast, is_plain_element)
        && !super::super::on_typed::is_typed_arrow(js.ast)
    {
        super::super::js::push_js_expr(cx, js);
        return;
    }
    if js.source.contains(';') {
        super::super::on_body::emit(cx, js.source);
    } else {
        cx.buf.push("$event => (");
        super::super::js::push_js_expr(cx, js);
        cx.buf.push(")");
    }
}

fn emit_raw_handler(cx: &mut EmitCx<'_>, on: &OnOp<'_>, js: &JsExpr<'_>) {
    if let Some((leading, trailing)) = authored_handler_padding(cx.source, on, js.source, js.span) {
        cx.buf.push(leading);
        cx.buf.push(js.source);
        cx.buf.push(trailing);
    } else {
        cx.buf.push(js.source);
    }
}

fn authored_handler_padding<'a>(
    source: &'a str,
    on: &OnOp<'_>,
    value: &str,
    value_span: Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(on.span.start).ok()?;
    let attr_end = usize::try_from(on.span.end).ok()?;
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

fn is_raw_function_handler(expr: &Expression<'_>, is_plain_element: bool) -> bool {
    if matches!(expr, Expression::FunctionExpression(_)) {
        return true;
    }
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            !is_plain_element || !arrow.params.items.is_empty()
        }
        _ => false,
    }
}
