//! Vue built-in component helpers (`Teleport`, `KeepAlive`, `Transition`,
//! `Suspense`, `TransitionGroup`, `BaseTransition`).
//!
//! Teleport / KeepAlive take a raw children array (`generate_node` per
//! child). KeepAlive always carries `DYNAMIC_SLOTS`. Teleport / KeepAlive
//! / Suspense stay `createBlock` even when nested.

use vize_disegno::op::{Namespace, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::helper::Helper;
use super::outlet;
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

pub(super) fn forces_block(name: &str) -> bool {
    matches!(
        name,
        "Teleport" | "teleport" | "Suspense" | "suspense" | "KeepAlive" | "keep-alive"
    )
}

pub(super) fn array_children(name: &str) -> bool {
    matches!(name, "Teleport" | "teleport" | "KeepAlive" | "keep-alive")
}

pub(super) fn always_dynamic_slots(name: &str) -> bool {
    matches!(name, "KeepAlive" | "keep-alive")
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
