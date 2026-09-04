use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::Attribute;

use super::super::js::{escape_js_string, is_valid_js_identifier};

/// First-occurrence static attrs as a single-line object, matching
/// hoisted `JsChildNode::Object` emission.
pub(in crate::emit) fn compact_props_object<'a>(
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
    scope_id: Option<&str>,
) -> String {
    let unique = unique_attrs(attributes);
    let scope = scope_id.filter(|scope| !unique.iter().any(|attr| attr.name == *scope));
    compact_props_object_from_unique(&unique, scope)
}

fn compact_props_object_from_unique(
    unique: &[&Attribute<'_>],
    hoisted_scope_id: Option<&str>,
) -> String {
    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    for attr in unique.iter() {
        if emitted > 0 {
            out.push_str(", ");
        }
        push_attr_pair(&mut out, attr);
        emitted += 1;
    }
    if let Some(scope_id) = hoisted_scope_id {
        if emitted > 0 {
            out.push_str(", ");
        }
        push_empty_attr_pair(&mut out, scope_id);
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
    push_pair(out, attr.name, attr.value.unwrap_or_default());
}

pub(in crate::emit) fn push_empty_attr_pair(out: &mut String, name: &str) {
    push_pair(out, name, "");
}

fn push_pair(out: &mut String, name: &str, value: &str) {
    let quoted = !is_valid_js_identifier(name);
    if quoted {
        out.push('"');
    }
    out.push_str(name);
    if quoted {
        out.push('"');
    }
    out.push_str(": \"");
    out.push_str(escape_js_string(value).as_str());
    out.push('"');
}
