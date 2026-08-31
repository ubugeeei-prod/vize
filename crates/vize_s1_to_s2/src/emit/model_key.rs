//! Object key spelling for component `ui.model` product props.

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use vize_s0::{String, camelize};
use vize_s2::expr::{ExprRef, JsExpr};

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
    read: &ExprRef<'_>,
    source: &str,
) {
    emit_update_key(cx, key);
    cx.buf.push(": ");
    emit_assignment(cx, read, source);
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

pub(super) fn emit_assignment(cx: &mut EmitCx<'_>, read: &ExprRef<'_>, source: &str) {
    if contains_ts_only_syntax(read) {
        cx.buf.push("$event => ($event => ($event => ((");
        cx.buf.push(source);
        cx.buf.push(") = $event)))");
        return;
    }
    cx.buf.push("$event => ((");
    cx.buf.push(source);
    cx.buf.push(") = $event)");
}

fn contains_ts_only_syntax(read: &ExprRef<'_>) -> bool {
    let ExprRef::Js(js) = read else {
        return false;
    };
    let mut scan = TsOnlySyntaxScan { seen: false };
    scan.visit_expression(js.ast);
    scan.seen
}

struct TsOnlySyntaxScan {
    seen: bool,
}

impl<'a> Visit<'a> for TsOnlySyntaxScan {
    fn visit_ts_as_expression(&mut self, _expr: &js::TSAsExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_satisfies_expression(&mut self, _expr: &js::TSSatisfiesExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_type_assertion(&mut self, _expr: &js::TSTypeAssertion<'a>) {
        self.seen = true;
    }

    fn visit_ts_non_null_expression(&mut self, _expr: &js::TSNonNullExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_instantiation_expression(&mut self, _expr: &js::TSInstantiationExpression<'a>) {
        self.seen = true;
    }
}
