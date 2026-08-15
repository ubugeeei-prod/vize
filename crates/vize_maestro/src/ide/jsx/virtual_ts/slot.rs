//! Scoped-slot scopes for the editor's plain-TypeScript document.
//!
//! Mirrors `vize_canon`'s batch `jsx_codegen::slot`: a JSX slot object or
//! render-prop child lowers to a synthetic `<template v-slot:name="pattern">`,
//! whose expression is a **binding pattern** scoping the slot body. Re-emitting
//! it through the ordinary directive walk produced a fabricated
//! "Cannot find name '<pattern>'" and, worse, masked real errors inside the body
//! (#4042), so the pattern and body are emitted as one scope instead:
//!
//! ```text
//! __vize_jsx_component_slot__(<Host>, "<name>", (<pattern>) => __vize_jsx_expr__(<body…>))
//! ```
#![cfg_attr(not(any(test, feature = "native")), allow(dead_code))]

use vize_canon::virtual_ts::VizeMapping;
use vize_relief::{ElementNode, ElementType, PropNode};

use super::{JsxEmit, JsxExpr, collect, push_mapped_expr};

/// A scoped slot: the host component's tag, the slot name, the binding pattern
/// the slot introduces, and the body evaluated with that pattern in scope.
pub(super) struct JsxSlotScope {
    host: String,
    name: String,
    params: JsxExpr,
    body: Vec<JsxEmit>,
}

impl JsxSlotScope {
    pub(super) fn params(&self) -> &JsxExpr {
        &self.params
    }

    pub(super) fn body(&self) -> &[JsxEmit] {
        &self.body
    }
}

/// Recognize a synthesized scoped-slot `<template>` child of `host`.
///
/// Returns `None` for anything that is not a `<template>` carrying a `v-slot`
/// directive with a binding pattern: a *non-scoped* slot introduces no bindings,
/// so its body is collected in the enclosing scope exactly as before.
pub(super) fn collect(
    element: &ElementNode<'_>,
    host: &JsxExpr,
    preserve_components: bool,
) -> Option<JsxSlotScope> {
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
        .unwrap_or_else(|| "default".to_string());

    let mut body = Vec::new();
    for child in &element.children {
        collect::collect_child(child, &mut body, preserve_components, None);
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
/// The host tag and slot-name literal are scaffolding and stay **unmapped** —
/// the tag is already mapped by the component call this slot belongs to, and
/// mapping it twice would double-report any diagnostic landing on it. Only the
/// authored binding pattern is mapped.
pub(super) fn render_open(out: &mut String, mappings: &mut Vec<VizeMapping>, scope: &JsxSlotScope) {
    out.push_str("__vize_jsx_component_slot__(");
    out.push_str(&scope.host);
    out.push_str(", ");
    out.push_str(&json_string(&scope.name));
    out.push_str(", (");
    push_mapped_expr(out, mappings, &scope.params);
    out.push_str(") => ");
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a Rust string to JSON cannot fail")
}
