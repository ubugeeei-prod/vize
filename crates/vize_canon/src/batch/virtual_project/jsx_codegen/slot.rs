//! Scoped-slot scopes for the plain-TypeScript JSX check document.
//!
//! `@vue/babel-plugin-jsx` writes slots as an object child
//! (`<List>{{ item: (row) => <li>{row.label}</li> }}</List>`) or a render-prop
//! child (`<List>{(row) => <li/>}</List>`); the JSX lowering turns both into the
//! same synthetic `<template v-slot:name="pattern">` element the SFC template
//! path produces.
//!
//! That `v-slot` expression is a **binding pattern**, not a readable value, and
//! it scopes the slot body. Re-emitting it through the ordinary directive walk
//! therefore produced a bare read of an undeclared name — a fabricated
//! "Cannot find name '<pattern>'" — and evaluated the body outside that scope,
//! which additionally *masked* real errors inside the body because every
//! reference to the slot parameter resolved to an error type (#4042).
//!
//! This module re-emits the pattern and body as one scope instead, mirroring how
//! [`JsxEmit::ForScope`](super::JsxEmit::ForScope) handles `v-for` aliases:
//!
//! ```text
//! __vize_jsx_component_slot__(<Host>, "<name>", (<pattern>) => __vize_jsx_expr__(<body…>))
//! ```
//!
//! The parameter's type comes from the host component's declared `$slots` via
//! `__VizeJsxSlotPayload`, the JSX analogue of the `.vue` template path's
//! `slot_props_type`, and falls back to `any` whenever the host or the slot is
//! untyped so untyped slot hosts never produce a false positive.

use vize_carton::{String as CompactString, ToCompactString};
use vize_relief::{ElementNode, ElementType, PropNode};

use crate::virtual_ts::VizeMapping;

use super::{JsxEmit, JsxExpr, collect, push_mapped_expr};

/// A scoped slot: the host component's tag, the slot name, the binding pattern
/// the slot introduces, and the body evaluated with that pattern in scope.
pub(super) struct JsxSlotScope {
    host: CompactString,
    name: CompactString,
    params: JsxExpr,
    body: Vec<JsxEmit>,
}

impl JsxSlotScope {
    pub(super) fn body(&self) -> &[JsxEmit] {
        &self.body
    }
}

/// Recognize a synthesized scoped-slot `<template>` child of `host`.
///
/// Returns `None` for anything that is not a `<template>` carrying a `v-slot`
/// directive with a binding pattern: a *non-scoped* slot introduces no bindings,
/// so its body is collected in the enclosing scope exactly as before.
pub(super) fn collect(element: &ElementNode<'_>, host: &JsxExpr) -> Option<JsxSlotScope> {
    if element.tag_type != ElementType::Template {
        return None;
    }
    let directive = element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive) if directive.name == "slot" => Some(directive),
        _ => None,
    })?;
    let params = collect::alias_expr(directive.exp.as_ref()?)?;
    let name = directive
        .arg
        .as_ref()
        .and_then(collect::static_text)
        .unwrap_or_else(|| "default".to_compact_string());

    let mut body = Vec::new();
    for child in &element.children {
        collect::collect_child(child, &mut body, None);
    }
    Some(JsxSlotScope {
        host: host.content.clone(),
        name,
        params,
        body,
    })
}

/// Emit the scope opening; the caller renders the body sink call and closes it.
///
/// The host tag is re-emitted **unmapped**: it is already mapped by the
/// component call this slot belongs to, and mapping it twice would double-report
/// any diagnostic that lands on the tag. The slot-name literal is scaffolding
/// and stays unmapped for the same reason; only the authored binding pattern is
/// mapped, because that is where a diagnostic about the pattern belongs.
pub(super) fn render_open(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    scope: &JsxSlotScope,
) {
    out.push_str("__vize_jsx_component_slot__(");
    out.push_str(&scope.host);
    out.push_str(", ");
    out.push_str(&json_string(&scope.name));
    out.push_str(", (");
    push_mapped_expr(out, mappings, &scope.params);
    out.push_str(") => ");
}

fn json_string(value: &str) -> CompactString {
    serde_json::to_string(value)
        .expect("serializing a Rust string to JSON cannot fail")
        .to_compact_string()
}
