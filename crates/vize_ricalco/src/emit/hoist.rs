//! Static vnode hoist (`/*#__PURE__*/ _createElementVNode(...)`).
//!
//! Shared by implicit default-slot children and static `ui.for` items.
//! Named / scoped slots and nested hoist-out-of-dynamic-parents stay
//! with later installments.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::op::{Attribute, ElementOp, Namespace, Op, Region};

use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};
use super::EmitCx;

pub(super) fn emit_hoisted_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), super::EmitError> {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    let alias = hoist_static_element(cx, element);
    cx.buf.push(alias.as_str());
    Ok(())
}

pub(super) fn hoist_static_element(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> String {
    walk_hoisted(cx, element);
    cx.buf.use_create_element_vnode();
    if hoist_needs_create_text(element) {
        cx.buf.use_create_text();
    }
    cx.buf.push_hoist(hoist_element_rhs(element, true))
}

pub(super) fn is_hoistable(element: &ElementOp<'_>) -> bool {
    element.namespace == Namespace::Html
        && element.tag != "template"
        && element.bindings.is_empty()
        && element.children.ops.iter().all(is_hoistable_child)
}

fn is_hoistable_child(op: &Op<'_>) -> bool {
    match op {
        Op::Text(_) => true,
        Op::Element(element) => is_hoistable(element),
        _ => false,
    }
}

fn walk_hoisted(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) {
    for op in element.children.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) => {
                let _id = cx.walk.mint();
            }
            Op::Element(child) => {
                let _id = cx.walk.mint();
                cx.walk.skip(child.bindings.len());
                walk_hoisted(cx, child);
            }
            _ => {}
        }
    }
}

fn hoist_needs_create_text(element: &ElementOp<'_>) -> bool {
    let kids = meaningful(&element.children);
    let has_text = kids.iter().any(|op| matches!(op, Op::Text(_)));
    let has_other = kids.iter().any(|op| !matches!(op, Op::Text(_)));
    (has_text && has_other)
        || kids.iter().any(|op| match op {
            Op::Element(child) => hoist_needs_create_text(child),
            _ => false,
        })
}

fn hoist_element_rhs(element: &ElementOp<'_>, pure: bool) -> String {
    let mut out = String::default();
    if pure {
        out.push_str("/*#__PURE__*/ ");
    }
    out.push_str(Buf::create_element_vnode_alias());
    out.push('(');
    out.push('"');
    out.push_str(element.tag);
    out.push('"');
    let kids = meaningful(&element.children);
    let has_attrs = !element.attributes.is_empty();
    if has_attrs || !kids.is_empty() {
        out.push_str(", ");
        if has_attrs {
            out.push_str(compact_props_object(element.attributes.iter()).as_str());
        } else {
            out.push_str("null");
        }
    }
    if !kids.is_empty() {
        out.push_str(", ");
        append_hoist_kids(&mut out, &kids);
    }
    out.push(')');
    out
}

fn append_hoist_kids(out: &mut String, kids: &[&Op<'_>]) {
    if kids.iter().all(|op| matches!(op, Op::Text(_))) {
        out.push('"');
        for op in kids.iter() {
            if let Op::Text(text) = op {
                out.push_str(escape_js_string(text.content).as_str());
            }
        }
        out.push('"');
        return;
    }
    out.push('[');
    for (i, op) in kids.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        match op {
            Op::Text(text) => {
                out.push_str(Buf::create_text_alias());
                out.push('(');
                out.push('"');
                out.push_str(escape_js_string(text.content).as_str());
                out.push('"');
                out.push(')');
            }
            Op::Element(element) => {
                out.push_str(hoist_element_rhs(element, false).as_str());
            }
            _ => {}
        }
    }
    out.push(']');
}

fn meaningful<'a>(children: &'a Region<'a>) -> StdVec<&'a Op<'a>> {
    children
        .ops
        .iter()
        .filter(|op| !is_whitespace_text(op))
        .collect()
}

fn is_whitespace_text(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if text.content.chars().all(char::is_whitespace))
}

/// First-occurrence static attrs as a single-line object, matching
/// hoisted `JsChildNode::Object` emission.
pub(super) fn compact_props_object<'a>(
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) -> String {
    let unique = unique_attrs(attributes);
    let mut out = String::from("{ ");
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        push_attr_pair(&mut out, attr);
    }
    out.push_str(" }");
    out
}

pub(super) fn unique_attrs<'a>(
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) -> StdVec<&'a Attribute<'a>> {
    let mut unique: StdVec<&Attribute<'_>> = StdVec::new();
    for attr in attributes {
        if unique.iter().any(|seen| seen.name == attr.name) {
            continue;
        }
        unique.push(attr);
    }
    unique
}

pub(super) fn push_attr_pair(out: &mut String, attr: &Attribute<'_>) {
    let quoted = !is_valid_js_identifier(attr.name);
    if quoted {
        out.push('"');
    }
    out.push_str(attr.name);
    if quoted {
        out.push('"');
    }
    out.push_str(": \"");
    if let Some(value) = attr.value {
        out.push_str(escape_js_string(value).as_str());
    }
    out.push('"');
}
