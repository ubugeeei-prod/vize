//! Object key spelling for component `ui.model` product props.

use vize_s0::{String, camelize};
use vize_s2::expr::JsExpr;

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

pub(super) fn emit_update(cx: &mut EmitCx<'_>, key: &ModelUpdateKey<'_>, source: &str) {
    emit_update_key(cx, key);
    cx.buf.push(": ");
    emit_assignment(cx, source);
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

pub(super) fn emit_assignment(cx: &mut EmitCx<'_>, source: &str) {
    cx.buf.push("$event => ((");
    cx.buf.push(source);
    cx.buf.push(") = $event)");
}
