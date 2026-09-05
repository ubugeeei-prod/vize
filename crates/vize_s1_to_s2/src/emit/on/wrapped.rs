use oxc_ast::ast::{ChainElement, Expression};
use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::OnOp;

use super::super::{EmitCx, EmitError, UnsupportedReason as Reason, buf::Buf};
use super::Classified;
use super::cache::needs_handler_cache;

pub(in crate::emit) fn emit_wrapped_handler(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    classified: &Classified<'_>,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    let cached = needs_handler_cache(cx, on);
    if cached {
        let slot = cx.once_cache_index;
        cx.once_cache_index += 1;
        cx.buf.push("_cache[");
        cx.push_cache_index(slot);
        cx.buf.push("] || (_cache[");
        cx.push_cache_index(slot);
        cx.buf.push("] = ");
    }
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
    let options_api = on
        .handler
        .and_then(|expr| options_api_handler_name(cx, &expr));
    match (options_api, on.handler) {
        // `generate_options_api_handler_reference`: a bare Options API
        // method name is guarded and forwarded, never prefixed.
        (Some(name), _) => {
            cx.buf.push("(...args) => (_ctx.");
            cx.buf.push(name);
            cx.buf.push(" && _ctx.");
            cx.buf.push(name);
            cx.buf.push("(...args))");
        }
        (None, Some(expr)) if cx.prefixing() => {
            let text = cx.prefixed_handler(&expr, cached)?;
            cx.buf.push(text.as_str());
        }
        (None, Some(ExprRef::Js(js))) => emit_handler(cx, on, js, is_plain_element, cached),
        (None, Some(ExprRef::Opaque(opaque))) if opaque.reason == OpaqueReason::MultiStatement => {
            let padding = authored_handler_padding(cx.source, on, opaque.source, opaque.span);
            // The shipped codegen prefix-parses the text: `a; b` reads as the
            // reference `a` and is pushed raw.
            if super::super::prefix::handler_source_is_reference(opaque.source) {
                let (leading, trailing) = padding.unwrap_or(("", ""));
                cx.buf.push(leading);
                cx.buf.push(opaque.source);
                cx.buf.push(trailing);
            } else if !opaque.source.contains(';')
                && super::super::prefix::handler_source_is_expression(opaque.source)
            {
                // An expression the prefix parse admits with no `;` is
                // paren-wrapped by the shipped codegen, trailing line
                // comment and all; statement bodies keep the block form.
                let (leading, trailing) = padding.unwrap_or(("", ""));
                cx.buf.push("$event => (");
                cx.buf.push(leading);
                cx.buf.push(opaque.source);
                cx.buf.push(trailing);
                cx.buf.push(")");
            } else {
                super::super::on_body::emit(cx, opaque.source, padding);
            }
        }
        (None, None) => cx.buf.push("() => {}"),
        (None, Some(expr)) => {
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
    if cached {
        cx.buf.push(")");
    }
    Ok(())
}

/// `options_api_handler_name`: the trimmed authored text is a simple
/// identifier the binding table records as an Options API member.
fn options_api_handler_name<'a>(cx: &EmitCx<'_>, expr: &ExprRef<'a>) -> Option<&'a str> {
    let source = match expr {
        ExprRef::Js(js) => js.source,
        ExprRef::Opaque(opaque) => opaque.source,
        ExprRef::Foreign(_) | ExprRef::Filter(_) => return None,
    }
    .trim();
    if !super::super::prefix::is_simple_identifier(source) {
        return None;
    }
    cx.scope
        .bindings()
        .and_then(|table| table.kind(source))
        .filter(|kind| *kind == super::super::options::BindingKind::Options)
        .map(|_| source)
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

fn emit_handler(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    js: &JsExpr<'_>,
    is_plain_element: bool,
    for_caching: bool,
) {
    if for_caching && is_handler_reference(js.ast) {
        // `generate_event_handler(for_caching)`: a cached reference is
        // guarded and forwarded instead of stored bare.
        cx.buf.push("(...args) => (");
        emit_raw_handler(cx, on, js);
        cx.buf.push(" && ");
        emit_raw_handler(cx, on, js);
        cx.buf.push("(...args))");
        return;
    }
    if is_handler_reference(js.ast)
        || (super::super::on_body::preserves_raw_function_handler(js)
            && !super::super::on_typed::uses_ts_only_syntax(js.ast))
    {
        emit_raw_handler(cx, on, js);
        return;
    }
    if super::super::on_typed::legacy_raw_non_null_call(js.ast) {
        emit_raw_handler_expr(cx, on, js);
        return;
    }
    if super::super::on_typed::legacy_raw_non_null_assignment(js.ast) {
        emit_raw_handler_expr(cx, on, js);
        return;
    }
    if is_raw_handler_expression(js.ast, is_plain_element)
        && !super::super::on_typed::uses_ts_only_syntax(js.ast)
    {
        emit_raw_handler_expr(cx, on, js);
        return;
    }
    if js.source.contains(';') {
        let padding = authored_handler_padding(cx.source, on, js.source, js.span);
        super::super::on_body::emit(cx, js.source, padding);
    } else {
        cx.buf.push("$event => (");
        emit_authored_handler_expr(cx, on, js);
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

fn emit_raw_handler_expr(cx: &mut EmitCx<'_>, on: &OnOp<'_>, js: &JsExpr<'_>) {
    let source = super::super::js::js_expr_source(js);
    if let Some((leading, trailing)) =
        authored_handler_padding(cx.source, on, source.as_str(), js.span)
    {
        cx.buf.push(leading);
        cx.buf.push(source.as_str());
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source.as_str());
    }
}

fn emit_authored_handler_expr(cx: &mut EmitCx<'_>, on: &OnOp<'_>, js: &JsExpr<'_>) {
    let source = handler_expr_source(js);
    if let Some((leading, trailing)) =
        authored_handler_padding(cx.source, on, source.as_str(), js.span)
    {
        cx.buf.push(leading);
        cx.buf.push(source.as_str());
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source.as_str());
    }
}

fn handler_expr_source<'a>(js: &JsExpr<'a>) -> super::super::js::RawJs<'a> {
    match js.ast {
        Expression::ArrowFunctionExpression(arrow) if !arrow.expression => {
            super::super::js::RawJs::Borrowed(js.source)
        }
        Expression::FunctionExpression(function) if function.body.is_some() => {
            super::super::js::RawJs::Borrowed(js.source)
        }
        _ if js.source.contains("//")
            && !super::super::on_body::ends_in_line_comment(js.source) =>
        {
            super::super::js::RawJs::Borrowed(js.source)
        }
        _ => super::super::js::js_expr_source(js),
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

fn is_raw_handler_expression(expr: &Expression<'_>, is_plain_element: bool) -> bool {
    if matches!(expr, Expression::NullLiteral(_)) {
        return true;
    }
    if matches!(expr, Expression::FunctionExpression(_)) {
        return true;
    }
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            !is_plain_element
                || (!super::super::on_typed::uses_ts_only_syntax(expr)
                    && (!arrow.params.items.is_empty() || arrow.expression))
        }
        _ => false,
    }
}
