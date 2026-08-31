use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::Attribute;

use super::super::js::{escape_js_string, is_valid_js_identifier};

/// First-occurrence static attrs as a single-line object, matching
/// hoisted `JsChildNode::Object` emission.
pub(in crate::emit) fn compact_props_object<'a>(
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

pub(in crate::emit) fn unique_attrs<'a>(
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

pub(in crate::emit) fn push_attr_pair(out: &mut String, attr: &Attribute<'_>) {
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
