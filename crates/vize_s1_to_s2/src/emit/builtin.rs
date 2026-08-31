//! Vue built-in component helpers (`Teleport`, `KeepAlive`, `Transition`,
//! `Suspense`, `TransitionGroup`, `BaseTransition`).
//!
//! Teleport / KeepAlive take a raw children array (`generate_node` per
//! child). KeepAlive always carries `DYNAMIC_SLOTS`. Teleport / KeepAlive
//! / Suspense stay `createBlock` even when nested.

use vize_s2::op::{BindingOp, ComponentOp, DynamicName, Namespace, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::helper::Helper;
use super::outlet;
use super::props::bind_value;
use super::slots;

pub(super) fn helper(name: &str) -> Option<Helper> {
    match name {
        "Teleport" | "teleport" => Some(Helper::Teleport),
        "Suspense" | "suspense" => Some(Helper::Suspense),
        "KeepAlive" | "keep-alive" => Some(Helper::KeepAlive),
        "BaseTransition" | "base-transition" => Some(Helper::BaseTransition),
        "Transition" | "transition" => Some(Helper::Transition),
        "TransitionGroup" | "transition-group" => Some(Helper::TransitionGroup),
        _ => None,
    }
}

pub(super) fn is_reserved_name(name: &str) -> bool {
    helper(name).is_some() || matches!(name, "component" | "Component")
}

pub(super) fn forces_block(component: &ComponentOp<'_>) -> bool {
    matches!(
        component.name,
        "Teleport" | "teleport" | "Suspense" | "suspense" | "KeepAlive" | "keep-alive"
    ) || is_dynamic_component(component)
}

pub(super) fn is_dynamic_component(component: &ComponentOp<'_>) -> bool {
    matches!(component.name, "component" | "Component") && has_is(component)
}

fn has_is(component: &ComponentOp<'_>) -> bool {
    component.attributes.iter().any(|attr| attr.name == "is")
        || component.bindings.iter().any(is_is_bind)
}

pub(super) fn is_is_bind(binding: &BindingOp<'_>) -> bool {
    matches!(
        binding,
        BindingOp::Bind(bind) if matches!(bind.name, Some(DynamicName::Static("is")))
    )
}

pub(super) fn emit_dynamic_tag(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
) -> Result<bool, EmitError> {
    if !is_dynamic_component(component) {
        return Ok(false);
    }
    cx.buf.use_helper(Helper::ResolveDynamicComponent);
    cx.buf.push(Helper::ResolveDynamicComponent.alias());
    cx.buf.push("(");
    if let Some(BindingOp::Bind(bind)) = component.bindings.iter().find(|b| is_is_bind(b)) {
        bind_value(bind)?.emit_authored(cx, bind);
    } else if let Some(attr) = component.attributes.iter().find(|attr| attr.name == "is") {
        cx.buf.push("\"");
        if let Some(value) = attr.value {
            cx.buf.push(value);
        }
        cx.buf.push("\"");
    } else {
        return Err(EmitError::unsupported_at(
            Reason::UnsupportedBindingKind,
            component.span,
        ));
    }
    cx.buf.push(")");
    Ok(true)
}

pub(super) fn array_children(name: &str) -> bool {
    matches!(name, "Teleport" | "teleport" | "KeepAlive" | "keep-alive")
}

pub(super) fn always_dynamic_slots(name: &str) -> bool {
    matches!(name, "KeepAlive" | "keep-alive")
}

pub(super) fn transition_slot_root(name: &str) -> bool {
    matches!(
        name,
        "BaseTransition" | "base-transition" | "Transition" | "transition"
    )
}

/// `has_only_static_nested_children` over meaningful kids: unused
/// static-props hoist for Teleport + static attrs + text/native kids,
/// not for KeepAlive wrapping a component.
pub(super) fn has_static_nested(region: &Region<'_>) -> bool {
    let mut any = false;
    for op in region.ops.iter() {
        if slots::is_whitespace_text(op) {
            continue;
        }
        any = true;
        if !is_static_nested(op) {
            return false;
        }
    }
    any
}

fn is_static_nested(op: &Op<'_>) -> bool {
    match op {
        Op::Text(_) | Op::Interpolation(_) => true,
        Op::Element(element) => {
            element.namespace == Namespace::Html
                && element.tag != "template"
                && element.bindings.is_empty()
                && element.attributes.iter().all(|attr| attr.name != "ref")
                && element.children.ops.iter().all(is_static_nested)
        }
        _ => false,
    }
}

pub(super) fn emit_array_children(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    compact: bool,
) -> Result<(), EmitError> {
    cx.buf.push("[");
    outlet::emit_fallback(cx, children, compact)?;
    cx.buf.push("]");
    Ok(())
}
