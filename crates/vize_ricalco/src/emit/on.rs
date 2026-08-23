//! Static-name `ui.on` (`@click` / `v-on:click`), including event / key /
//! option modifiers (`withModifiers` / `withKeys`, `onClickOnce`, …).

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{ChainElement, Expression};
use vize_carton::{String, camelize, capitalize};
use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{DynamicName, OnOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::is_valid_js_identifier;

struct Classified<'a> {
    options: StdVec<&'a str>,
    event: StdVec<&'a str>,
    keys: StdVec<&'a str>,
}

pub(super) fn admit_on(on: &OnOp<'_>, seen: &mut StdVec<String>) -> Result<(), EmitError> {
    let name = static_on_name(on)?;
    if name.contains(':') {
        return Err(EmitError::Unsupported);
    }
    classify(on)?;
    match on.handler {
        None | Some(ExprRef::Js(_)) => {}
        Some(_) => return Err(EmitError::Unsupported),
    }
    let key = event_key_for(on)?;
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

pub(super) fn event_key_for(on: &OnOp<'_>) -> Result<String, EmitError> {
    let classified = classify(on)?;
    let mut key = event_key(remapped_name(static_on_name(on)?, &classified.event));
    for option in &classified.options {
        key.push_str(capitalize(option).as_str());
    }
    Ok(key)
}

pub(super) fn needs_hydration(key: &str, on: &OnOp<'_>) -> bool {
    key != "onClick" || classify(on).is_ok_and(|classified| !classified.keys.is_empty())
}

pub(super) fn wraps_on(on: &OnOp<'_>) -> bool {
    classify(on).is_ok_and(|classified| !classified.event.is_empty() || !classified.keys.is_empty())
}

pub(super) fn is_inline_handler_source(source: &str) -> bool {
    source.contains('(')
        || source.contains('+')
        || source.contains('-')
        || source.contains('=')
        || source.contains(' ')
}

pub(super) fn emit_on_pair(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let classified = classify(on)?;
    let key = event_key_for(on)?;
    if !is_valid_js_identifier(key.as_str()) {
        cx.buf.push("\"");
        cx.buf.push(key.as_str());
        cx.buf.push("\"");
    } else {
        cx.buf.push(key.as_str());
    }
    cx.buf.push(": ");
    emit_wrapped_handler(cx, on, &classified)
}

fn emit_wrapped_handler(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    classified: &Classified<'_>,
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
        Some(ExprRef::Js(js)) => emit_handler(cx, js),
        None => cx.buf.push("() => {}"),
        Some(_) => return Err(EmitError::Unsupported),
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

fn classify<'a>(on: &'a OnOp<'a>) -> Result<Classified<'a>, EmitError> {
    let name = static_on_name(on)?;
    let keyboard = matches!(name, "keydown" | "keyup" | "keypress");
    let mut options = StdVec::new();
    let mut event = StdVec::new();
    let mut keys = StdVec::new();
    for modifier in on.modifiers.iter() {
        match *modifier {
            "native" => return Err(EmitError::Unsupported),
            "capture" | "once" | "passive" => options.push(*modifier),
            "left" | "right" if keyboard => keys.push(*modifier),
            "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "middle"
            | "exact" | "left" | "right" => event.push(*modifier),
            _ => keys.push(*modifier),
        }
    }
    Ok(Classified {
        options,
        event,
        keys,
    })
}

fn remapped_name<'a>(raw: &'a str, event: &[&str]) -> &'a str {
    if raw == "click" && event.contains(&"right") {
        "contextmenu"
    } else if raw == "click" && event.contains(&"middle") {
        "mouseup"
    } else {
        raw
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
