//! unbound-method positions: receiver checks, accessed names and destructured keys.
use corsa::lint::LintNode;
use serde_json::Value;
use vize_l0::String;

/// `obj.method()` and `a.b.c` never unbind `this`; skip their checker work.
pub(super) fn is_receiver_position(node: &LintNode) -> bool {
    matches!(
        node.field_str("__parentKind"),
        Some("MemberExpression" | "CallExpression")
    ) && node
        .fields
        .get("__ancestorChain")
        .and_then(Value::as_array)
        .and_then(|chain| chain.first())
        .and_then(|parent| parent.get("childKey"))
        .and_then(Value::as_str)
        .is_some_and(|key| matches!(key, "callee" | "object"))
}

/// The statically known property name of a member access.
pub(super) fn accessed_name(node: &LintNode) -> Option<String> {
    let property = node.child("property")?;
    if node.field_bool("computed") == Some(true) {
        return match property.fields.get("value")? {
            Value::String(value) => Some(String::from(value.as_str())),
            Value::Number(value) => Some(vize_l0::cstr!("{value}")),
            _ => None,
        };
    }
    (property.kind == "Identifier")
        .then(|| property.field_str("name").map(String::from))
        .flatten()
}

pub(super) fn pattern_keys(node: &LintNode) -> Vec<String> {
    node.child_list("properties")
        .unwrap_or(&[])
        .iter()
        .filter(|property| {
            property.kind == "Property" && property.field_bool("computed") != Some(true)
        })
        .filter_map(|property| property.child("key"))
        .filter(|key| key.kind == "Identifier")
        .filter_map(|key| key.field_str("name").map(String::from))
        .collect()
}
