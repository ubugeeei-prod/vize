//! `rewrite_props_aliases`, ported: the destructured-prop alias projection
//! the shipped lane runs as a find/replace post-pass over the rewritten
//! text (`$props.local` → `$props.key`, or `$props["key"]` when the key
//! is not an identifier). The transform projects both `__props` and
//! `$props`; the codegen visitor projects `$props` alone.

use core::fmt::Write as _;

use vize_s0::String;

use super::super::options::BindingTable;
use super::globals::is_simple_identifier;

pub(super) fn rewrite_props_aliases(
    code: String,
    bindings: Option<&BindingTable>,
    objects: &[&str],
) -> String {
    let Some(table) = bindings else {
        return code;
    };
    let mut rewritten = code;
    for (local, key) in table.aliases() {
        for object in objects {
            rewritten = replace_prefixed_alias_access(rewritten, object, local, key);
        }
    }
    rewritten
}

fn prop_access_expression(object: &str, key: &str) -> String {
    let mut out = String::with_capacity(object.len() + key.len() + 4);
    out.push_str(object);
    if is_simple_identifier(key) {
        out.push('.');
        out.push_str(key);
        return out;
    }
    out.push('[');
    // The shipped lane spells the key with `{:?}` (Rust string escaping).
    let _ = write!(&mut out, "{key:?}");
    out.push(']');
    out
}

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn replace_prefixed_alias_access(code: String, object: &str, local: &str, key: &str) -> String {
    let mut needle = String::with_capacity(object.len() + local.len() + 1);
    needle.push_str(object);
    needle.push('.');
    needle.push_str(local);
    if !code.contains(needle.as_str()) {
        return code;
    }
    let replacement = prop_access_expression(object, key);

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
            result.push_str(replacement.as_str());
        } else {
            result.push_str(&code[start..end]);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

#[cfg(test)]
mod tests {
    use super::rewrite_props_aliases;
    use crate::emit::options::{BindingKind, BindingTable};
    use vize_s0::String;

    #[test]
    fn aliases_project_onto_their_prop_keys() {
        let table = BindingTable::new(
            [
                ("label", BindingKind::PropsAliased),
                ("x", BindingKind::PropsAliased),
            ],
            [("label", "aria-label"), ("x", "posX")],
            true,
        );
        let code = String::from("$props.label + $props.labelled + __props.x + $props.x");
        assert_eq!(
            rewrite_props_aliases(code, Some(&table), &["__props", "$props"]).as_str(),
            "$props[\"aria-label\"] + $props.labelled + __props.posX + $props.posX"
        );
        let untouched = String::from("$props.label");
        assert_eq!(
            rewrite_props_aliases(untouched, None, &["$props"]).as_str(),
            "$props.label"
        );
    }
}
