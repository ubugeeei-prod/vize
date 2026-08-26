//! Inline static props for native element calls.

use vize_s0::String;
use vize_s2::op::{Attribute, ElementOp};

use super::EmitCx;
use super::hoist::{push_attr_pair, unique_attrs};

pub(super) fn root_should_hoist(element: &ElementOp<'_>) -> bool {
    element.bindings.is_empty()
        && !element.attributes.is_empty()
        && element
            .attributes
            .iter()
            .all(|attribute| attribute.name != "ref")
}

pub(super) fn emit_inline<'a>(
    cx: &mut EmitCx<'_>,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1 && !cx.in_v_for;
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        if cx.in_v_for && attr.name == "ref" {
            cx.buf.push("ref_for: true, ");
        }
        let mut pair = String::default();
        push_attr_pair(&mut pair, attr);
        cx.buf.push(pair.as_str());
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
}
