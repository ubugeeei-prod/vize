//! Custom `vue.directive` realization: `resolveDirective` assets and
//! `_withDirectives` wrap, merging a native `v-model` entry first when
//! the owner is `input` / `textarea` / `select`. `v-slots` stays a
//! slots-object spread, not a runtime directive.

use alloc::vec::Vec as StdVec;

use vize_s2::expr::ExprRef;
use vize_s2::op::{
    BindingOp, ComponentOp, DynamicName, ElementOp, ModelOp, Op, Region, VueDirectiveOp,
};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::helper::Helper;
use super::js::asset_ident;
use super::slots;

pub(super) fn is_custom(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::VueDirective(_)) && !slots::is_slots_spread(binding)
}

pub(super) fn has_custom(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(is_custom)
}

pub(super) fn prefer_helpers(buf: &mut Buf, bindings: &[BindingOp<'_>]) {
    if has_custom(bindings) {
        buf.prefer(Helper::WithDirectives);
        buf.prefer(Helper::ResolveDirective);
    }
}

pub(super) fn collect_names<'a>(root: &'a Region<'a>) -> StdVec<&'a str> {
    let mut names = StdVec::new();
    collect_from(root, &mut names);
    names
}

pub(super) fn emit_resolves(cx: &mut EmitCx<'_>, names: &[&str]) {
    cx.buf.use_resolve_directive();
    for name in names {
        cx.buf.push("const ");
        cx.buf.push(asset_ident("directive", name).as_str());
        cx.buf.push(" = ");
        cx.buf.push(Buf::resolve_directive_alias());
        cx.buf.push("(\"");
        cx.buf.push(name);
        cx.buf.push("\")");
        cx.buf.newline();
    }
}

pub(super) fn wrap_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let custom = customs(&element.bindings);
    let model = super::model::first_runtime_model(element);
    if custom.is_empty() && model.is_none() {
        return emit(cx);
    }
    wrap(cx, model.map(|model| (element, model)), &custom, emit)
}

pub(super) fn wrap_component(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let custom = customs(&component.bindings);
    if custom.is_empty() {
        return emit(cx);
    }
    wrap(cx, None, &custom, emit)
}

pub(super) fn admit(directive: &VueDirectiveOp<'_>) -> Result<(), EmitError> {
    if let Some(value) = directive.value {
        js_expr(value)?;
    }
    if let Some(DynamicName::Dynamic(expr)) = directive.argument {
        js_expr(expr)?;
    }
    Ok(())
}

fn wrap(
    cx: &mut EmitCx<'_>,
    native: Option<(&ElementOp<'_>, &ModelOp<'_>)>,
    custom: &[&VueDirectiveOp<'_>],
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    cx.buf.use_with_directives();
    cx.buf.push(Buf::with_directives_alias());
    cx.buf.push("(");
    emit(cx)?;
    cx.buf.push(", [");
    cx.buf.newline();
    let mut first = true;
    if let Some((element, model)) = native {
        super::model::emit_native_entry(cx, element, model)?;
        first = false;
    }
    for directive in custom.iter() {
        if !first {
            cx.buf.push(",");
            cx.buf.newline();
        }
        emit_entry(cx, directive)?;
        first = false;
    }
    cx.buf.newline();
    cx.buf.push("])");
    Ok(())
}

fn emit_entry(cx: &mut EmitCx<'_>, directive: &VueDirectiveOp<'_>) -> Result<(), EmitError> {
    cx.buf.push("  [");
    cx.buf
        .push(asset_ident("directive", directive.name).as_str());
    let value = match directive.value {
        Some(expr) => Some(js_expr(expr)?),
        None => None,
    };
    if let Some(source) = value {
        cx.buf.push(", ");
        cx.buf.push(source);
    }
    if let Some(argument) = directive.argument {
        if value.is_none() {
            cx.buf.push(", void 0");
        }
        cx.buf.push(", ");
        emit_argument(cx, argument)?;
    }
    if !directive.modifiers.is_empty() {
        if value.is_none() && directive.argument.is_none() {
            cx.buf.push(", void 0, void 0");
        } else if directive.argument.is_none() {
            cx.buf.push(", void 0");
        }
        cx.buf.push(", { ");
        for (i, modifier) in directive.modifiers.iter().enumerate() {
            if i > 0 {
                cx.buf.push(", ");
            }
            cx.buf.push(modifier);
            cx.buf.push(": true");
        }
        cx.buf.push(" }");
    }
    cx.buf.push("]");
    Ok(())
}

fn emit_argument(cx: &mut EmitCx<'_>, argument: DynamicName<'_>) -> Result<(), EmitError> {
    match argument {
        DynamicName::Static(name) => {
            cx.buf.push("\"");
            cx.buf.push(name);
            cx.buf.push("\"");
            Ok(())
        }
        DynamicName::Dynamic(expr) => {
            cx.buf.push(js_expr(expr)?);
            Ok(())
        }
    }
}

fn customs<'a>(bindings: &'a [BindingOp<'a>]) -> StdVec<&'a VueDirectiveOp<'a>> {
    bindings
        .iter()
        .filter_map(|binding| match binding {
            BindingOp::VueDirective(directive) if !slots::is_slots_spread(binding) => {
                Some(&**directive)
            }
            _ => None,
        })
        .collect()
}

fn collect_from<'a>(region: &'a Region<'a>, names: &mut StdVec<&'a str>) {
    for op in region.ops.iter() {
        match op {
            Op::Element(element) => {
                push_from(&element.bindings, names);
                collect_from(&element.children, names);
            }
            Op::Component(component) => {
                push_from(&component.bindings, names);
                collect_from(&component.children, names);
            }
            Op::Slot(slot) => {
                push_from(&slot.bindings, names);
                collect_from(&slot.fallback, names);
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    collect_from(&branch.region, names);
                }
            }
            Op::For(for_op) => collect_from(&for_op.region, names),
            Op::Text(_) | Op::Interpolation(_) => {}
        }
    }
}

fn push_from<'a>(bindings: &'a [BindingOp<'a>], names: &mut StdVec<&'a str>) {
    for binding in bindings.iter() {
        let Some(name) = custom_name(binding) else {
            continue;
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }
}

fn custom_name<'a>(binding: &'a BindingOp<'a>) -> Option<&'a str> {
    match binding {
        BindingOp::VueDirective(directive) if !slots::is_slots_spread(binding) => {
            Some(directive.name)
        }
        _ => None,
    }
}

fn js_expr(expr: ExprRef<'_>) -> Result<&str, EmitError> {
    match expr {
        ExprRef::Js(js) => Ok(js.source),
        _ => Err(EmitError::Unsupported),
    }
}
