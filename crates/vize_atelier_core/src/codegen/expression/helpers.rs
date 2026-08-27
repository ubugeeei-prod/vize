//! Expression generation helpers.
//!
//! Internal utilities for context-aware identifier prefixing,
//! line comment conversion, and slot parameter stripping.

use super::super::context::CodegenContext;
use vize_s0::String;

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn is_simple_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn props_access_expression(object: &str, key: &str) -> String {
    if is_simple_identifier_name(key) {
        let mut out = String::with_capacity(object.len() + key.len() + 1);
        out.push_str(object);
        out.push('.');
        out.push_str(key);
        return out;
    }

    let mut out = String::with_capacity(object.len() + key.len() + 4);
    out.push_str(object);
    out.push('[');
    use std::fmt::Write as _;
    let _ = write!(&mut out, "{:?}", key);
    out.push(']');
    out
}

fn replace_prefixed_alias_access(code: String, object: &str, local: &str, key: &str) -> String {
    let needle = {
        let mut needle = String::with_capacity(object.len() + local.len() + 1);
        needle.push_str(object);
        needle.push('.');
        needle.push_str(local);
        needle
    };
    let replacement = props_access_expression(object, key);

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0;
    while let Some(rel_pos) = code[cursor..].find(needle.as_str()) {
        let start = cursor + rel_pos;
        let end = start + needle.len();
        let after_ok = code[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_identifier_continue(c));

        result.push_str(&code[cursor..start]);
        if after_ok {
            result.push_str(&replacement);
        } else {
            result.push_str(&code[start..end]);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

pub(super) fn rewrite_props_aliases(code: String, ctx: &CodegenContext) -> String {
    let Some(bindings) = &ctx.options.binding_metadata else {
        return code;
    };
    if bindings.props_aliases.is_empty() {
        return code;
    }

    let mut rewritten = code;
    for (local, key) in &bindings.props_aliases {
        rewritten = replace_prefixed_alias_access(rewritten, "$props", local, key);
    }
    rewritten
}
