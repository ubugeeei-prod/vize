use alloc::vec::Vec as StdVec;

use vize_l0::String;
use vize_l2::op::Attribute;

use super::super::js::{escape_js_string, is_valid_js_identifier};

/// First-occurrence static attrs as a single-line object, matching
/// hoisted `JsChildNode::Object` emission.
pub(in crate::targets::dom::artifact) fn compact_props_object<'a>(
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
    scope_id: Option<&str>,
    escape_names: bool,
) -> String {
    let unique = unique_attrs(attributes);
    let scope = scope_id.filter(|scope| !unique.iter().any(|attr| attr.name == *scope));
    compact_props_object_from_unique(&unique, scope, escape_names)
}

fn compact_props_object_from_unique(
    unique: &[&Attribute<'_>],
    hoisted_scope_id: Option<&str>,
    escape_names: bool,
) -> String {
    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    for attr in unique.iter() {
        if emitted > 0 {
            out.push_str(", ");
        }
        push_attr_pair(&mut out, attr, escape_names);
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

pub(in crate::targets::dom::artifact) fn unique_attrs<'a>(
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

/// `escape_names`: the lexer saw a backslash in a name of this compile.
pub(in crate::targets::dom::artifact) fn push_attr_pair(
    out: &mut String,
    attr: &Attribute<'_>,
    escape_names: bool,
) {
    push_pair(out, attr.name, attr.value.unwrap_or_default(), escape_names);
}

/// A scope-id pair; its name is never authored.
pub(in crate::targets::dom::artifact) fn push_empty_attr_pair(out: &mut String, name: &str) {
    push_pair(out, name, "", false);
}

fn push_pair(out: &mut String, name: &str, value: &str, escape_names: bool) {
    let quoted = !is_valid_js_identifier(name);
    if quoted && escape_names {
        push_quoted_name_escaping(out, name);
    } else {
        if quoted {
            out.push('"');
        }
        out.push_str(name);
        if quoted {
            out.push('"');
        }
    }
    out.push_str(": \"");
    out.push_str(escape_js_string(value).as_str());
    out.push('"');
}

#[cold]
#[inline(never)]
fn push_quoted_name_escaping(out: &mut String, name: &str) {
    use vize_l0::general::quoted_name::{escaped_name_contents, name_needs_escape};
    out.push('"');
    if name_needs_escape(name) {
        out.push_str(escaped_name_contents(name).as_str());
    } else {
        out.push_str(name);
    }
    out.push('"');
}
