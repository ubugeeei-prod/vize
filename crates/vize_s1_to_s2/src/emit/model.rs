//! `ui.model` realization for the DOM lane: component product props
//! (`modelValue` / `onUpdate:…` / `modelModifiers`) and native
//! `withDirectives` + `vModelText`-family helpers. The S2 contract
//! itself is never expanded; this module is the P2-11 spelling.

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName, ElementOp, ModelOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::helper::Helper;
use super::js::{RawJs, expr_source};
use super::model_key::{self, ModelName};
use super::prefix::Site;
use super::props::Piece;

const KIND: &str = "element-kind";

pub(super) fn admit(model: &ModelOp<'_>) -> Result<(), EmitError> {
    js_source(model)?;
    argument(model)?;
    Ok(())
}

pub(super) fn expand<'a>(
    model: &'a ModelOp<'a>,
    out: &mut StdVec<Piece<'a>>,
) -> Result<(), EmitError> {
    js_source(model)?;
    let span = model.span;
    if is_component(model) {
        let prop = argument(model)?.unwrap_or(ModelName::Static("modelValue"));
        out.push(Piece::ModelValue {
            name: prop,
            model,
            span,
        });
        out.push(Piece::ModelUpdate {
            key: model_key::update_key_for(prop),
            model,
            span,
        });
        let modifiers = component_modifiers(model);
        if !modifiers.is_empty() {
            out.push(Piece::ModelModifiers {
                name: model_key::modifiers_key(prop),
                modifiers,
                span,
            });
        }
    } else {
        out.push(Piece::ModelUpdate {
            key: model_key::update_key_for(ModelName::Static("modelValue")),
            model,
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
    caches_handlers: bool,
) {
    if has_dynamic_argument(model) {
        *flag |= 16;
        return;
    }
    // The synthesized `onUpdate:` closure is cached like any other
    // handler, so it stops being a patch target; a component's value
    // prop still is.
    if is_component || !caches_handlers {
        *flag |= 8;
    }
    patch_keys(model, is_component, dynamic_props, caches_handlers);
}

pub(super) fn patch_keys(
    model: &ModelOp<'_>,
    is_component: bool,
    dynamic_props: &mut StdVec<String>,
    caches_handlers: bool,
) {
    if has_dynamic_argument(model) {
        return;
    }
    if is_component {
        let prop = static_argument(model).unwrap_or("modelValue");
        push_dynamic(dynamic_props, prop);
        if !caches_handlers {
            push_dynamic(dynamic_props, model_key::static_update_key(prop).as_str());
        }
    } else if !caches_handlers {
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
    let source = RawJs::Owned(native_read_source(cx, model)?);
    let modifiers = native_modifiers(model);
    cx.buf.use_helper(helper);
    if modifiers.is_empty() {
        cx.buf.push("  [");
        cx.buf.push(helper.alias());
        cx.buf.push(", ");
        cx.buf.push(source.as_str());
        cx.buf.push("]");
    } else {
        emit_modified_entry(cx, helper, &source, &modifiers);
    }
    Ok(())
}

fn emit_modified_entry(
    cx: &mut EmitCx<'_>,
    helper: Helper,
    source: &RawJs<'_>,
    modifiers: &[&str],
) {
    cx.buf.push("  [");
    cx.buf.newline();
    cx.buf.push("    ");
    cx.buf.push(helper.alias());
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push("    ");
    cx.buf.push(source.as_str());
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

/// The model's read expression as the shipped lane wrote it at `site`:
/// the transform-prefixed content, then `generate_expression`
/// (`Expression`) or a raw push (`Raw`, the dynamic-argument spelling).
pub(super) fn value_source(
    cx: &EmitCx<'_>,
    model: &ModelOp<'_>,
    site: Site,
) -> Result<String, EmitError> {
    let raw = js_source(model)?;
    if cx.prefixing() {
        return cx.prefixed_trimmed_expr(&model.contract.read, site);
    }
    Ok(String::from(raw.as_str()))
}

/// The `withDirectives` entry's read expression: `generate_expression`
/// over the directive node, whose content keeps the authored padding
/// inside the quotes (`v-model=" msg "` → `[_vModelText,  msg ]`); the
/// component `modelValue` value is trimmed by the shipped transform and
/// stays on [`value_source`].
fn native_read_source(cx: &EmitCx<'_>, model: &ModelOp<'_>) -> Result<String, EmitError> {
    let raw = js_source(model)?;
    if cx.prefixing() {
        return cx.prefixed_expr(&model.contract.read, Site::Expression);
    }
    let read = model.contract.read;
    let content = super::prefix::node_content(cx.source, read.source(), read.span());
    let padded = content.text.as_str();
    if padded.trim() != read.source() {
        return Ok(String::from(raw.as_str()));
    }
    let leading = padded.len() - padded.trim_start().len();
    let trailing = padded.len() - padded.trim_end().len();
    let mut out = String::with_capacity(padded.len());
    out.push_str(&padded[..leading]);
    out.push_str(raw.as_str());
    out.push_str(&padded[padded.len() - trailing..]);
    Ok(out)
}

pub(super) fn js_source<'a>(model: &'a ModelOp<'a>) -> Result<RawJs<'a>, EmitError> {
    expr_source(&model.contract.read, false).ok_or_else(|| {
        EmitError::unsupported_at(Reason::ModelExpressionNotJs, model.contract.read.span())
    })
}

fn is_component(model: &ModelOp<'_>) -> bool {
    attr(model, KIND) == Some("component")
}

fn argument<'a>(model: &'a ModelOp<'a>) -> Result<Option<ModelName<'a>>, EmitError> {
    match model.argument {
        None => Ok(None),
        Some(DynamicName::Static(name)) => Ok(Some(ModelName::Static(name))),
        Some(DynamicName::Dynamic(ExprRef::Js(js))) => Ok(Some(ModelName::Dynamic(js))),
        Some(DynamicName::Dynamic(expr)) => Err(EmitError::unsupported_at(
            Reason::ModelArgumentNotJs,
            expr.span(),
        )),
    }
}

fn static_argument<'a>(model: &'a ModelOp<'a>) -> Option<&'a str> {
    match model.argument {
        Some(DynamicName::Static(name)) => Some(name),
        _ => None,
    }
}

pub(super) fn has_dynamic_argument(model: &ModelOp<'_>) -> bool {
    matches!(model.argument, Some(DynamicName::Dynamic(_)))
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
        .filter(|attribute| attribute.name != KIND)
        .map(|attribute| attribute.name)
        .collect()
}

fn native_modifiers<'a>(model: &'a ModelOp<'a>) -> StdVec<&'a str> {
    component_modifiers(model)
        .into_iter()
        .filter(|modifier| matches!(*modifier, "lazy" | "number" | "trim"))
        .collect()
}

fn push_dynamic(dynamic_props: &mut StdVec<String>, name: &str) {
    let owned = String::from(name);
    if !dynamic_props.contains(&owned) {
        dynamic_props.push(owned);
    }
}
