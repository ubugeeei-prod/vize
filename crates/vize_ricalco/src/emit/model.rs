//! `ui.model` realization for the DOM lane: component product props
//! (`modelValue` / `onUpdate:…` / `modelModifiers`) and native
//! `withDirectives` + `vModelText`-family helpers. The S2 contract
//! itself is never expanded; this module is the P2-11 spelling.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{BindingOp, ElementOp, ModelOp};

use super::EmitCx;
use super::EmitError;
use super::helper::Helper;
use super::js::push_ident_key;
use super::props::Piece;

const KIND: &str = "element-kind";
const ARGUMENT: &str = "argument";

pub(super) fn admit(model: &ModelOp<'_>) -> Result<(), EmitError> {
    js_source(model)?;
    Ok(())
}

pub(super) fn expand<'a>(
    model: &'a ModelOp<'a>,
    out: &mut StdVec<Piece<'a>>,
) -> Result<(), EmitError> {
    let source = js_source(model)?;
    let span = model.span;
    if is_component(model) {
        let prop = argument(model).unwrap_or("modelValue");
        out.push(Piece::ModelValue {
            name: prop,
            source,
            span,
        });
        out.push(Piece::ModelUpdate {
            key: update_key(prop),
            source,
            span,
        });
        let modifiers = component_modifiers(model);
        if !modifiers.is_empty() {
            out.push(Piece::ModelModifiers {
                name: modifiers_key(prop),
                modifiers,
                span,
            });
        }
    } else {
        out.push(Piece::ModelUpdate {
            key: update_key("modelValue"),
            source,
            span,
        });
    }
    Ok(())
}

pub(super) fn patch(
    model: &ModelOp<'_>,
    is_component: bool,
    flag: &mut i32,
    dynamic_props: &mut StdVec<String>,
) {
    *flag |= 8;
    patch_keys(model, is_component, dynamic_props);
}

pub(super) fn patch_keys(
    model: &ModelOp<'_>,
    is_component: bool,
    dynamic_props: &mut StdVec<String>,
) {
    if is_component {
        let prop = argument(model).unwrap_or("modelValue");
        push_dynamic(dynamic_props, prop);
        push_dynamic(dynamic_props, update_key(prop).as_str());
    } else {
        push_dynamic(dynamic_props, "onUpdate:modelValue");
    }
}

pub(super) fn first_runtime_model<'a>(element: &'a ElementOp<'a>) -> Option<&'a ModelOp<'a>> {
    if !matches!(element.tag, "input" | "textarea" | "select") {
        return None;
    }
    element.bindings.iter().find_map(|binding| match binding {
        BindingOp::Model(model) => Some(&**model),
        _ => None,
    })
}

pub(super) fn emit_native_entry(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    model: &ModelOp<'_>,
) -> Result<(), EmitError> {
    let helper = native_helper(element);
    let source = js_source(model)?;
    let modifiers = native_modifiers(model);
    cx.buf.use_helper(helper);
    if modifiers.is_empty() {
        cx.buf.push("  [");
        cx.buf.push(helper.alias());
        cx.buf.push(", ");
        cx.buf.push(source);
        cx.buf.push("]");
    } else {
        emit_modified_entry(cx, helper, source, &modifiers);
    }
    Ok(())
}

pub(super) fn emit_value(cx: &mut EmitCx<'_>, name: &str, source: &str) {
    push_ident_key(cx, name);
    cx.buf.push(": ");
    cx.buf.push(source);
}

pub(super) fn emit_update(cx: &mut EmitCx<'_>, key: &str, source: &str) {
    push_ident_key(cx, key);
    cx.buf.push(": ");
    emit_assignment(cx, source);
}

pub(super) fn emit_modifiers(cx: &mut EmitCx<'_>, name: &str, modifiers: &[&str]) {
    push_ident_key(cx, name);
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

pub(super) fn emit_assignment(cx: &mut EmitCx<'_>, source: &str) {
    cx.buf.push("$event => ((");
    cx.buf.push(source);
    cx.buf.push(") = $event)");
}

fn emit_modified_entry(cx: &mut EmitCx<'_>, helper: Helper, source: &str, modifiers: &[&str]) {
    cx.buf.push("  [");
    cx.buf.newline();
    cx.buf.push("    ");
    cx.buf.push(helper.alias());
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push("    ");
    cx.buf.push(source);
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push("    void 0,");
    cx.buf.newline();
    if modifiers.len() == 1 {
        cx.buf.push("    { ");
        cx.buf.push(modifiers[0]);
        cx.buf.push(": true }");
    } else {
        cx.buf.push("    {");
        for (i, modifier) in modifiers.iter().enumerate() {
            cx.buf.newline();
            cx.buf.push("      ");
            cx.buf.push(modifier);
            cx.buf.push(": true");
            if i + 1 < modifiers.len() {
                cx.buf.push(",");
            }
        }
        cx.buf.newline();
        cx.buf.push("    }");
    }
    cx.buf.newline();
    cx.buf.push("  ]");
}

fn native_helper(element: &ElementOp<'_>) -> Helper {
    match element.tag {
        "select" => Helper::VModelSelect,
        "textarea" => Helper::VModelText,
        "input" => match static_type(element) {
            Some("checkbox") => Helper::VModelCheckbox,
            Some("radio") => Helper::VModelRadio,
            _ => Helper::VModelText,
        },
        _ => Helper::VModelText,
    }
}

fn static_type<'a>(element: &'a ElementOp<'a>) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|attribute| attribute.name == "type")
        .and_then(|attribute| attribute.value)
}

fn js_source<'a>(model: &'a ModelOp<'a>) -> Result<&'a str, EmitError> {
    match model.contract.read {
        ExprRef::Js(js) => Ok(js.source),
        _ => Err(EmitError::Unsupported),
    }
}

fn is_component(model: &ModelOp<'_>) -> bool {
    attr(model, KIND) == Some("component")
}

fn argument<'a>(model: &'a ModelOp<'a>) -> Option<&'a str> {
    attr(model, ARGUMENT)
}

fn attr<'a>(model: &'a ModelOp<'a>, name: &str) -> Option<&'a str> {
    model
        .attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| attribute.value)
}

fn component_modifiers<'a>(model: &'a ModelOp<'a>) -> StdVec<&'a str> {
    model
        .attributes
        .iter()
        .filter(|attribute| attribute.name != KIND && attribute.name != ARGUMENT)
        .map(|attribute| attribute.name)
        .collect()
}

fn native_modifiers<'a>(model: &'a ModelOp<'a>) -> StdVec<&'a str> {
    component_modifiers(model)
        .into_iter()
        .filter(|modifier| matches!(*modifier, "lazy" | "number" | "trim"))
        .collect()
}

fn update_key(prop: &str) -> String {
    let mut key = String::from("onUpdate:");
    key.push_str(prop);
    key
}

fn modifiers_key(prop: &str) -> String {
    if prop == "modelValue" {
        return String::from("modelModifiers");
    }
    let mut key = String::from(prop);
    key.push_str("Modifiers");
    key
}

fn push_dynamic(dynamic_props: &mut StdVec<String>, name: &str) {
    let owned = String::from(name);
    if !dynamic_props.contains(&owned) {
        dynamic_props.push(owned);
    }
}
